// TODO: Delete this crate when no longer needed for experimentation.

use std::cell::RefCell;

use anyhow::Result;

thread_local! {
    static ISOLATE: RefCell<Option<v8::OwnedIsolate>> = const { RefCell::new(None) };
}

static INIT_V8: std::sync::OnceLock<()> = std::sync::OnceLock::new();

fn init_v8() {
    INIT_V8.get_or_init(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

fn init_tls_isolate() {
    ISOLATE.with(|isolate| isolate.replace(Some(v8::Isolate::new(v8::CreateParams::default()))));
}

fn do_isolate_things(isolate: &mut v8::OwnedIsolate, i: usize) -> Result<()> {
    let mut scope = v8::HandleScope::new(isolate);
    let ctx = v8::Context::new(&mut scope, v8::ContextOptions::default());
    // For a Global<Context>: let ctx = v8::Global::new(&mut scope, ctx);

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
}

fn main() {
    init_v8();

    for _ in 0..5 {
        std::thread::scope(|s| {
            const NUM_THREADS: usize = 10;
            let barrier = std::sync::Arc::new(std::sync::Barrier::new(NUM_THREADS));
            let mut thread_handles = Vec::with_capacity(NUM_THREADS);

            for i in 0..10 {
                let barrier = barrier.clone();
                let handle = s.spawn(move || {
                    init_tls_isolate();

                    ISOLATE.with(|isolate| {
                        barrier.wait();
                        let result = do_isolate_things(isolate.borrow_mut().as_mut().unwrap(), i);
                        if let Err(err) = result {
                            log::error!("Error: {err}");
                        }
                    });
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
