use googletest::prelude::*;
use map_macro::hashbrown::hash_map_e;

use super::*;
use crate::{plparams, testutil::*};

#[gtest]
fn test_params() {
    let mut foo_sys = MockFakeSystem::new();
    let mut bar_sys = MockFakeSystem::new();

    // GIVEN: a Foo node.
    let foo_node = Rc::new(FakeNode::default_with_spec(FooSpec::default()));

    // GIVEN: a Bar node.
    let bar_node = Rc::new(FakeNode::default_with_spec(BarSpec::default()));

    // GIVEN: the foo_sys will return the given parameters.
    let new_expected_foo_params = || TestParams {
        params: vec![TestParam {
            param_id: plparams::ParamId("foo-param"),
            description: "foo-param description.".into(),
            param_type: TestParamType::TypeOne,
        }],
    };
    foo_sys
        .expect_params()
        .withf_st({
            let foo_node = foo_node.clone();
            move |node| node == foo_node.as_ref()
        })
        .return_once_st(move |_| new_expected_foo_params());

    // GIVEN: the bar_sys will return the given parameters.
    let new_expected_bar_params = || TestParams {
        params: vec![TestParam {
            param_id: plparams::ParamId("bar-param"),
            description: "bar-param description.".into(),
            param_type: TestParamType::TypeTwo,
        }],
    };
    bar_sys
        .expect_params()
        .withf_st({
            let bar_node = bar_node.clone();
            move |node| node == bar_node.as_ref()
        })
        .return_once_st(move |_| new_expected_bar_params());

    // GIVEN: a meta_system that dispatches for Foo and Bar.
    let systems: TestSystemMap = hash_map_e! {
        FakeSpecDiscriminants::Foo => Rc::new(foo_sys),
        FakeSpecDiscriminants::Bar => Rc::new(bar_sys),
    };
    let meta_system = GenericMetaSystem::new(systems);

    // WHEN: the params are requested for the Foo node.
    let got_foo_params = meta_system.params(&foo_node);

    // THEN: the single param should be for an input PDF file path.
    let expected_foo_params = new_expected_foo_params();
    expect_that!(got_foo_params, eq(&expected_foo_params));

    // WHEN: the params are requested for the Bar node.
    let got_bar_params = meta_system.params(&bar_node);

    // THEN: the single param should be for an output JSON file path.
    let expected_bar_params = new_expected_bar_params();
    expect_that!(got_bar_params, eq(&expected_bar_params));
}

#[gtest]
fn test_inputs() {
    let mut foo_sys = MockFakeSystem::new();
    let mut bar_sys = MockFakeSystem::new();

    // GIVEN: foo_sys and bar_sys will return clones of a node's spec's dependencies
    foo_sys
        .expect_inputs()
        .withf_st(|node| matches!(node.spec, FakeSpec::Foo(_)))
        .returning_st(|node| match &node.spec {
            FakeSpec::Foo(spec) => spec.deps.clone(),
            _ => vec![],
        });
    bar_sys
        .expect_inputs()
        .withf_st(|node| matches!(node.spec, FakeSpec::Bar(_)))
        .returning_st(|node| match &node.spec {
            FakeSpec::Bar(spec) => spec.deps.clone(),
            _ => vec![],
        });

    let systems: TestSystemMap = hash_map_e! {
        FakeSpecDiscriminants::Foo => Rc::new(foo_sys),
        FakeSpecDiscriminants::Bar => Rc::new(bar_sys),
    };

    // GIVEN: a meta_system that dispatches for Foo and Bar.
    let meta_system = GenericMetaSystem::new(systems);

    // WHEN: the input for a `Foo` node is requested.
    let foo_inputs = meta_system.inputs(&FakeNode {
        spec: FooSpec {
            deps: vec![node_id("foo-dep-1"), node_id("foo-dep-2")],
            ..Default::default()
        }
        .into(),
        ..Default::default()
    });
    // THEN: the expected dependencies are returned.
    expect_that!(
        foo_inputs,
        unordered_elements_are![&node_id("foo-dep-1"), &node_id("foo-dep-2")]
    );

    // WHEN: the input for a `bar` node is requested.
    let bar_inputs = meta_system.inputs(&FakeNode {
        spec: BarSpec {
            deps: vec![node_id("bar-dep-1"), node_id("bar-dep-2")],
            ..Default::default()
        }
        .into(),
        ..Default::default()
    });
    // THEN: the expected dependencies are returned.
    expect_that!(
        bar_inputs,
        unordered_elements_are![&node_id("bar-dep-1"), &node_id("bar-dep-2")]
    );
}
