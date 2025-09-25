use anyhow::{Context, Result, bail};

use v8wrapper::CatchToResult;

use crate::table::Table;

/// Definition of an ECMAScript script.
#[derive(Debug)]
pub struct ESScript {
    pub source: String,
    pub origin: v8wrapper::ESScriptOrigin,
}

/// Manages an ECMAScript context in which table transformations are performed.
pub struct ESTransformer {
    ctx: v8::Global<v8::Context>,
}

impl ESTransformer {
    /// Creates a new [ESTransformer] that uses the given context.
    pub fn new() -> Result<Self> {
        let ctx = v8wrapper::try_with_isolate(|tls_isolate| tls_isolate.new_ctx())?;
        Ok(Self { ctx })
    }

    /// Runs the given script within the context. Useful to define common code to be used by
    /// transforms.
    pub fn run_script(&mut self, script: ESScript) -> Result<()> {
        let result = v8wrapper::try_with_isolate(|tls_isolate| {
            let scope = &mut tls_isolate.scope();
            let ctx = v8::Local::new(scope, &self.ctx);
            let scope = &mut v8::ContextScope::new(scope, ctx);

            let origin_v8 = script.origin.try_make_origin(scope)?;
            let source_str_v8 = v8wrapper::new_v8_string(scope, &script.source)
                .with_context(|| "could not create source code string")?;

            let result = {
                let try_catch = &mut v8::TryCatch::new(scope);
                let script_v8 = v8::Script::compile(try_catch, source_str_v8, Some(&origin_v8))
                    .to_exception_result(try_catch)
                    .with_context(|| "could not compile script")?;

                script_v8
                    .run(try_catch)
                    .to_exception_result(try_catch)
                    .with_context(|| "could not run script")?
            };

            if !result.is_undefined() {
                bail!(
                    "running script resulted in a value other than undefined, got value of type {}",
                    result.type_of(scope).to_rust_string_lossy(scope)
                );
            }

            Ok(())
        })?;

        result
    }

    /// Performs a table transformation.
    pub fn transform(&self, func: TransformFn, tables: Vec<Table>) -> Result<Table> {
        let result = v8wrapper::try_with_isolate(|tls_isolate| {
            let scope = &mut tls_isolate.scope();
            let ctx = v8::Local::new(scope, &self.ctx);
            let scope = &mut v8::ContextScope::new(scope, ctx);

            let origin_v8 = func.origin.try_make_origin(scope)?;
            let body_str_v8 = v8wrapper::new_v8_string(scope, &func.function_body)
                .with_context(|| "could not create function body string")?;
            let mut body_v8 = v8::script_compiler::Source::new(body_str_v8, Some(&origin_v8));

            let arg_name_v8 = v8wrapper::new_v8_string(scope, "tables")
                .with_context(|| "could not create argument name")?;

            let result_v8: v8::Local<'_, v8::Value> = {
                let try_catch = &mut v8::TryCatch::new(scope);
                let function_v8 = v8::script_compiler::compile_function(
                    try_catch,
                    &mut body_v8,
                    &[arg_name_v8],
                    &[],
                    v8::script_compiler::CompileOptions::NoCompileOptions,
                    v8::script_compiler::NoCacheReason::NoReason,
                )
                .to_exception_result(try_catch)
                .with_context(|| "could not compile transform function")?;

                let in_tables_v8: v8::Local<'_, v8::Value> =
                    serde_v8::to_v8(try_catch.as_mut(), &tables)?;

                let global = try_catch.get_current_context().global(try_catch);
                function_v8
                    .call(try_catch, global.into(), &[in_tables_v8])
                    .to_exception_result(try_catch)?
            };

            let out_table_v8: Table = serde_v8::from_v8(scope, result_v8)?;

            Ok(out_table_v8)
        })?;

        result
    }
}

pub struct TransformFn {
    pub function_body: String,
    pub origin: v8wrapper::ESScriptOrigin,
}

#[cfg(test)]
mod test {
    use googletest::prelude::*;

    use crate::extraction::tableextract::estransform::TransformFn;
    use crate::table::{Row, Table};

    use super::{ESScript, ESTransformer};

    fn new_script(source: impl Into<String>) -> ESScript {
        ESScript {
            source: source.into(),
            origin: new_origin(),
        }
    }

    fn new_origin() -> v8wrapper::ESScriptOrigin {
        v8wrapper::ESScriptOrigin {
            resource_name: "script.js".to_string(),
            ..Default::default()
        }
    }

    #[gtest]
    fn test_run_script() -> anyhow::Result<()> {
        v8wrapper::init_v8_for_testing();
        let tls_isolate = v8wrapper::TlsIsolate::for_current_thread()?;

        let mut estrn = ESTransformer::new()?;
        let result = estrn.run_script(new_script("const foo = {};"));

        assert_that!(result, ok(eq(&())));

        drop(tls_isolate);
        Ok(())
    }

    #[gtest]
    fn test_run_script_syntax_error() -> anyhow::Result<()> {
        v8wrapper::init_v8_for_testing();
        let tls_isolate = v8wrapper::TlsIsolate::for_current_thread()?;

        let mut estrn = ESTransformer::new()?;
        let result = estrn.run_script(new_script("I'm invalid ECMAScript!"));

        assert_that!(result, err(anything()));

        drop(tls_isolate);
        Ok(())
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

    #[gtest]
    fn test_transform_table_with_function_defined_in_script() -> anyhow::Result<()> {
        v8wrapper::init_v8_for_testing();
        let tls_isolate = v8wrapper::TlsIsolate::for_current_thread()?;

        let mut estrn = ESTransformer::new()?;

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
            ok(eq(&Table(vec![
                Row(vec!["t1r1c1".to_string(), "t1r1c2".to_string()]),
                Row(vec!["t2r1c1".to_string(), "t2r1c2".to_string()]),
            ])))
        );

        drop(tls_isolate);
        Ok(())
    }
}
