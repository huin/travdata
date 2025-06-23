use googletest::prelude::*;

use super::*;

/// Adapts [anyhow::Error] to [std::error::Error] to make it compatible with [googletest] tests
/// that use fixtures.
#[derive(Debug)]
struct WrappedError(anyhow::Error);

trait WrapError<T> {
    fn wrap_error(self) -> std::result::Result<T, WrappedError>;
}

/// Trait to convert an [anyhow::Result] to a [std::result::Result<T, WrappedError>].
impl<T> WrapError<T> for anyhow::Result<T> {
    fn wrap_error(self) -> std::result::Result<T, WrappedError> {
        self.map_err(WrappedError::from)
    }
}

impl std::fmt::Display for WrappedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for WrappedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<anyhow::Error> for WrappedError {
    fn from(value: anyhow::Error) -> Self {
        Self(value)
    }
}

#[gtest]
fn test_thread_isolate_create_and_call_function(
    handle: &&IsolateThreadHandleForTest,
) -> googletest::Result<()> {
    let client = handle.create_client();

    let ctx_key = client.new_context().wrap_error()?;
    let result: f64 = client
        .run(&ctx_key, |try_catch| {
            let func_v8 = new_v8_function(
                try_catch,
                &["arg1"],
                &ESScriptOrigin::default(),
                r#"return arg1 + 2"#,
            )?;

            let global = try_catch.get_current_context().global(try_catch);
            let arg1_v8 = new_v8_number(try_catch, 3.0);
            let result_v8 = func_v8
                .call(try_catch, global.safe_cast(), &[arg1_v8.safe_cast()])
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
    let client = handle.create_client();

    // Given two contexts.
    let ctx_key_1 = client.new_context().wrap_error()?;
    let ctx_key_2 = client.new_context().wrap_error()?;

    const FUNC_NAME: &str = "my_func";

    // Given a function is created on the first context's global.
    client
        .run(&ctx_key_1, |try_catch| {
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
                .set(try_catch, func_name_v8.safe_cast(), func_v8.safe_cast())
                .context("setting function on global object")?;

            Ok(())
        })
        .context("calling client.run")
        .wrap_error()?;

    // Then calling the function in the first context should work and return the expected answer.
    let result = client
        .run(&ctx_key_1, |try_catch| {
            let global = try_catch.get_current_context().global(try_catch);
            let func_name_v8 =
                new_v8_string(try_catch, FUNC_NAME).context("creating function name string")?;

            let func_v8 = global
                .get(try_catch, func_name_v8.safe_cast())
                .context("getting function from global object")?
                .try_cast::<v8::Function>()
                .context("casting Value to Function")?;

            let arg1_v8 = new_v8_number(try_catch, 3.0);
            let result_v8 = func_v8
                .call(try_catch, global.safe_cast(), &[arg1_v8.safe_cast()])
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
    let func_existed_on_other_context = client
        .run(&ctx_key_2, |try_catch| {
            let global = try_catch.get_current_context().global(try_catch);
            let func_name_v8 =
                new_v8_string(try_catch, FUNC_NAME).context("creating function name string")?;

            let func_v8 = global
                .get(try_catch, func_name_v8.safe_cast())
                .context("getting function from global object")?;

            Ok(!func_v8.is_undefined())
        })
        .context("calling client.run")
        .wrap_error()?;
    assert_that!(func_existed_on_other_context, is_false());

    Ok(())
}
