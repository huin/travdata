use anyhow::Result;
use googletest::prelude::*;
use testutils::WrapError;

use super::*;

#[gtest]
fn test_thread_isolate_create_and_call_function() -> googletest::Result<()> {
    init_v8_for_testing();

    let tls_isolate = TlsIsolate::for_current_thread().wrap_error()?;

    let result: f64 = try_with_isolate(|tls_isolate| -> Result<f64> {
        let mut scope = tls_isolate.scope();
        let ctx = v8::Context::new(&mut scope, v8::ContextOptions::default());

        let mut ctx_scope = v8::ContextScope::new(&mut scope, ctx);
        let try_catch = &mut v8::TryCatch::new(&mut ctx_scope);

        let func_v8 = new_v8_function(
            try_catch,
            &["arg1"],
            &ESScriptOrigin::default(),
            r#"return arg1 + 2"#,
        )?;

        let global = ctx.global(try_catch);
        let arg1_v8 = new_v8_number(try_catch, 3.0);
        let result_v8 = func_v8
            .call(try_catch, global.into(), &[arg1_v8.into()])
            .ok_or_else(|| try_catch_to_result(try_catch))
            .context("calling function")?;

        let result: f64 = result_v8
            .number_value(try_catch)
            .ok_or_else(|| anyhow!("expected number, got {}", result_v8.type_repr()))
            .context("casting result to number")?;

        Ok(result)
    })?
    .wrap_error()?;

    assert_that!(result, approx_eq(5.0));

    drop(tls_isolate);

    Ok(())
}

#[gtest]
fn test_thread_isolate_create_store_and_later_use_function() -> googletest::Result<()> {
    init_v8_for_testing();

    let tls_isolate = TlsIsolate::for_current_thread().wrap_error()?;

    // Given two contexts.
    let (ctx_1, ctx_2) =
        try_with_isolate(|tls_isolate| (tls_isolate.new_ctx(), tls_isolate.new_ctx()))?;

    const FUNC_NAME: &str = "my_func";

    // Given a function is created on the first context's global.
    try_with_isolate(|tls_isolate| {
        let mut scope = tls_isolate.scope();
        let ctx = v8::Local::new(&mut scope, &ctx_1);
        let mut ctx_scope = v8::ContextScope::new(&mut scope, ctx);
        let try_catch = &mut v8::TryCatch::new(&mut ctx_scope);

        let func_v8 = new_v8_function(
            try_catch,
            &["arg1"],
            &ESScriptOrigin::default(),
            r#"return arg1 + 2"#,
        )?;

        let global = try_catch.get_current_context().global(try_catch);
        let func_name_v8 =
            new_v8_string(try_catch, FUNC_NAME).context("creating function name string")?;

        global
            .set(try_catch, func_name_v8.into(), func_v8.into())
            .context("setting function on global object")?;

        Ok(())
    })?
    .wrap_error()?;

    // Then calling the function in the first context should work and return the expected answer.
    let result = try_with_isolate(|tls_isolate| -> Result<f64> {
        let mut scope = tls_isolate.scope();
        let ctx = v8::Local::new(&mut scope, &ctx_1);
        let mut ctx_scope = v8::ContextScope::new(&mut scope, ctx);
        let try_catch = &mut v8::TryCatch::new(&mut ctx_scope);

        let global = try_catch.get_current_context().global(try_catch);
        let func_name_v8 =
            new_v8_string(try_catch, FUNC_NAME).context("creating function name string")?;

        let func_v8 = global
            .get(try_catch, func_name_v8.into())
            .context("getting function from global object")?
            .try_cast::<v8::Function>()
            .context("casting Value to Function")?;

        let arg1_v8 = new_v8_number(try_catch, 3.0);
        let result_v8 = func_v8
            .call(try_catch, global.into(), &[arg1_v8.into()])
            .ok_or_else(|| try_catch_to_result(try_catch))
            .context("calling function")?;

        let result: f64 = result_v8
            .number_value(try_catch)
            .ok_or_else(|| anyhow!("expected number, got {}", result_v8.type_repr()))
            .context("casting result to number")?;

        Ok(result)
    })?
    .wrap_error()?;
    assert_that!(result, approx_eq(5.0));

    // Then the function should not be present in the second context.
    let func_existed_on_other_context = try_with_isolate(|tls_isolate| -> Result<bool> {
        let mut scope = tls_isolate.scope();
        let ctx = v8::Local::new(&mut scope, &ctx_2);
        let mut ctx_scope = v8::ContextScope::new(&mut scope, ctx);
        let try_catch = &mut v8::TryCatch::new(&mut ctx_scope);

        let global = try_catch.get_current_context().global(try_catch);
        let func_name_v8 =
            new_v8_string(try_catch, FUNC_NAME).context("creating function name string")?;

        let func_v8 = global
            .get(try_catch, func_name_v8.into())
            .context("getting function from global object")?;

        Ok(!func_v8.is_undefined())
    })?
    .wrap_error()?;
    assert_that!(func_existed_on_other_context, is_false());

    drop(tls_isolate);

    Ok(())
}

#[gtest]
fn test_set_and_get_values_in_separate_contexts() -> googletest::Result<()> {
    fn set_value(
        tls_isolate: &mut TlsIsolateGuard,
        ctx_global: &v8::Global<v8::Context>,
        value: &str,
    ) -> Result<()> {
        let mut scope = tls_isolate.scope();
        let ctx = v8::Local::new(&mut scope, ctx_global);

        let mut ctx_scope = v8::ContextScope::new(&mut scope, ctx);
        let try_catch = &mut v8::TryCatch::new(&mut ctx_scope);

        let key = new_v8_string(try_catch, "foo")?;
        let value = new_v8_string(try_catch, value)?;

        let global_obj = ctx.global(try_catch);
        global_obj
            .set(try_catch, key.cast(), value.cast())
            .ok_or_else(|| try_catch_to_result(try_catch))?;

        Ok(())
    }

    fn get_value(
        tls_isolate: &mut TlsIsolateGuard,
        ctx_global: &v8::Global<v8::Context>,
    ) -> Result<String> {
        let mut scope = tls_isolate.scope();

        let ctx = v8::Local::new(&mut scope, ctx_global);
        let mut ctx_scope = v8::ContextScope::new(&mut scope, ctx);
        let try_catch = &mut v8::TryCatch::new(&mut ctx_scope);

        let global_obj = ctx.global(try_catch);

        let key = new_v8_string(try_catch, "foo")?;
        Ok(global_obj
            .get(try_catch, key.cast())
            .ok_or_else(|| try_catch_to_result(try_catch))?
            .try_cast::<v8::String>()?
            .to_rust_string_lossy(try_catch))
    }

    const SET_VALUE_1: &str = "value in context 1";
    const SET_VALUE_2: &str = "value in context 2";

    init_v8_for_testing();

    let tls_isolate = TlsIsolate::for_current_thread().wrap_error()?;

    let (ctx_1_global, ctx_2_global) =
        try_with_isolate(|tls_isolate| (tls_isolate.new_ctx(), tls_isolate.new_ctx()))?;

    try_with_isolate(|tls_isolate| -> Result<()> {
        set_value(tls_isolate, &ctx_1_global, SET_VALUE_1)?;
        set_value(tls_isolate, &ctx_2_global, SET_VALUE_2)?;

        Ok(())
    })?
    .wrap_error()?;

    let (value_1, value_2) = try_with_isolate(|tls_isolate| -> Result<(String, String)> {
        let value_1 = get_value(tls_isolate, &ctx_1_global)?;
        let value_2 = get_value(tls_isolate, &ctx_2_global)?;

        Ok((value_1, value_2))
    })?
    .wrap_error()?;

    assert_that!(value_1, eq(SET_VALUE_1));
    assert_that!(value_2, eq(SET_VALUE_2));

    drop(tls_isolate);

    Ok(())
}
