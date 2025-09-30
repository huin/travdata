//! Provides thread-local instances of [v8::Isolate]s.
#![allow(dead_code)]

pub mod modules;

#[cfg(test)]
mod test;

use anyhow::{Result, bail};

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

/// Origin of some code.
#[derive(Debug)]
pub struct ESScriptOrigin {
    pub resource_name: String,
    pub resource_line_offset: i32,
    pub resource_column_offset: i32,
    pub script_id: i32,
    pub is_module: bool,
}

impl ESScriptOrigin {
    pub fn try_make_origin<'s>(
        &self,
        scope: &mut v8::HandleScope<'s>,
    ) -> ExceptionResult<v8::ScriptOrigin<'s>> {
        let try_catch = &mut v8::TryCatch::new(scope);
        self.make_origin(try_catch).to_exception_result(try_catch)
    }

    pub fn make_origin<'s>(&self, scope: &mut v8::HandleScope<'s>) -> Option<v8::ScriptOrigin<'s>> {
        let resource_name_v8: v8::Local<v8::Value> =
            v8::String::new(scope, &self.resource_name)?.cast();
        Some(v8::ScriptOrigin::new(
            scope,
            resource_name_v8,
            self.resource_line_offset,
            self.resource_column_offset,
            false,
            self.script_id,
            None,
            false,
            false,
            self.is_module,
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
            is_module: false,
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

/// Wraps [v8::String::new], translating any thrown exception into an [ExceptionResult].
pub fn new_v8_string<'s>(
    scope: &mut v8::HandleScope<'s>,
    value: &str,
) -> ExceptionResult<v8::Local<'s, v8::String>> {
    let try_catch = &mut v8::TryCatch::new(scope);
    v8::String::new(try_catch, value).to_exception_result(try_catch)
}

/// Creates a new [v8::Function]. Translates any thrown exception into an [ExceptionResult].
pub fn new_v8_function<'s>(
    scope: &mut v8::HandleScope<'s>,
    arg_names: &[&str],
    origin: &ESScriptOrigin,
    source: &str,
) -> ExceptionResult<v8::Local<'s, v8::Function>> {
    let try_catch = &mut v8::TryCatch::new(scope);
    let body_str_v8 = v8::String::new(try_catch, source).to_exception_result(try_catch)?;
    let origin_v8 = origin
        .make_origin(try_catch)
        .to_exception_result(try_catch)?;
    let mut body_src_v8 = v8::script_compiler::Source::new(body_str_v8, Some(&origin_v8));

    let arg_names_v8: Vec<v8::Local<v8::String>> = arg_names
        .iter()
        .map(|arg_name| v8::String::new(try_catch, arg_name))
        .collect::<Option<Vec<_>>>()
        .to_exception_result(try_catch)?;

    v8::script_compiler::compile_function(
        try_catch,
        &mut body_src_v8,
        &arg_names_v8,
        &[],
        v8::script_compiler::CompileOptions::NoCompileOptions,
        v8::script_compiler::NoCacheReason::NoReason,
    )
    .to_exception_result(try_catch)
}

// TODO: Look into creating a non-lossy version of to_rust_string_lossy, using
// `utf8_length` and `write_utf8_v2`.

/// Copies all "own" properties from `src` to `dest`.
pub fn shallow_copy_object_properties<'s, 'a, 'b>(
    scope: &mut v8::HandleScope<'s>,
    src: v8::Local<'a, v8::Object>,
    dest: v8::Local<'b, v8::Object>,
) -> ExceptionResult<()> {
    let try_catch = &mut v8::TryCatch::new(scope);
    let keys = src
        .get_own_property_names(try_catch, v8::GetPropertyNamesArgs::default())
        .to_exception_result(try_catch)?;
    for index in 0..keys.length() {
        let key = keys
            .get_index(try_catch, index)
            .to_exception_result(try_catch)?;

        let value = src.get(try_catch, key).to_exception_result(try_catch)?;

        dest.set(try_catch, key, value)
            .to_exception_result(try_catch)?;
    }

    Ok(())
}

pub type ExceptionResult<T> = std::result::Result<T, ExceptionError>;

#[derive(Debug)]
pub enum ExceptionError {
    NothingCaught,
    Caught(ExceptionErrorDetail),
}

#[derive(Debug, Default)]
pub struct ExceptionErrorDetail {
    exception: Option<String>,
    msg: Option<String>,
    resource_name: Option<String>,
    line_number: Option<usize>,
    stack_trace: Option<String>,
}

impl ExceptionError {
    fn capture<'s, 'p, P>(try_catch: &mut v8::TryCatch<'s, P>) -> Self
    where
        'p: 's,
        P: AsMut<v8::HandleScope<'p, ()>>,
        v8::TryCatch<'s, P>: AsMut<v8::HandleScope<'p, ()>>,
        v8::TryCatch<'s, P>: AsMut<v8::HandleScope<'p, v8::Context>>,
    {
        if !try_catch.has_caught() {
            return ExceptionError::NothingCaught;
        }

        let mut detail = ExceptionErrorDetail::default();
        if let Some(exc) = try_catch.exception() {
            let handle_scope: &mut v8::HandleScope<'p, v8::Context> = try_catch.as_mut();
            detail.exception = Some(format!(
                "exception: {}",
                exc.to_rust_string_lossy(handle_scope)
            ));
        }

        if let Some(message) = try_catch.message() {
            let handle_scope: &mut v8::HandleScope<'p, v8::Context> = try_catch.as_mut();
            detail.msg = Some(message.get(handle_scope).to_rust_string_lossy(handle_scope));
            detail.line_number = message.get_line_number(handle_scope);
            detail.resource_name = message
                .get_script_resource_name(handle_scope)
                .map(|v| v.to_rust_string_lossy(handle_scope));
        }

        Self::Caught(detail)
    }
}

impl std::fmt::Display for ExceptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExceptionError::NothingCaught => {
                writeln!(
                    f,
                    "expected JavaScript exception but none caught, this is likely a bug at the callsite"
                )
            }
            ExceptionError::Caught(detail) => {
                if let Some(exception) = &detail.exception {
                    writeln!(f, "JavaScript exception <{exception}>")?;
                } else {
                    writeln!(f, "JavaScript exception of unknown type")?;
                }

                if let Some(msg) = &detail.msg {
                    writeln!(f, "{msg}")?;
                }

                match (detail.line_number, &detail.resource_name) {
                    (Some(line_number), Some(resource_name)) => {
                        write!(f, "at {resource_name}:{line_number}")?;
                    }
                    (None, Some(resource_name)) => {
                        write!(f, "in {resource_name}")?;
                    }
                    (Some(line_number), None) => {
                        write!(f, "at {line_number}:?")?;
                    }
                    _ => {
                        write!(f, "at unknown location")?;
                    }
                }

                Ok(())
            }
        }
    }
}

impl std::error::Error for ExceptionError {}

pub trait CatchToResult<T> {
    /// Method to convert a value (typically an [Option<v8::Local>]) to an [ExceptionResult] by
    /// catching an exception with the [v8::TryCatch] when the value is [None]. This is appropriate
    /// to use whenever the C++ v8 API would return a `MaybeLocal` in its place, implying that an
    /// exception might have been raised.
    ///
    /// The [v8::TryCatch] given must have been used as the scope in the operation that produced
    /// the [Option].
    fn to_exception_result<'s, 'p, P>(
        self,
        try_catch: &mut v8::TryCatch<'s, P>,
    ) -> ExceptionResult<T>
    where
        'p: 's,
        P: AsMut<v8::HandleScope<'p, ()>>,
        v8::TryCatch<'s, P>: AsMut<v8::HandleScope<'p, ()>>,
        v8::TryCatch<'s, P>: AsMut<v8::HandleScope<'p, v8::Context>>;
}

impl<T> CatchToResult<T> for Option<T> {
    fn to_exception_result<'s, 'p, P>(
        self,
        try_catch: &mut v8::TryCatch<'s, P>,
    ) -> ExceptionResult<T>
    where
        'p: 's,
        P: AsMut<v8::HandleScope<'p, ()>>,
        v8::TryCatch<'s, P>: AsMut<v8::HandleScope<'p, ()>>,
        v8::TryCatch<'s, P>: AsMut<v8::HandleScope<'p, v8::Context>>,
    {
        match self {
            Some(v) => Ok(v),
            None => Err(ExceptionError::capture(try_catch)),
        }
    }
}
