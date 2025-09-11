// TODO: Delete this crate when no longer needed for experimentation.

use anyhow::Result;

fn set_value(
    tls_isolate: &mut v8wrapper::TlsIsolateGuard,
    ctx_global: &v8::Global<v8::Context>,
    value: &str,
) -> Result<()> {
    let mut scope = tls_isolate.scope();
    let ctx = v8::Local::new(&mut scope, ctx_global);

    let mut ctx_scope = v8::ContextScope::new(&mut scope, ctx);
    let try_catch = &mut v8::TryCatch::new(&mut ctx_scope);

    let key = v8wrapper::new_v8_string(try_catch, "foo")?;
    let value = v8wrapper::new_v8_string(try_catch, value)?;

    let global_obj = ctx.global(try_catch);
    global_obj
        .set(try_catch, key.cast(), value.cast())
        .ok_or_else(|| v8wrapper::try_catch_to_result(try_catch))?;

    Ok(())
}

fn get_value(
    tls_isolate: &mut v8wrapper::TlsIsolateGuard,
    ctx_global: &v8::Global<v8::Context>,
) -> Result<String> {
    let mut scope = tls_isolate.scope();

    let ctx = v8::Local::new(&mut scope, ctx_global);
    let mut ctx_scope = v8::ContextScope::new(&mut scope, ctx);
    let try_catch = &mut v8::TryCatch::new(&mut ctx_scope);

    let global_obj = ctx.global(try_catch);

    let key = v8wrapper::new_v8_string(try_catch, "foo")?;
    Ok(global_obj
        .get(try_catch, key.cast())
        .ok_or_else(|| v8wrapper::try_catch_to_result(try_catch))?
        .try_cast::<v8::String>()?
        .to_rust_string_lossy(try_catch))
}

fn do_isolate_things(i: usize) -> Result<()> {
    v8wrapper::try_with_isolate(|tls_isolate| -> Result<()> {
        let mut scope = tls_isolate.scope();
        let ctx = v8::Context::new(&mut scope, v8::ContextOptions::default());

        let mut ctx_scope = v8::ContextScope::new(&mut scope, ctx);
        let try_catch = &mut v8::TryCatch::new(&mut ctx_scope);

        let origin = v8wrapper::ESScriptOrigin {
            resource_name: "foo.js".into(),
            resource_line_offset: 0,
            resource_column_offset: 0,
            script_id: 0,
        };
        let func = v8wrapper::new_v8_function(
            try_catch,
            &["foo"],
            &origin,
            r#"
                return foo * 2
            "#,
        )?;

        let global = ctx.global(try_catch);
        let arg = v8wrapper::new_v8_number(try_catch, i as f64);
        let result = func
            .call(try_catch, global.into(), &[arg.cast()])
            .ok_or_else(|| v8wrapper::try_catch_to_result(try_catch))?;

        println!("result = {i} * 2 = {:?}", result.number_value(try_catch));

        Ok(())
    })??;

    let (ctx_1_global, ctx_2_global) =
        v8wrapper::try_with_isolate(|tls_isolate| (tls_isolate.new_ctx(), tls_isolate.new_ctx()))?;

    v8wrapper::try_with_isolate(|tls_isolate| -> Result<()> {
        set_value(tls_isolate, &ctx_1_global, "value in context 1")?;
        set_value(tls_isolate, &ctx_2_global, "value in context 2")?;

        Ok(())
    })??;

    v8wrapper::try_with_isolate(|tls_isolate| -> Result<()> {
        let value_1 = get_value(tls_isolate, &ctx_1_global)?;
        let value_2 = get_value(tls_isolate, &ctx_2_global)?;

        println!("Got value back out of context 1: {value_1:?}");
        println!("Got value back out of context 2: {value_2:?}");

        Ok(())
    })??;

    Ok(())
}

fn main() {
    v8wrapper::init_v8();

    let _ = simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default());

    for _ in 0..5 {
        std::thread::scope(|s| {
            const NUM_THREADS: usize = 10;
            let barrier = std::sync::Arc::new(std::sync::Barrier::new(NUM_THREADS));
            let mut thread_handles = Vec::with_capacity(NUM_THREADS);

            for i in 0..10 {
                let barrier = barrier.clone();
                let handle = s.spawn(move || {
                    barrier.wait();

                    let tls_isolate = v8wrapper::TlsIsolate::for_current_thread();
                    if let Err(err) = tls_isolate {
                        log::error!("Error creating thread local isolate: {err}");
                        return;
                    }

                    let result = do_isolate_things(i);
                    if let Err(err) = result {
                        log::error!("Error doing things with the isolate: {err}");
                    }
                });
                thread_handles.push(handle);
            }

            for handle in thread_handles {
                if let Err(err) = handle.join() {
                    log::warn!("Error joining thread: {err:?}.");
                }
            }
        });
    }
}
