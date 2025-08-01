use googletest::prelude::*;
use testutils::WrapError;

use super::{testisolate::IsolateThreadHandleForTest, *};

#[gtest]
fn test_thread_isolate_create_and_call_function(
    handle: &&IsolateThreadHandleForTest,
) -> googletest::Result<()> {
    let ctx_client = handle.new_context().wrap_error()?;

    let result: f64 = ctx_client
        .run(|try_catch| {
            let func_v8 = new_v8_function(
                try_catch,
                &["arg1"],
                &ESScriptOrigin::default(),
                r#"return arg1 + 2"#,
            )?;

            let global = try_catch.get_current_context().global(try_catch);
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
        })
        .context("calling client.run")
        .wrap_error()?;

    assert_that!(result, approx_eq(5.0));

    Ok(())
}

#[gtest]
fn test_thread_isolate_create_store_and_later_use_function(
    handle: &&IsolateThreadHandleForTest,
) -> googletest::Result<()> {
    // Given two contexts.
    let ctx_client_1 = handle.new_context().wrap_error()?;
    let ctx_client_2 = handle.new_context().wrap_error()?;

    const FUNC_NAME: &str = "my_func";

    // Given a function is created on the first context's global.
    ctx_client_1
        .run(|try_catch| {
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
        })
        .context("calling client.run")
        .wrap_error()?;

    // Then calling the function in the first context should work and return the expected answer.
    let result = ctx_client_1
        .run(|try_catch| {
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
        })
        .context("calling client.run")
        .wrap_error()?;
    assert_that!(result, approx_eq(5.0));

    // Then the function should not be present in the second context.
    let func_existed_on_other_context = ctx_client_2
        .run(|try_catch| {
            let global = try_catch.get_current_context().global(try_catch);
            let func_name_v8 =
                new_v8_string(try_catch, FUNC_NAME).context("creating function name string")?;

            let func_v8 = global
                .get(try_catch, func_name_v8.into())
                .context("getting function from global object")?;

            Ok(!func_v8.is_undefined())
        })
        .context("calling client.run")
        .wrap_error()?;
    assert_that!(func_existed_on_other_context, is_false());

    Ok(())
}
