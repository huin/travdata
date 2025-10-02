use anyhow::{Result, anyhow};
use googletest::prelude::*;
use hashbrown::{HashMap, HashSet};
use map_macro::hashbrown::{hash_map, hash_map_e, hash_set};

use super::*;
use crate::{plparams, testutil::*};

#[gtest]
fn test_params() -> Result<()> {
    let mut foo_sys = MockFakeSystem::new();
    let mut bar_sys = MockFakeSystem::new();

    // GIVEN: a Foo node.
    let foo_node = Rc::new(FakeNode::default_with_spec(FooSpec::default()));

    // GIVEN: a Bar node.
    let bar_node = Rc::new(FakeNode::default_with_spec(BarSpec::default()));

    let foo_param_id = plparams::ParamId::from_static("foo-param");
    let bar_param_id = plparams::ParamId::from_static("bar-param");

    // GIVEN: the foo_sys will return the given parameters.
    foo_sys
        .expect_params()
        .withf_st({
            let foo_node = foo_node.clone();
            move |node, _| node == foo_node.as_ref()
        })
        .return_once_st({
            let foo_param_id = foo_param_id.clone();
            move |_, reg| {
                reg.add_param(
                    foo_param_id.clone(),
                    TestParamType::TypeOne,
                    "foo-param description.".into(),
                );
                Ok(())
            }
        });

    // GIVEN: the bar_sys will return the given parameters.
    bar_sys
        .expect_params()
        .withf_st({
            let bar_node = bar_node.clone();
            move |node, _| node == bar_node.as_ref()
        })
        .return_once_st({
            let bar_param_id = bar_param_id.clone();
            move |_, reg| {
                reg.add_param(
                    bar_param_id.clone(),
                    TestParamType::TypeTwo,
                    "bar-param description.".into(),
                );
                Ok(())
            }
        });

    // GIVEN: a meta_system that dispatches for Foo and Bar.
    let systems: TestSystemMap = hash_map_e! {
        FakeSpecDiscriminants::Foo => Rc::new(foo_sys),
        FakeSpecDiscriminants::Bar => Rc::new(bar_sys),
    };
    let meta_system = GenericMetaSystem::new(systems);

    // GIVEN: a registrator.
    let mut reg = TestParams::registrator();

    // WHEN: the params are requested for the Foo and Bar nodes.
    meta_system.params(&foo_node, &mut reg.for_node(&foo_node.id))?;
    meta_system.params(&bar_node, &mut reg.for_node(&bar_node.id))?;

    // THEN: both params should be present in the result.
    let got_params = reg.build();
    let want_params = TestParams {
        params: hash_map! {
            plparams::ParamKey::new(
                foo_node.id.clone(),
                foo_param_id,
            ) => TestParam {
                description: "foo-param description.".into(),
                param_type: TestParamType::TypeOne,
            },
            plparams::ParamKey::new(
                bar_node.id.clone(),
                bar_param_id,
            ) => TestParam {
                description: "bar-param description.".into(),
                param_type: TestParamType::TypeTwo,
            },
        },
    };
    expect_that!(got_params, eq(&want_params));

    Ok(())
}

#[gtest]
fn test_inputs() -> Result<()> {
    let mut foo_sys = MockFakeSystem::new();
    let mut bar_sys = MockFakeSystem::new();

    // GIVEN: foo_sys and bar_sys will return clones of a node's spec's dependencies
    foo_sys
        .expect_inputs()
        .withf_st(|node, _| matches!(node.spec, FakeSpec::Foo(_)))
        .returning_st(|node, reg| node.add_inputs(reg));
    bar_sys
        .expect_inputs()
        .withf_st(|node, _| matches!(node.spec, FakeSpec::Bar(_)))
        .returning_st(|node, reg| node.add_inputs(reg));

    let systems: TestSystemMap = hash_map_e! {
        FakeSpecDiscriminants::Foo => Rc::new(foo_sys),
        FakeSpecDiscriminants::Bar => Rc::new(bar_sys),
    };

    // GIVEN: a meta_system that dispatches for Foo and Bar.
    let meta_system = GenericMetaSystem::new(systems);

    // WHEN: the inputs for `Foo` and `Bar` nodes are requested.
    let mut reg = plinputs::InputsRegistrator::new();
    meta_system.inputs(
        &FakeNode {
            spec: FooSpec {
                deps: vec![node_id("foo-dep-1"), node_id("foo-dep-2")],
                ..Default::default()
            }
            .into(),
            ..Default::default()
        },
        &mut reg.for_node(&node_id("foo")),
    )?;
    meta_system.inputs(
        &FakeNode {
            spec: BarSpec {
                deps: vec![node_id("bar-dep-1"), node_id("bar-dep-2")],
                ..Default::default()
            }
            .into(),
            ..Default::default()
        },
        &mut reg.for_node(&node_id("bar")),
    )?;

    // THEN: the expected dependencies are registered.
    let inputs = reg.build();
    expect_that!(
        inputs,
        eq(&hash_map! {
            node_id("foo") => hash_set! {node_id("foo-dep-1"), node_id("foo-dep-2")},
            node_id("bar") => hash_set! {node_id("bar-dep-1"), node_id("bar-dep-2")},
        })
    );

    Ok(())
}

fn process_fixture() -> (TestArgSet, TestIntermediateSet) {
    // GIVEN: arguments.
    let mut args = TestArgSet::default();
    args.set(
        node_id("foo-1"),
        plparams::ParamId::from_static("param-1"),
        TestArgValue::TypeOne(3),
    );
    args.set(
        node_id("foo-2"),
        plparams::ParamId::from_static("param-1"),
        TestArgValue::TypeOne(4),
    );
    args.set(
        node_id("bar"),
        plparams::ParamId::from_static("param-1"),
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
                    args.get(
                        &node_id("foo-1"),
                        &plparams::ParamId::from_static("param-1")
                    ),
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
                    args.get(
                        &node_id("foo-2"),
                        &plparams::ParamId::from_static("param-1")
                    ),
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
                    args.get(&node_id("bar"), &plparams::ParamId::from_static("param-1")),
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
