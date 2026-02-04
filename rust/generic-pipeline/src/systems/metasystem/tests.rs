use std::result::Result;

use googletest::{Result as GResult, prelude::*};
use hashbrown::{HashMap, HashSet};
use map_macro::hashbrown::{hash_map, hash_map_e, hash_set};
use mockall::mock;
use serde::{Deserialize, Serialize};

use super::*;
use crate::{intermediates, node, plargs, plinputs, plparams, systems, testutil::node_id};

#[gtest]
fn test_params() -> GResult<()> {
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
    let meta_system =
        GenericMetaSystem::new(systems, Box::new(|_discrim| TestSystemError::ErrorOne));

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
fn test_inputs() -> GResult<()> {
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
    let meta_system =
        GenericMetaSystem::new(systems, Box::new(|_discrim| TestSystemError::ErrorOne));

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
        .returning_st(|_node, _args, _intermediates| Err(TestSystemError::ErrorOne));

    let systems: TestSystemMap = hash_map_e! {
        FakeSpecDiscriminants::Foo => Rc::new(foo_sys),
        FakeSpecDiscriminants::Bar => Rc::new(bar_sys),
    };

    // GIVEN: a meta_system that dispatches for Foo and Bar.
    let meta_system =
        GenericMetaSystem::new(systems, Box::new(|_discrim| TestSystemError::ErrorOne));

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
                    value: Err(TestSystemError::ErrorOne),
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
    let meta_system =
        GenericMetaSystem::new(systems, Box::new(|_discrim| TestSystemError::ErrorOne));

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

/// Per-type wrapper of a specific type of extraction configuration node.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, strum_macros::EnumDiscriminants)]
#[strum_discriminants(derive(Hash))]
#[serde(tag = "type", content = "spec")]
pub enum FakeSpec {
    Foo(FooSpec),
    Bar(BarSpec),
}

impl DiscriminatedSpec for FakeSpec {
    type Discrim = FakeSpecDiscriminants;

    fn discriminant(&self) -> Self::Discrim {
        self.into()
    }
}

impl From<FooSpec> for FakeSpec {
    fn from(value: FooSpec) -> Self {
        Self::Foo(value)
    }
}

impl From<BarSpec> for FakeSpec {
    fn from(value: BarSpec) -> Self {
        Self::Bar(value)
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FooSpec {
    pub value: String,
    pub deps: Vec<node::NodeId>,
}

impl Default for FooSpec {
    fn default() -> Self {
        Self {
            value: "foo-value".into(),
            deps: Default::default(),
        }
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BarSpec {
    pub value: String,
    pub deps: Vec<node::NodeId>,
}

impl Default for BarSpec {
    fn default() -> Self {
        Self {
            value: "bar-value".into(),
            deps: Default::default(),
        }
    }
}

pub type FakeNode = node::GenericNode<FakeSpec>;

impl Default for FakeNode {
    fn default() -> Self {
        Self {
            id: node_id("default-node-id"),
            tags: Default::default(),
            public: Default::default(),
            spec: FooSpec::default().into(),
        }
    }
}

impl node::GenericNode<FakeSpec> {
    pub fn default_with_spec<S>(spec: S) -> Self
    where
        S: Into<FakeSpec>,
    {
        Self {
            id: node_id("foo"),
            tags: Default::default(),
            public: false,
            spec: spec.into(),
        }
    }

    pub fn add_inputs<'a>(
        &self,
        reg: &'a mut plinputs::NodeInputsRegistrator<'a>,
    ) -> Result<(), TestSystemError> {
        let deps = match &self.spec {
            FakeSpec::Foo(foo_spec) => &foo_spec.deps,
            FakeSpec::Bar(bar_spec) => &bar_spec.deps,
        };

        for dep in deps {
            reg.add_input(dep);
        }

        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum TestParamType {
    TypeOne,
    TypeTwo,
}

pub type TestParam = plparams::GenericParam<TestParamType>;
pub type TestParams = plparams::GenericParams<TestParamType>;

#[derive(Debug, Eq, PartialEq)]
pub enum TestArgValue {
    TypeOne(u16),
    TypeTwo(u32),
}

pub type TestArgSet = plargs::GenericArgSet<TestArgValue>;

#[derive(Debug, Eq, PartialEq)]
pub enum TestIntermediateValue {
    ValueOne(u16),
}

pub type TestIntermediateSet = intermediates::GenericIntermediateSet<TestIntermediateValue>;

pub struct TestPipelineTypes;

impl crate::PipelineTypes for TestPipelineTypes {
    type Spec = FakeSpec;

    type ParamType = TestParamType;

    type ArgValue = TestArgValue;

    type IntermediateValue = TestIntermediateValue;

    type SystemError = TestSystemError;
}

#[derive(Debug, Eq, PartialEq)]
pub enum TestSystemError {
    ErrorOne,
    ErrorTwo,
}

impl std::fmt::Display for TestSystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for TestSystemError {}

pub type TestSystemMap = hashbrown::HashMap<
    FakeSpecDiscriminants,
    std::rc::Rc<dyn systems::GenericSystem<TestPipelineTypes>>,
>;

mock! {
    pub FakeSystem {}

    impl systems::GenericSystem<TestPipelineTypes> for FakeSystem {
        fn params<'a>(
            &self,
            node: &FakeNode,
            params: &'a mut plparams::GenericNodeParamsRegistrator<'a, TestParamType>,
        ) -> Result<(), TestSystemError>;

        fn inputs<'a>(
            &self,
            node: &FakeNode,
            reg: &'a mut plinputs::NodeInputsRegistrator<'a>,
        ) -> Result<(), TestSystemError>;

        fn process(
            &self,
            node: &FakeNode,
            args: &TestArgSet,
            intermediates: &TestIntermediateSet,
        ) -> Result<TestIntermediateValue, TestSystemError>;

        fn process_multiple<'a>(
            &self,
            nodes: &'a [&'a FakeNode],
            args: &plargs::GenericArgSet<TestArgValue>,
            intermediates: &TestIntermediateSet,
        ) -> Vec<systems::NodeResult<TestPipelineTypes>>;
    }
}
