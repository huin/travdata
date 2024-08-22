use std::sync::mpsc;
use std::sync::OnceLock;

use anyhow::Context;
use anyhow::Error;
use anyhow::{anyhow, bail, Result};

use crate::table::Table;

static INIT_V8: OnceLock<()> = OnceLock::new();

fn init_v8() {
    INIT_V8.get_or_init(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

/// Definition of an ECMAScript script.
#[derive(Debug)]
pub struct ESScript {
    pub source: String,
    pub origin: ESScriptOrigin,
}

/// Origin of an [ESScript].
#[derive(Debug, Default)]
pub struct ESScriptOrigin {
    pub resource_name: String,
    pub resource_line_offset: i32,
    pub resource_column_offset: i32,
    pub script_id: i32,
}

impl ESScriptOrigin {
    fn make_origin<'s>(&self, try_catch: &mut TryScope<'_, 's>) -> Result<v8::ScriptOrigin<'s>> {
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

type CtxScope<'s, 'p> = v8::ContextScope<'s, v8::HandleScope<'p>>;
type TryScope<'s, 'p> = v8::TryCatch<'s, v8::HandleScope<'p>>;

pub struct ESTransformer {
    // The declaration order of request_send and thread_join is such that they will be dropped in
    // an order that shuts down correctly. Dropping `request_send` implicitly requests that the
    // thread stops waiting for more requests.
    request_send: mpsc::Sender<Request>,
    // `thread_join` is kept so that the `ESTransformer` implementation of `Drop` blocks until the
    // thread shuts down.
    #[allow(dead_code)]
    thread_join: std::thread::JoinHandle<()>,
}

impl ESTransformer {
    pub fn new() -> Self {
        init_v8();

        let (request_send, request_recv) = mpsc::channel();
        let thread_join = std::thread::spawn(move || Self::isolate_thread(request_recv));

        Self {
            request_send,
            thread_join,
        }
    }

    pub fn run_script(&mut self, script: ESScript) -> Result<()> {
        let (result_send, result_recv) = mpsc::channel();
        self.request_send.send(Request::RunScript {
            script,
            result_send,
        })?;
        match result_recv.recv() {
            Ok(result) => result,
            Err(recv_err) => bail!("failure to receive result from V8 thread: {}", recv_err),
        }
    }

    pub fn transform(&self, func: TransformFn, tables: Vec<Table>) -> Result<Table> {
        let (result_send, result_recv) = mpsc::channel();
        self.request_send.send(Request::Transform {
            func,
            tables,
            result_send,
        })?;
        match result_recv.recv() {
            Ok(result) => result,
            Err(recv_err) => bail!("failure to receive result from V8 thread: {}", recv_err),
        }
    }

    fn isolate_thread(request_recv: mpsc::Receiver<Request>) {
        let mut isolate = v8::Isolate::new(Default::default());
        let outer_scope = &mut v8::HandleScope::new(&mut isolate);
        let ctx = v8::Context::new(outer_scope, v8::ContextOptions::default());
        let global = ctx.global(outer_scope);

        while let Ok(request) = request_recv.recv() {
            let mut ctx_scope = v8::ContextScope::new(outer_scope, ctx);
            match request {
                Request::RunScript {
                    script,
                    result_send,
                } => {
                    let result: Result<()> = Self::handle_run_script(&mut ctx_scope, script);
                    if result_send.send(result).is_err() {
                        // TODO: log this error?
                    }
                }
                Request::Transform {
                    func,
                    tables,
                    result_send,
                } => {
                    let result = Self::handle_transform(global, &mut ctx_scope, func, tables);
                    if result_send.send(result).is_err() {
                        // TODO: log this error?
                    }
                }
            }
        }
    }

    fn handle_run_script(ctx_scope: &mut CtxScope<'_, '_>, script: ESScript) -> Result<()> {
        let mut try_catch = v8::TryCatch::new(ctx_scope);

        let origin_v8 = script.origin.make_origin(&mut try_catch)?;
        let source_str_v8 = new_v8_string(&mut try_catch, &script.source)
            .with_context(|| "could not create source code string")?;
        let script_v8 = v8::Script::compile(&mut try_catch, source_str_v8, Some(&origin_v8))
            .ok_or_else(|| try_catch_to_result(&mut try_catch))
            .with_context(|| "could not compile script")?;

        let result = script_v8
            .run(&mut try_catch)
            .ok_or_else(|| try_catch_to_result(&mut try_catch))
            .with_context(|| "could not run script")?;

        if !result.is_undefined() {
            bail!(
                "running script resulted in a value other than undefined, got value of type {}",
                result
                    .type_of(&mut try_catch)
                    .to_rust_string_lossy(&mut try_catch)
            );
        }

        Ok(())
    }

    fn handle_transform(
        global: v8::Local<'_, v8::Object>,
        ctx_scope: &mut CtxScope<'_, '_>,
        func: TransformFn,
        tables: Vec<Table>,
    ) -> Result<Table> {
        let mut try_catch = v8::TryCatch::new(ctx_scope);

        let origin_v8 = func.origin.make_origin(&mut try_catch)?;
        let body_str_v8 = new_v8_string(&mut try_catch, &func.function_body)
            .with_context(|| "could not create function body string")?;
        let mut body_v8 = v8::script_compiler::Source::new(body_str_v8, Some(&origin_v8));

        let arg_name_v8 = new_v8_string(&mut try_catch, "tables")
            .with_context(|| "could not create argument name")?;
        let function_v8 = v8::script_compiler::compile_function(
            &mut try_catch,
            &mut body_v8,
            &[arg_name_v8],
            &[],
            v8::script_compiler::CompileOptions::NoCompileOptions,
            v8::script_compiler::NoCacheReason::NoReason,
        )
        .ok_or_else(|| try_catch_to_result(&mut try_catch))
        .with_context(|| "could not compile transform function")?;

        let in_tables_v8 = serde_v8::to_v8(&mut try_catch, &tables)?;

        let result_v8: v8::Local<'_, v8::Value> = function_v8
            .call(&mut try_catch, global.into(), &[in_tables_v8])
            .ok_or_else(|| try_catch_to_result(&mut try_catch))?;

        let out_table_v8: Table = serde_v8::from_v8(&mut try_catch, result_v8)?;

        Ok(out_table_v8)
    }
}

enum Request {
    RunScript {
        script: ESScript,
        result_send: mpsc::Sender<Result<()>>,
    },
    Transform {
        func: TransformFn,
        tables: Vec<Table>,
        result_send: mpsc::Sender<Result<Table>>,
    },
}

pub struct TransformFn {
    pub function_body: String,
    pub origin: ESScriptOrigin,
}

/// Always returns an `Err`, but will use information from the given [v8::TryCatch] if a message is
/// present.
fn try_catch_to_result(try_catch: &mut TryScope<'_, '_>) -> Error {
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

fn new_v8_string<'s>(
    try_catch: &mut TryScope<'_, 's>,
    string: &str,
) -> Result<v8::Local<'s, v8::String>> {
    v8::String::new(try_catch, string).ok_or_else(|| try_catch_to_result(try_catch))
}

#[cfg(test)]
mod test {
    use googletest::{
        assert_that,
        matchers::{anything, err, ok},
        prelude::eq,
    };

    use crate::extraction::tableextract::estransform::{ESScriptOrigin, TransformFn};
    use crate::table::{Row, Table};

    use super::{ESScript, ESTransformer};

    fn new_script(source: impl Into<String>) -> ESScript {
        ESScript {
            source: source.into(),
            origin: new_origin(),
        }
    }

    fn new_origin() -> ESScriptOrigin {
        ESScriptOrigin {
            resource_name: "script.js".to_string(),
            resource_line_offset: 0,
            resource_column_offset: 0,
            script_id: 1,
        }
    }

    #[test]
    fn test_run_script() {
        let mut estrn = ESTransformer::new();
        let result = estrn.run_script(new_script("const foo = {};"));
        assert_that!(result, ok(eq(())));
    }

    #[test]
    fn test_run_script_syntax_error() {
        let mut estrn = ESTransformer::new();
        let result = estrn.run_script(new_script("I'm invalid ECMAScript!"));
        assert_that!(result, err(anything()));
    }

    const CONCAT_TABLE_DATA_FN: &str = r#"
        function concatTableData(tables) {
            const result = [];
            for (const table of tables) {
                result.splice(result.length, 0, ...table);
            }
            return result;
        };
        "#;

    #[test]
    fn test_transform_table_with_function_defined_in_script() {
        let mut estrn = ESTransformer::new();

        let result = estrn.run_script(new_script(CONCAT_TABLE_DATA_FN));
        assert_that!(result, ok(anything()));

        let result = estrn.transform(
            TransformFn {
                function_body: r#"return concatTableData(tables);"#.to_string(),
                origin: new_origin(),
            },
            vec![
                Table(vec![Row(vec!["t1r1c1".to_string(), "t1r1c2".to_string()])]),
                Table(vec![Row(vec!["t2r1c1".to_string(), "t2r1c2".to_string()])]),
            ],
        );
        assert_that!(
            result,
            ok(eq(Table(vec![
                Row(vec!["t1r1c1".to_string(), "t1r1c2".to_string()]),
                Row(vec!["t2r1c1".to_string(), "t2r1c2".to_string()]),
            ])))
        );
    }
}
