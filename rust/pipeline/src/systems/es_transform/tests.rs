use generic_pipeline::systems::GenericSystem;
use googletest::prelude::*;
use testutils::{DefaultForTest, WrapError};
use v8wrapper::testisolate::IsolateThreadHandleForTest;

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
fn test_inputs() {}

#[gtest]
fn test_process() {}
