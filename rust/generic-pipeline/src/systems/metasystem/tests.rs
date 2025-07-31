use anyhow::anyhow;
use googletest::prelude::*;
use hashbrown::{HashMap, HashSet};
use map_macro::hashbrown::{hash_map_e, hash_set};

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

fn process_fixture() -> (TestArgSet, TestIntermediateSet) {
    // GIVEN: arguments.
    let mut args = TestArgSet::default();
    args.set(
        node_id("foo-1"),
        plparams::ParamId("param-1"),
        TestArgValue::TypeOne(3),
    );
    args.set(
        node_id("foo-2"),
        plparams::ParamId("param-1"),
        TestArgValue::TypeOne(4),
    );
    args.set(
        node_id("bar"),
        plparams::ParamId("param-1"),
        TestArgValue::TypeTwo(4),
    );

    // GIVEN: intermediates.
    let mut intermediates = TestIntermediateSet::new();
    intermediates.set(node_id("base-node"), TestIntermediateValue::ValueOne(2));

    (args, intermediates)
}

#[gtest]
fn test_process() {
    let mut foo_sys = MockFakeSystem::new();
    let mut bar_sys = MockFakeSystem::new();

    // GIVEN: foo_sys and bar_sys will return clones of a node's spec's dependencies
    foo_sys
        .expect_process()
        .withf_st(|node, args, intermediates| {
            node.id == node_id("foo-1")
                && matches!(node.spec, FakeSpec::Foo(_))
                && matches!(
                    args.get(&node_id("foo-1"), &plparams::ParamId("param-1")),
                    Some(&TestArgValue::TypeOne(3))
                )
                && matches!(
                    intermediates.get(&node_id("base-node")),
                    Some(&TestIntermediateValue::ValueOne(2))
                )
        })
        .returning_st(|_node, _args, _intermediates| Ok(TestIntermediateValue::ValueOne(1)));
    foo_sys
        .expect_process()
        .withf_st(|node, args, intermediates| {
            node.id == node_id("foo-2")
                && matches!(node.spec, FakeSpec::Foo(_))
                && matches!(
                    args.get(&node_id("foo-2"), &plparams::ParamId("param-1")),
                    Some(&TestArgValue::TypeOne(4))
                )
                && matches!(
                    intermediates.get(&node_id("base-node")),
                    Some(&TestIntermediateValue::ValueOne(2))
                )
        })
        .returning_st(|_node, _args, _intermediates| Ok(TestIntermediateValue::ValueOne(2)));
    bar_sys
        .expect_process()
        .withf_st(|node, args, intermediates| {
            node.id == node_id("bar")
                && matches!(node.spec, FakeSpec::Bar(_))
                && matches!(
                    args.get(&node_id("bar"), &plparams::ParamId("param-1")),
                    Some(&TestArgValue::TypeTwo(4))
                )
                && matches!(
                    intermediates.get(&node_id("base-node")),
                    Some(&TestIntermediateValue::ValueOne(2))
                )
        })
        .returning_st(|_node, _args, _intermediates| Err(anyhow!("some error")));

    let systems: TestSystemMap = hash_map_e! {
        FakeSpecDiscriminants::Foo => Rc::new(foo_sys),
        FakeSpecDiscriminants::Bar => Rc::new(bar_sys),
    };

    // GIVEN: a meta_system that dispatches for Foo and Bar.
    let meta_system = GenericMetaSystem::new(systems);

    let (args, intermediates) = process_fixture();

    // WHEN: process is called with the first Foo node.
    let foo_1_result = meta_system.process(
        &FakeNode {
            id: node_id("foo-1"),
            spec: FooSpec::default().into(),
            ..Default::default()
        },
        &args,
        &intermediates,
    );

    // THEN: the expected result is returned.
    expect_that!(foo_1_result, ok(eq(&TestIntermediateValue::ValueOne(1))));

    // WHEN: process is called with the first Foo node.
    let foo_2_result = meta_system.process(
        &FakeNode {
            id: node_id("foo-2"),
            spec: FooSpec::default().into(),
            ..Default::default()
        },
        &args,
        &intermediates,
    );

    // THEN: the expected result is returned.
    expect_that!(foo_2_result, ok(eq(&TestIntermediateValue::ValueOne(2))));

    // WHEN: process is called with the Bar node.
    let bar_result = meta_system.process(
        &FakeNode {
            id: node_id("bar"),
            spec: BarSpec::default().into(),
            ..Default::default()
        },
        &args,
        &intermediates,
    );

    // THEN: the expected result is returned.
    expect_that!(bar_result, err(anything()));
}

#[gtest]
fn test_process_multiple() {
    let mut foo_sys = MockFakeSystem::new();
    let mut bar_sys = MockFakeSystem::new();

    // GIVEN: foo_sys and bar_sys will return clones of a node's spec's dependencies
    foo_sys
        .expect_process_multiple()
        .withf_st(|nodes, _args, _intermediates| {
            nodes
                .iter()
                .map(|node| node.id.clone())
                .collect::<HashSet<_>>()
                == hash_set! {
                    node_id("foo-1"),
                    node_id("foo-2"),
                }
        })
        .returning_st(|_node, _args, _intermediates| {
            vec![
                NodeResult {
                    id: node_id("foo-1"),
                    value: Ok(TestIntermediateValue::ValueOne(1)),
                },
                NodeResult {
                    id: node_id("foo-2"),
                    value: Err(anyhow!("some error")),
                },
            ]
        });
    bar_sys
        .expect_process_multiple()
        .withf_st(|nodes, _args, _intermediates| {
            nodes
                .iter()
                .map(|node| node.id.clone())
                .collect::<HashSet<_>>()
                == hash_set! {
                    node_id("bar"),
                }
        })
        .returning_st(|_node, _args, _intermediates| {
            vec![NodeResult {
                id: node_id("bar"),
                value: Ok(TestIntermediateValue::ValueOne(3)),
            }]
        });

    let systems: TestSystemMap = hash_map_e! {
        FakeSpecDiscriminants::Foo => Rc::new(foo_sys),
        FakeSpecDiscriminants::Bar => Rc::new(bar_sys),
    };

    // GIVEN: a meta_system that dispatches for Foo and Bar.
    let meta_system = GenericMetaSystem::new(systems);

    let (args, intermediates) = process_fixture();

    // WHEN: process is called with the first Foo node.
    let result = meta_system.process_multiple(
        &[
            &FakeNode {
                id: node_id("foo-1"),
                spec: FooSpec::default().into(),
                ..Default::default()
            },
            &FakeNode {
                id: node_id("foo-2"),
                spec: FooSpec::default().into(),
                ..Default::default()
            },
            &FakeNode {
                id: node_id("bar"),
                spec: BarSpec::default().into(),
                ..Default::default()
            },
        ],
        &args,
        &intermediates,
    );

    // THEN: the expected result is returned.
    expect_that!(result, len(eq(3)));
    let result_map: HashMap<_, _> = result
        .into_iter()
        .map(|node_result| (node_result.id, node_result.value))
        .collect();
    expect_that!(
        result_map.get(&node_id("foo-1")),
        some(ok(eq(&TestIntermediateValue::ValueOne(1)))),
    );
    expect_that!(result_map.get(&node_id("foo-2")), some(err(anything())));
    expect_that!(
        result_map.get(&node_id("bar")),
        some(ok(eq(&TestIntermediateValue::ValueOne(3)))),
    );
}
