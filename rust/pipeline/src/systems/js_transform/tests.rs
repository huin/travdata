use generic_pipeline::{PipelineNode as _, systems::GenericSystem};
use googletest::prelude::*;
use map_macro::hashbrown::{hash_map, hash_set};
use serde_json::json;
use testutils::DefaultForTest;

use crate::{
    NodeMeta, intermediates,
    monomorph::{InputsRegistrator, Params},
    specs::JsTransform,
    testutil::TlsIsolateFixture,
};

use super::*;

#[gtest]
fn test_params(_tls_isolate_fixture: &TlsIsolateFixture) -> Result<()> {
    let system = JsTransformSystem;

    let mut reg = Params::registrator();

    let node = crate::Node {
        meta: DefaultForTest::default_for_test(),
        spec: crate::specs::Spec::JsTransform(JsTransform {
            context: "context-id".into(),
            input_data: hash_map! {},
            code: "".into(),
        }),
    };

    system.params(&node, &mut reg.for_node(node.id()))?;
    let got_params = reg.build();

    expect_that!(got_params.params, is_empty());
    Ok(())
}

#[gtest]
fn test_inputs(_tls_isolate_fixture: &TlsIsolateFixture) -> Result<()> {
    let system = JsTransformSystem;

    let mut reg = InputsRegistrator::new();

    let node = crate::Node {
        meta: NodeMeta::new("foo"),
        spec: crate::specs::Spec::JsTransform(JsTransform {
            context: "context-id".into(),
            input_data: hash_map! {
                "a".into() => "foo-dep-1".into(),
                "b".into() => "foo-dep-2".into(),
            },
            code: "".into(),
        }),
    };

    system.inputs(&node, &mut reg.for_node(node.id()))?;
    let got_inputs = reg.build();

    expect_that!(
        got_inputs,
        eq(&hash_map! {
            "foo".into() => hash_set! {
                "context-id".into(),
                "foo-dep-1".into(),
                "foo-dep-2".into(),
            },
        })
    );

    Ok(())
}

#[gtest]
fn test_process_syntax_error(_tls_isolate_fixture: &TlsIsolateFixture) -> Result<()> {
    let system = JsTransformSystem;

    let node = crate::Node {
        meta: DefaultForTest::default_for_test(),
        spec: crate::specs::Spec::JsTransform(JsTransform {
            code: "I'm invalid JavaScript!".into(),
            ..DefaultForTest::default_for_test()
        }),
    };

    let got = system.process(&node, &Default::default(), &Default::default());

    expect_that!(got, err(anything()));

    Ok(())
}

#[gtest]
fn test_process_uses_intermediate_values(_tls_isolate_fixture: &TlsIsolateFixture) -> Result<()> {
    let system = JsTransformSystem;

    let node = crate::Node {
        meta: DefaultForTest::default_for_test(),
        spec: crate::specs::Spec::JsTransform(JsTransform {
            context: "context-id".into(),
            input_data: hash_map! {
                "a".into() => "node-a".into(),
                "b".into() => "node-b".into(),
            },
            code: r#"
                return a + " " + b
            "#
            .into(),
        }),
    };

    let context = v8wrapper::try_with_isolate(|tls_isolate| -> v8::Global<v8::Context> {
        tls_isolate.new_ctx()
    })?;

    let mut interms = intermediates::IntermediateSet::new();
    interms.set(
        "context-id".into(),
        intermediates::JsContext(context).into(),
    );
    interms.set(
        "node-a".into(),
        intermediates::JsonData(json!("foo")).into(),
    );
    interms.set(
        "node-b".into(),
        intermediates::JsonData(json!("bar")).into(),
    );
    let got = system.process(&node, &Default::default(), &interms);

    let expected = intermediates::JsonData(json!("foo bar")).into();
    expect_that!(got, ok(eq(&expected)));

    Ok(())
}

// TODO: test that intermediate values are frozen
