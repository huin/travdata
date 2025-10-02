use anyhow::Result;
use generic_pipeline::systems::GenericSystem;
use googletest::prelude::*;
use map_macro::hashbrown::{hash_map, hash_set};
use testutils::DefaultForTest;

use crate::{intermediates, plparams, specs::JsTransform, testutil::node_id};

use super::*;

#[gtest]
fn test_params() -> Result<()> {
    v8wrapper::init_v8_for_testing();
    let tls_isolate = v8wrapper::TlsIsolate::for_current_thread()?;

    let system = JsTransformSystem::new();

    let mut reg = plparams::Params::registrator();

    let node = crate::Node {
        ..DefaultForTest::default_for_test()
    };

    system.params(&node, &mut reg.for_node(&node.id))?;
    let got_params = reg.build();

    expect_that!(got_params.params, is_empty());

    drop(tls_isolate);
    Ok(())
}

#[gtest]
fn test_inputs() -> Result<()> {
    v8wrapper::init_v8_for_testing();
    let tls_isolate = v8wrapper::TlsIsolate::for_current_thread()?;

    let system = JsTransformSystem::new();

    let mut reg = plinputs::InputsRegistrator::new();

    let node = crate::Node {
        id: node_id("foo"),
        spec: crate::specs::Spec::JsTransform(JsTransform {
            context: node_id("context-id"),
            input_data: hash_map! {
                "a".into() => node_id("foo-dep-1"),
                "b".into() => node_id("foo-dep-2"),
            },
            code: "".into(),
        }),
        ..DefaultForTest::default_for_test()
    };

    system.inputs(&node, &mut reg.for_node(&node.id))?;
    let got_inputs = reg.build();

    expect_that!(
        got_inputs,
        eq(&hash_map! {
            node_id("foo") => hash_set! {
                node_id("context-id"),
                node_id("foo-dep-1"),
                node_id("foo-dep-2"),
            },
        })
    );

    drop(tls_isolate);
    Ok(())
}

#[gtest]
fn test_process_syntax_error() -> Result<()> {
    v8wrapper::init_v8_for_testing();
    let tls_isolate = v8wrapper::TlsIsolate::for_current_thread()?;

    let system = JsTransformSystem::new();

    let node = crate::Node {
        spec: crate::specs::Spec::JsTransform(JsTransform {
            code: "I'm invalid JavaScript!".into(),
            ..DefaultForTest::default_for_test()
        }),
        ..DefaultForTest::default_for_test()
    };

    let got = system.process(&node, &Default::default(), &Default::default());

    expect_that!(got, err(anything()));

    drop(tls_isolate);
    Ok(())
}

#[gtest]
fn test_process_uses_intermediate_values() -> Result<()> {
    v8wrapper::init_v8_for_testing();
    let tls_isolate = v8wrapper::TlsIsolate::for_current_thread()?;

    let system = JsTransformSystem::new();

    let node = crate::Node {
        spec: crate::specs::Spec::JsTransform(JsTransform {
            context: node_id("context-id"),
            input_data: hash_map! {
                "a".into() => node_id("node-a"),
                "b".into() => node_id("node-b"),
            },
            code: r#"
                return a + " " + b
            "#
            .into(),
        }),
        ..DefaultForTest::default_for_test()
    };

    let context = v8wrapper::try_with_isolate(|tls_isolate| -> v8::Global<v8::Context> {
        tls_isolate.new_ctx()
    })?;

    let mut interms = intermediates::IntermediateSet::new();
    interms.set(
        node_id("context-id"),
        intermediates::IntermediateValue::JsContext(context),
    );
    interms.set(
        node_id("node-a"),
        intermediates::IntermediateValue::JsonData("foo".into()),
    );
    interms.set(
        node_id("node-b"),
        intermediates::IntermediateValue::JsonData("bar".into()),
    );
    let got = system.process(&node, &Default::default(), &interms);

    expect_that!(
        got,
        ok(eq(&intermediates::IntermediateValue::JsonData(
            serde_json::Value::String("foo bar".into())
        )))
    );

    drop(tls_isolate);
    Ok(())
}

// TODO: test that intermediate values are frozen
