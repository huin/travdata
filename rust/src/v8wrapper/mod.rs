#![allow(dead_code)]

#[cfg(test)]
mod test;

use std::{any::Any, collections::HashMap, sync::mpsc};

use anyhow::{Context, Error, Result, anyhow};

use crate::mpscutil;

static INIT_V8: std::sync::OnceLock<()> = std::sync::OnceLock::new();

fn init_v8() {
    INIT_V8.get_or_init(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

pub type TryScope<'s, 'p> = v8::TryCatch<'s, v8::HandleScope<'p>>;

type RunResult = Result<Box<dyn Any + Send + 'static>>;

trait RunFn: FnOnce(&mut TryScope<'_, '_>) -> RunResult + Send {}
impl<F> RunFn for F where F: FnOnce(&mut TryScope<'_, '_>) -> RunResult + Send {}

type BoxedRunFn = Box<dyn RunFn>;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ContextKey(u32);

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

pub struct IsolateThreadHandle {
    // The declaration order of request_send and thread_join is such that they will be dropped in
    // an order that shuts down correctly. Dropping `request_send` implicitly requests that the
    // thread stops waiting for more requests.
    request_send: mpsc::SyncSender<Request>,
    // `thread_join` is kept so that `Drop` blocks until the thread shuts down.
    #[allow(dead_code)]
    thread_join: std::thread::JoinHandle<()>,
}

impl IsolateThreadHandle {
    pub fn new() -> Self {
        let (request_send, request_recv) = mpsc::sync_channel(0);

        let thread_join = std::thread::spawn(move || run_isolate_thread(request_recv));

        Self {
            request_send,
            thread_join,
        }
    }

    pub fn create_client(&self) -> IsolateThreadClient {
        IsolateThreadClient {
            request_send: self.request_send.clone(),
        }
    }
}

// TODO: refactor client to be an `Rc` that takes down the context with it. ContextKey can then be
// internal to this package.
pub struct IsolateThreadClient {
    // The declaration order of request_send and thread_join is such that they will be dropped in
    // an order that shuts down correctly. Dropping `request_send` implicitly requests that the
    // thread stops waiting for more requests.
    request_send: mpsc::SyncSender<Request>,
}

impl IsolateThreadClient {
    /// Creates a new [v8::Context] and returns a reference to it.
    pub fn new_context(&self) -> Result<ContextKey> {
        let (result_send, result_recv) = mpsc::sync_channel(0);
        self.request_send
            .send(Request::NewContext(result_send))
            .map_err(|err| anyhow!("{}", err))
            .context("sending request to IsolateThread")?;
        result_recv
            .recv()
            .context("receiving result from IsolateThread")?
    }

    /// Deletes the [v8::Context] referenced by `ctx_key`.
    pub fn drop_context(&self, ctx_key: ContextKey) -> Result<()> {
        self.request_send
            .send(Request::DropContext(ctx_key))
            .map_err(|err| anyhow!("{}", err))
            .context("sending request to IsolateThread")?;
        Ok(())
    }

    /// Runs the closure `f` against the [v8::Context] identified by `ctx_key`.
    pub fn run<F, T>(&self, ctx_key: &ContextKey, f: F) -> Result<T>
    where
        F: FnOnce(&mut TryScope) -> Result<T> + Send + 'static,
        T: Any + Send + 'static,
    {
        let (result_send, result_recv) = mpsc::sync_channel(0);

        self.request_send
            .send(Request::Run {
                ctx_key: ctx_key.clone(),
                func: Box::new(|context: &mut TryScope| -> RunResult {
                    let value = f(context)?;
                    Ok(Box::new(value))
                }),
                result_send,
            })
            .map_err(|err| anyhow!("{}", err))
            .context("sending request to IsolateThread")?;

        result_recv
            .recv()
            .context("receiving result from IsolateThread")?
            .and_then(|value| {
                value.downcast::<T>().map_err(|err| {
                    anyhow!(
                        "received wrong type in result from IsolateThread (got value {:?})",
                        err,
                    )
                })
            })
            .map(|value| *value)
    }
}

/// Entry point that creates an internal [IsolateThread] and dispatches requests to it.
///
/// This method blocks until the given `request_recv` is closed.
fn run_isolate_thread(request_recv: mpsc::Receiver<Request>) {
    init_v8();

    let mut isolate = v8::Isolate::new(v8::CreateParams::default());
    let mut scope = v8::HandleScope::new(&mut isolate);

    let mut processor = IsolateThread::new(&mut scope);
    processor.perform_requests(request_recv);
}

struct IsolateThread<'s, 'i> {
    isolate_scope: &'i mut v8::HandleScope<'s, ()>,
    contexts: HashMap<ContextKey, v8::Local<'s, v8::Context>>,
    next_ctx_key: ContextKey,
    free_ctx_keys: Vec<ContextKey>,
}

impl<'s, 'i> IsolateThread<'s, 'i>
where
    's: 'i,
{
    fn new(isolate_scope: &'i mut v8::HandleScope<'s, ()>) -> Self {
        Self {
            isolate_scope,
            contexts: HashMap::new(),
            next_ctx_key: ContextKey(0),
            free_ctx_keys: Vec::new(),
        }
    }

    fn perform_requests(&mut self, request_recv: mpsc::Receiver<Request>) {
        while let Ok(request) = request_recv.recv() {
            match request {
                Request::NewContext(result_send) => {
                    let result = self.handle_new_context();
                    mpscutil::send_or_log_warning(
                        &result_send,
                        "result of call to IsolateThread::handle_new_context",
                        result,
                    );
                }
                Request::DropContext(ctx_key) => {
                    self.handle_drop_context(ctx_key);
                }
                Request::Run {
                    ctx_key,
                    func,
                    result_send,
                } => {
                    let result = self.handle_run(ctx_key, func);
                    mpscutil::send_or_log_warning(
                        &result_send,
                        "result of call to IsolateThread::handle_run",
                        result,
                    );
                }
            }
        }
    }

    fn handle_new_context(&mut self) -> Result<ContextKey> {
        let ctx_key = self
            .new_ctx_key()
            .ok_or_else(|| anyhow!("run out of context keys"))?;

        let context = v8::Context::new(self.isolate_scope, v8::ContextOptions::default());
        self.contexts.insert(ctx_key.clone(), context);
        Ok(ctx_key)
    }

    fn handle_drop_context(&mut self, ctx_key: ContextKey) {
        if self.contexts.remove(&ctx_key).is_some() {
            self.recycle_ctx_key(ctx_key);
        }
    }

    fn handle_run(&mut self, ctx_key: ContextKey, func: BoxedRunFn) -> Result<Box<dyn Any + Send>> {
        let context = self
            .contexts
            .get(&ctx_key)
            .ok_or_else(|| anyhow!("invalid context key {:?}", ctx_key))?;
        let context_scope = &mut v8::ContextScope::new(self.isolate_scope, *context);
        let handle_scope = &mut v8::HandleScope::new(context_scope);
        let try_catch = &mut v8::TryCatch::new(handle_scope);

        func(try_catch)
    }

    fn new_ctx_key(&mut self) -> Option<ContextKey> {
        if let Some(ctx_key) = self.free_ctx_keys.pop() {
            Some(ctx_key)
        } else {
            let ctx_key_value = self.next_ctx_key.clone();
            self.next_ctx_key = self.next_ctx_key.0.checked_add(1).map(ContextKey)?;
            Some(ctx_key_value)
        }
    }

    fn recycle_ctx_key(&mut self, ctx_key: ContextKey) {
        self.free_ctx_keys.push(ctx_key);
    }
}

enum Request {
    NewContext(mpsc::SyncSender<Result<ContextKey>>),
    DropContext(ContextKey),
    Run {
        ctx_key: ContextKey,
        func: BoxedRunFn,
        result_send: mpsc::SyncSender<RunResult>,
    },
}

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

pub fn new_v8_string<'s>(
    try_catch: &mut TryScope<'_, 's>,
    string: &str,
) -> Result<v8::Local<'s, v8::String>> {
    v8::String::new(try_catch, string).ok_or_else(|| try_catch_to_result(try_catch))
}

pub fn new_v8_number<'s>(
    try_catch: &mut TryScope<'_, 's>,
    number: f64,
) -> v8::Local<'s, v8::Number> {
    v8::Number::new(try_catch, number)
}

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

/// Provides a shared [IsolateThreadHandle] for tests. At the time of writing [v8] only supports
/// creating one [v8::Isolate] per process (even after removing the first).
#[cfg(test)]
pub struct IsolateThreadHandleForTest {
    handle: IsolateThreadHandle,
}

// Unclear if this is safe to implement, but it's for tests only.
#[cfg(test)]
impl std::panic::RefUnwindSafe for IsolateThreadHandleForTest {}

#[cfg(test)]
impl std::ops::Deref for IsolateThreadHandleForTest {
    type Target = IsolateThreadHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

#[cfg(test)]
impl googletest::fixtures::StaticFixture for IsolateThreadHandleForTest {
    fn set_up_once() -> googletest::Result<Self> {
        Ok(Self {
            handle: IsolateThreadHandle::new(),
        })
    }
}

/// Should only be implemented for types that will safely cast from [v8::Local<'s, T>] to
/// [v8::Local<'s, v8::Value>] without panicing.
pub trait SafeCastV8<'s> {
    fn safe_cast(self) -> v8::Local<'s, v8::Value>;
}

impl<'s> SafeCastV8<'s> for v8::Local<'s, v8::Function> {
    fn safe_cast(self) -> v8::Local<'s, v8::Value> {
        self.cast()
    }
}

impl<'s> SafeCastV8<'s> for v8::Local<'s, v8::Number> {
    fn safe_cast(self) -> v8::Local<'s, v8::Value> {
        self.cast()
    }
}

impl<'s> SafeCastV8<'s> for v8::Local<'s, v8::Object> {
    fn safe_cast(self) -> v8::Local<'s, v8::Value> {
        self.cast()
    }
}

impl<'s> SafeCastV8<'s> for v8::Local<'s, v8::String> {
    fn safe_cast(self) -> v8::Local<'s, v8::Value> {
        self.cast()
    }
}
