//! Provides a wrapper of a [v8::Isolate] that can be shared between threads by making serialised
//! requests against it.
#![allow(dead_code)]

#[cfg(test)]
mod test;

use anyhow::{Context, Error, Result, anyhow, bail};

static INIT_V8: std::sync::OnceLock<()> = std::sync::OnceLock::new();

/// Initialises [v8]. Must be called before any other functions in [v8] or this crate. Can safely
/// be called multiple times. Must be called from the main thread.
pub fn init_v8() {
    INIT_V8.get_or_init(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

/// Initialises [v8]. Must be called before any other functions in [v8] or this crate. Can safely
/// be called multiple times. May be called from any thread, but provides fewer security
/// protections, and so is appropriate to call from tests.
pub fn init_v8_for_testing() {
    INIT_V8.get_or_init(|| {
        let platform = v8::new_unprotected_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

pub type TryScope<'s, 'p> = v8::TryCatch<'s, v8::HandleScope<'p>>;

/// Origin of some code.
#[derive(Debug)]
pub struct ESScriptOrigin {
    pub resource_name: String,
    pub resource_line_offset: i32,
    pub resource_column_offset: i32,
    pub script_id: i32,
}

impl ESScriptOrigin {
    pub fn make_origin<'s>(
        &self,
        try_catch: &mut TryScope<'_, 's>,
    ) -> Result<v8::ScriptOrigin<'s>> {
        let resource_name_v8 = new_v8_string(try_catch, &self.resource_name)?.try_cast()?;
        Ok(v8::ScriptOrigin::new(
            try_catch,
            resource_name_v8,
            self.resource_line_offset,
            self.resource_column_offset,
            false,
            self.script_id,
            None,
            false,
            false,
            false,
            None,
        ))
    }
}

impl Default for ESScriptOrigin {
    fn default() -> Self {
        Self {
            resource_name: Default::default(),
            resource_line_offset: Default::default(),
            resource_column_offset: Default::default(),
            script_id: -1,
        }
    }
}

thread_local! {
    static ISOLATE: std::cell::RefCell<Option<TlsIsolateGuard>> = const {
        std::cell::RefCell::new(None)
    };
}

/// Upon creation creates a [v8::OwnedIsolate] for the current thread, and destroys it upon [Drop].
pub struct TlsIsolate(
    // Uses the `PhantomData *const` to implicitly mark as neither [Send] nor [Sync] until
    // https://github.com/rust-lang/rust/issues/68318 is resolved.
    std::marker::PhantomData<*const ()>,
);

impl TlsIsolate {
    /// Creates a [TlsIsolate] for the current thread, which enables calls to [try_with_isolate]
    /// until it is dropped.
    ///
    /// This explicitly manages the lifetime of the thread local [v8::OwnedIsolate].
    pub fn for_current_thread() -> Result<Self> {
        ISOLATE.try_with(|isolate_refcell| {
            if isolate_refcell.borrow().is_some() {
                bail!("TlsIsolate already exists for this thread");
            }
            isolate_refcell.replace(Some(TlsIsolateGuard::new()));
            Ok(Self(std::marker::PhantomData))
        })?
    }
}

impl Drop for TlsIsolate {
    fn drop(&mut self) {
        let result = ISOLATE.try_with(|isolate_refcell| {
            let _ = isolate_refcell.borrow_mut().take();
        });
        if let Err(err) = result {
            log::warn!("Could not drop TlsIsolate: {err}.");
        }
    }
}

/// Runs the given lambda with the thread's local v8 isolate.
pub fn try_with_isolate<F, R>(f: F) -> Result<R, TlsIsolateError>
where
    F: FnOnce(&mut TlsIsolateGuard) -> R,
{
    ISOLATE
        .try_with(|isolate| -> Result<R, TlsIsolateError> {
            let mut isolate_opt_ref = isolate.borrow_mut();
            let isolate_ref = isolate_opt_ref.as_mut().ok_or(TlsIsolateError::NotExist)?;
            Ok(f(isolate_ref))
        })
        .map_err(TlsIsolateError::AccessError)
        .and_then(|r| r)
}

pub struct TlsIsolateGuard {
    isolate: v8::OwnedIsolate,
}

impl TlsIsolateGuard {
    fn new() -> Self {
        Self {
            isolate: v8::Isolate::new(v8::CreateParams::default()),
        }
    }

    pub fn isolate(&mut self) -> &mut v8::OwnedIsolate {
        &mut self.isolate
    }

    pub fn scope<'s>(&'s mut self) -> v8::HandleScope<'s, ()> {
        v8::HandleScope::new(&mut self.isolate)
    }

    pub fn new_ctx(&mut self) -> v8::Global<v8::Context> {
        let mut scope = self.scope();
        let ctx = v8::Context::new(&mut scope, v8::ContextOptions::default());
        v8::Global::new(&mut scope, ctx)
    }
}

/// Describes the cause of an error with [try_with_isolate].
#[derive(Debug)]
pub enum TlsIsolateError {
    /// Error accessing the thread local.
    AccessError(std::thread::AccessError),
    /// [TlsIsolate] does not exist for the current thread.
    NotExist,
}

impl std::fmt::Display for TlsIsolateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TlsIsolateError::AccessError(_) => write!(f, "AccessError"),
            TlsIsolateError::NotExist => {
                write!(f, "TlsIsolate does not exist on the current thread")
            }
        }
    }
}

impl std::error::Error for TlsIsolateError {}

/// Always returns an `Err`, but will use information from the given [v8::TryCatch] if a message is
/// present.
pub fn try_catch_to_result(try_catch: &mut TryScope<'_, '_>) -> Error {
    match try_catch.message() {
        None => anyhow!("unknown cause"),
        Some(msg) => {
            let text = msg.get(try_catch).to_rust_string_lossy(try_catch);
            let line_number_str = msg
                .get_line_number(try_catch)
                .as_ref()
                .map(usize::to_string)
                .unwrap_or_else(|| "?".to_string());
            let src_name = msg
                .get_script_resource_name(try_catch)
                .map(|v| v.to_rust_string_lossy(try_catch))
                .unwrap_or_else(|| "?".to_string());
            anyhow!("{}:{}: {}", src_name, line_number_str, text)
        }
    }
}

/// Creates a new [v8::String].
pub fn new_v8_string<'s>(
    try_catch: &mut TryScope<'_, 's>,
    string: &str,
) -> Result<v8::Local<'s, v8::String>> {
    v8::String::new(try_catch, string).ok_or_else(|| try_catch_to_result(try_catch))
}

/// Creates a new [v8::Number].
pub fn new_v8_number<'s>(
    try_catch: &mut TryScope<'_, 's>,
    number: f64,
) -> v8::Local<'s, v8::Number> {
    v8::Number::new(try_catch, number)
}

/// Creates a new [v8::Function].
pub fn new_v8_function<'s>(
    try_catch: &mut TryScope<'s, '_>,
    arg_names: &[&str],
    origin: &ESScriptOrigin,
    source: &str,
) -> anyhow::Result<v8::Local<'s, v8::Function>> {
    let body_str_v8 = new_v8_string(try_catch, source).context("creating body string")?;
    let origin_v8 = origin.make_origin(try_catch).context("creating origin")?;
    let mut body_src_v8 = v8::script_compiler::Source::new(body_str_v8, Some(&origin_v8));

    let arg_names_v8: Vec<v8::Local<v8::String>> = arg_names
        .iter()
        .map(|arg_name| new_v8_string(try_catch, arg_name))
        .collect::<anyhow::Result<Vec<_>>>()?;

    v8::script_compiler::compile_function(
        try_catch,
        &mut body_src_v8,
        &arg_names_v8,
        &[],
        v8::script_compiler::CompileOptions::NoCompileOptions,
        v8::script_compiler::NoCacheReason::NoReason,
    )
    .ok_or_else(|| try_catch_to_result(try_catch))
    .with_context(|| "could not compile function")
}
