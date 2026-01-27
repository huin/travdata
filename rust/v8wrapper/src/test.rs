use anyhow::{Context, Result, anyhow};
use googletest::prelude::*;
use map_macro::hashbrown::hash_map;

use super::*;

#[gtest]
fn test_thread_isolate_create_and_call_function() -> Result<()> {
    init_v8_for_testing();

    let tls_isolate = TlsIsolate::for_current_thread()?;

    let result: f64 = try_with_isolate(|tls_isolate| -> Result<f64> {
        v8::scope!(let scope, tls_isolate.isolate());
        let ctx = v8::Context::new(scope, v8::ContextOptions::default());
        v8::scope_with_context!(let scope, scope, ctx);

        let func_v8 = new_v8_function(
            scope,
            &["arg1"],
            &ESScriptOrigin::default(),
            r#"return arg1 + 2"#,
        )?;

        let global = ctx.global(scope);
        let arg1_v8 = v8::Number::new(scope, 3.0);

        let result_v8 = {
            v8::tc_scope!(let try_catch, scope);
            func_v8
                .call(try_catch, global.into(), &[arg1_v8.into()])
                .to_exception_result(try_catch)
                .context("calling function")?
        };

        let result: f64 = result_v8
            .number_value(scope)
            .ok_or_else(|| anyhow!("expected number, got {}", result_v8.type_repr()))
            .context("casting result to number")?;

        Ok(result)
    })??;

    assert_that!(result, approx_eq(5.0));

    drop(tls_isolate);

    Ok(())
}

#[gtest]
fn test_thread_isolate_create_store_and_later_use_function() -> Result<()> {
    init_v8_for_testing();

    let tls_isolate = TlsIsolate::for_current_thread()?;

    // Given two contexts.
    let (ctx_1, ctx_2) =
        try_with_isolate(|tls_isolate| (tls_isolate.new_ctx(), tls_isolate.new_ctx()))?;

    const FUNC_NAME: &str = "my_func";

    // Given a function is created on the first context's global.
    try_with_isolate(|tls_isolate| -> Result<()> {
        v8::scope!(let scope, tls_isolate.isolate());
        let ctx = v8::Local::new(scope, &ctx_1);
        v8::scope_with_context!(let scope, scope, ctx);

        let func_v8 = new_v8_function(
            scope,
            &["arg1"],
            &ESScriptOrigin::default(),
            r#"return arg1 + 2"#,
        )?;

        let global = scope.get_current_context().global(scope);
        let func_name_v8 =
            new_v8_string(scope, FUNC_NAME).context("creating function name string")?;

        {
            v8::tc_scope!(let try_catch, scope);
            global
                .set(try_catch, func_name_v8.into(), func_v8.into())
                .to_exception_result(try_catch)
                .context("setting function on global object")?;
        }

        Ok(())
    })??;

    // Then calling the function in the first context should work and return the expected answer.
    let result = try_with_isolate(|tls_isolate| -> Result<f64> {
        v8::scope!(let scope, tls_isolate.isolate());
        let ctx = v8::Local::new(scope, &ctx_1);
        v8::scope_with_context!(let scope, scope, ctx);

        let global = scope.get_current_context().global(scope);
        let func_name_v8 =
            new_v8_string(scope, FUNC_NAME).context("creating function name string")?;

        v8::tc_scope!(let try_catch, scope);
        let func_v8 = global
            .get(try_catch, func_name_v8.into())
            .context("getting function from global object")?
            .try_cast::<v8::Function>()
            .context("casting Value to Function")?;

        let arg1_v8 = v8::Number::new(try_catch, 3.0);
        let result_v8 = func_v8
            .call(try_catch, global.into(), &[arg1_v8.into()])
            .to_exception_result(try_catch)
            .context("calling function")?;

        let result: f64 = result_v8
            .number_value(try_catch)
            .to_exception_result(try_catch)
            .context("casting result to number")?;

        Ok(result)
    })??;
    assert_that!(result, approx_eq(5.0));

    // Then the function should not be present in the second context.
    let func_existed_on_other_context = try_with_isolate(|tls_isolate| -> Result<bool> {
        v8::scope!(let scope, tls_isolate.isolate());
        let ctx = v8::Local::new(scope, &ctx_2);
        v8::scope_with_context!(let scope, scope, ctx);

        let global = scope.get_current_context().global(scope);
        let func_name_v8 =
            new_v8_string(scope, FUNC_NAME).context("creating function name string")?;

        v8::tc_scope!(let try_catch, scope);
        let func_v8 = global
            .get(try_catch, func_name_v8.into())
            .context("getting function from global object")?;

        Ok(!func_v8.is_undefined())
    })??;
    assert_that!(func_existed_on_other_context, is_false());

    drop(tls_isolate);

    Ok(())
}

#[gtest]
fn test_set_and_get_values_in_separate_contexts() -> Result<()> {
    fn set_value(
        tls_isolate: &mut TlsIsolateGuard,
        ctx_global: &v8::Global<v8::Context>,
        value: &str,
    ) -> Result<()> {
        v8::scope!(let scope, tls_isolate.isolate());
        let ctx = v8::Local::new(scope, ctx_global);
        v8::scope_with_context!(let scope, scope, ctx);

        let key = new_v8_string(scope, "foo")?;
        let value = new_v8_string(scope, value)?;

        let global_obj = ctx.global(scope);

        v8::tc_scope!(let try_catch, scope);
        global_obj
            .set(try_catch, key.cast(), value.cast())
            .to_exception_result(try_catch)?;

        Ok(())
    }

    fn get_value(
        tls_isolate: &mut TlsIsolateGuard,
        ctx_global: &v8::Global<v8::Context>,
    ) -> Result<String> {
        v8::scope!(let scope, tls_isolate.isolate());
        let ctx = v8::Local::new(scope, ctx_global);
        v8::scope_with_context!(let scope, scope, ctx);

        let global_obj = ctx.global(scope);
        let key = new_v8_string(scope, "foo")?;

        v8::tc_scope!(let try_catch, scope);
        Ok(global_obj
            .get(try_catch, key.cast())
            .to_exception_result(try_catch)?
            .try_cast::<v8::String>()?
            .to_rust_string_lossy(try_catch))
    }

    const SET_VALUE_1: &str = "value in context 1";
    const SET_VALUE_2: &str = "value in context 2";

    init_v8_for_testing();

    let tls_isolate = TlsIsolate::for_current_thread()?;

    let (ctx_1_global, ctx_2_global) =
        try_with_isolate(|tls_isolate| (tls_isolate.new_ctx(), tls_isolate.new_ctx()))?;

    try_with_isolate(|tls_isolate| -> Result<()> {
        set_value(tls_isolate, &ctx_1_global, SET_VALUE_1)?;
        set_value(tls_isolate, &ctx_2_global, SET_VALUE_2)?;

        Ok(())
    })??;

    let (value_1, value_2) = try_with_isolate(|tls_isolate| -> Result<(String, String)> {
        let value_1 = get_value(tls_isolate, &ctx_1_global)?;
        let value_2 = get_value(tls_isolate, &ctx_2_global)?;

        Ok((value_1, value_2))
    })??;

    assert_that!(value_1, eq(SET_VALUE_1));
    assert_that!(value_2, eq(SET_VALUE_2));

    drop(tls_isolate);

    Ok(())
}

#[gtest]
fn test_modules() -> anyhow::Result<()> {
    init_v8_for_testing();

    let tls_isolate = TlsIsolate::for_current_thread()?;

    try_with_isolate(|tls_isolate| -> Result<()> {
        let ctx_global = tls_isolate.new_ctx();

        v8::scope!(let scope, tls_isolate.isolate());
        let ctx = v8::Local::new(scope, ctx_global);
        v8::scope_with_context!(let scope, scope, ctx);

        let modules = modules::ModuleDefs::new(hash_map! {
            "./barmodule.js".into() => modules::ModuleDef{
                src: r#"
                 export function bar() {
                     return "bar";
                 }
                 "#.into(),
                origin: ESScriptOrigin {
                    resource_name: "barmodule.js".into(),
                    is_module: true,
                    ..Default::default()
                },
            },
        });
        modules.install_into_context(ctx);

        let top_level_src = new_v8_string(
            scope,
            r#"
             import {bar} from './barmodule.js';

             export function foo() {
                 return "foo " + bar();
             }
             "#,
        )?;

        let top_level_origin: v8::ScriptOrigin = ESScriptOrigin {
            resource_name: "top_level.js".into(),
            is_module: true,
            ..Default::default()
        }
        .try_make_origin(scope)?;

        let module = {
            v8::tc_scope!(let try_catch, scope);

            let mut source =
                v8::script_compiler::Source::new(top_level_src, Some(&top_level_origin));
            let module = v8::script_compiler::compile_module(try_catch, &mut source)
                .to_exception_result(try_catch)
                .context("compiling module")?;

            module
                .instantiate_module(try_catch, modules::ModuleDefs::resolver_callback)
                .to_exception_result(try_catch)
                .context("instantiating module")?;

            module
                .evaluate(try_catch)
                .to_exception_result(try_catch)
                .context("evaluating module")?;

            expect_that!(module.is_graph_async(), is_false());

            module
        };

        let module_namespace = module.get_module_namespace();
        let module_object = module_namespace.try_cast::<v8::Object>()?;
        let global = ctx.global(scope);
        shallow_copy_object_properties(scope, module_object, global)?;

        let origin = ESScriptOrigin {
            resource_name: "function.js".into(),
            is_module: true,
            ..Default::default()
        };

        let func_v8 = new_v8_function(
            scope,
            &[],
            &origin,
            r#"
             return foo();
             "#,
        )?;

        let result_v8 = {
            v8::tc_scope!(let try_catch, scope);
            func_v8
                .call(try_catch, module_namespace, &[])
                .to_exception_result(try_catch)
                .context("calling function")
        };

        expect_that!(
            result_v8.map(|result_v8| result_v8.to_rust_string_lossy(scope)),
            ok(eq("foo bar"))
        );

        Ok(())
    })??;

    drop(tls_isolate);

    Ok(())
}
