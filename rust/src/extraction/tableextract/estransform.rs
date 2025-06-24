use anyhow::{Context, Result, bail};

use crate::table::Table;
use crate::v8wrapper;

/// Definition of an ECMAScript script.
#[derive(Debug)]
pub struct ESScript {
    pub source: String,
    pub origin: v8wrapper::ESScriptOrigin,
}

/// Manages an ECMAScript context in which table transformations are performed.
pub struct ESTransformer {
    ctx_client: v8wrapper::ContextClient,
}

impl ESTransformer {
    /// Creates a new [ESTransformer] that uses the given context.
    pub fn new(ctx_client: v8wrapper::ContextClient) -> Self {
        Self { ctx_client }
    }

    /// Runs the given script within the context. Useful to define common code to be used by
    /// transforms.
    pub fn run_script(&mut self, script: ESScript) -> Result<()> {
        self.ctx_client.run(move |try_catch| {
            let origin_v8 = script.origin.make_origin(try_catch)?;
            let source_str_v8 = v8wrapper::new_v8_string(try_catch, &script.source)
                .with_context(|| "could not create source code string")?;
            let script_v8 = v8::Script::compile(try_catch, source_str_v8, Some(&origin_v8))
                .ok_or_else(|| v8wrapper::try_catch_to_result(try_catch))
                .with_context(|| "could not compile script")?;

            let result = script_v8
                .run(try_catch)
                .ok_or_else(|| v8wrapper::try_catch_to_result(try_catch))
                .with_context(|| "could not run script")?;

            if !result.is_undefined() {
                bail!(
                    "running script resulted in a value other than undefined, got value of type {}",
                    result.type_of(try_catch).to_rust_string_lossy(try_catch)
                );
            }

            Ok(())
        })
    }

    /// Performs a table transformation.
    pub fn transform(&self, func: TransformFn, tables: Vec<Table>) -> Result<Table> {
        self.ctx_client.run(move |try_catch| {
            let origin_v8 = func.origin.make_origin(try_catch)?;
            let body_str_v8 = v8wrapper::new_v8_string(try_catch, &func.function_body)
                .with_context(|| "could not create function body string")?;
            let mut body_v8 = v8::script_compiler::Source::new(body_str_v8, Some(&origin_v8));

            let arg_name_v8 = v8wrapper::new_v8_string(try_catch, "tables")
                .with_context(|| "could not create argument name")?;
            let function_v8 = v8::script_compiler::compile_function(
                try_catch,
                &mut body_v8,
                &[arg_name_v8],
                &[],
                v8::script_compiler::CompileOptions::NoCompileOptions,
                v8::script_compiler::NoCacheReason::NoReason,
            )
            .ok_or_else(|| v8wrapper::try_catch_to_result(try_catch))
            .with_context(|| "could not compile transform function")?;

            let in_tables_v8: v8::Local<'_, v8::Value> =
                serde_v8::to_v8(try_catch.as_mut(), &tables)?;

            let global = try_catch.get_current_context().global(try_catch);
            let result_v8: v8::Local<'_, v8::Value> = function_v8
                .call(try_catch, global.into(), &[in_tables_v8])
                .ok_or_else(|| v8wrapper::try_catch_to_result(try_catch))?;

            let out_table_v8: Table = serde_v8::from_v8(try_catch.as_mut(), result_v8)?;

            Ok(out_table_v8)
        })
    }
}

pub struct TransformFn {
    pub function_body: String,
    pub origin: v8wrapper::ESScriptOrigin,
}

#[cfg(test)]
mod test {
    use googletest::prelude::*;

    use crate::{
        extraction::tableextract::estransform::TransformFn,
        v8wrapper::{self, IsolateThreadHandleForTest},
    };
    use crate::{
        table::{Row, Table},
        testutil::WrapError,
    };

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
            resource_line_offset: 0,
            resource_column_offset: 0,
            script_id: -1,
        }
    }

    #[gtest]
    fn test_run_script(handle: &&IsolateThreadHandleForTest) -> googletest::Result<()> {
        let ctx_client = handle.new_context().wrap_error()?;
        let mut estrn = ESTransformer::new(ctx_client);

        let result = estrn.run_script(new_script("const foo = {};"));
        assert_that!(result, ok(eq(&())));

        Ok(())
    }

    #[gtest]
    fn test_run_script_syntax_error(
        handle: &&IsolateThreadHandleForTest,
    ) -> googletest::Result<()> {
        let ctx_client = handle.new_context().wrap_error()?;
        let mut estrn = ESTransformer::new(ctx_client);

        let result = estrn.run_script(new_script("I'm invalid ECMAScript!"));
        assert_that!(result, err(anything()));

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
    fn test_transform_table_with_function_defined_in_script(
        handle: &&IsolateThreadHandleForTest,
    ) -> googletest::Result<()> {
        let ctx_client = handle.new_context().wrap_error()?;
        let mut estrn = ESTransformer::new(ctx_client);

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

        Ok(())
    }
}
