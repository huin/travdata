use generic_pipeline::systems::GenericSystem;
use googletest::prelude::*;
use map_macro::hashbrown::{hash_map, hash_set};
use testutils::{DefaultForTest, WrapError};
use v8wrapper::testisolate::IsolateThreadHandleForTest;

use crate::{specs::EsTransform, testutil::node_id};

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
fn test_process() {}
