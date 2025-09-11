use generic_pipeline::systems::GenericSystem;
use googletest::prelude::*;
use map_macro::hashbrown::{hash_map, hash_set};
use testutils::{DefaultForTest, WrapError};
use v8wrapper::testisolate::IsolateThreadHandleForTest;

use crate::{intermediates, specs::EsTransform, testutil::node_id};

use super::*;

#[gtest]
fn test_params(handle: &&IsolateThreadHandleForTest) -> googletest::Result<()> {
    let system = EsTransformSystem::new(handle.new_context().wrap_error()?);

    let mut reg = plparams::Params::registrator();

    let node = crate::Node {
        ..DefaultForTest::default_for_test()
    };

    system.params(&node, &mut reg.for_node(&node.id));
    let got_params = reg.build();

    expect_that!(got_params.params, is_empty());

    Ok(())
}

#[gtest]
fn test_inputs(handle: &&IsolateThreadHandleForTest) -> googletest::Result<()> {
    let system = EsTransformSystem::new(handle.new_context().wrap_error()?);

    let mut reg = plinputs::InputsRegistrator::new();

    let node = crate::Node {
        id: node_id("foo"),
        spec: crate::specs::Spec::EsTransform(EsTransform {
            input_data: hash_map! {
                "a".into() => node_id("foo-dep-1"),
                "b".into() => node_id("foo-dep-2"),
            },
            code: "".into(),
        }),
        ..DefaultForTest::default_for_test()
    };

    system.inputs(&node, &mut reg.for_node(&node.id));
    let got_inputs = reg.build();

    expect_that!(
        got_inputs,
        eq(&hash_map! {
            node_id("foo") => hash_set! {
                node_id("foo-dep-1"),
                node_id("foo-dep-2"),
            },
        })
    );

    Ok(())
}

#[gtest]
fn test_process_syntax_error(handle: &&IsolateThreadHandleForTest) -> googletest::Result<()> {
    let system = EsTransformSystem::new(handle.new_context().wrap_error()?);

    let node = crate::Node {
        spec: crate::specs::Spec::EsTransform(EsTransform {
            code: "I'm invalid ECMAScript!".into(),
            ..DefaultForTest::default_for_test()
        }),
        ..DefaultForTest::default_for_test()
    };

    let got = system.process(&node, &Default::default(), &Default::default());

    expect_that!(got, err(anything()));

    Ok(())
}

#[gtest]
fn test_process_uses_intermediate_values(
    handle: &&IsolateThreadHandleForTest,
) -> googletest::Result<()> {
    let system = EsTransformSystem::new(handle.new_context().wrap_error()?);

    let node = crate::Node {
        spec: crate::specs::Spec::EsTransform(EsTransform {
            input_data: hash_map! {
                "a".into() => node_id("node-a"),
                "b".into() => node_id("node-b"),
            },
            code: r#"
                return a.foo + " " + b.bar
            "#
            .into(),
        }),
        ..DefaultForTest::default_for_test()
    };

    let mut interms = intermediates::IntermediateSet::new();
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

    Ok(())
}

// XXX test that intermediate values are frozen
