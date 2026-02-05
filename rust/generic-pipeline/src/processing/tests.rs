use std::rc::Rc;
use std::result::Result;

use googletest::prelude::*;
use hashbrown::HashSet;
use map_macro::hashbrown::hash_map;
use mockall::mock;
use predicates::constant::always;

use crate::{
    intermediates, node, pipeline, plargs, plinputs, plparams, processing,
    processing::{NodeError, NodeUnprocessedReason, UnprocessedDependencyReason},
    systems,
    systems::NodeResult,
    testutil::node_id,
};

const FOO_1_ID: &str = "foo-1-id";
const FOO_2_ID: &str = "foo-2-id";
const BAR_1_ID: &str = "bar-1-id";
const BAR_2_ID: &str = "bar-2-id";

#[gtest]
#[test_log::test]
fn test_basic_process_order() {
    let mut sys = MockFakeSystem::new();

    // GIVEN: nodes where BAR_1_ID will depend on FOO_1_ID.
    let node_set = TestPipeline::new(vec![
        foo_node(FOO_1_ID, &[]),
        bar_node(BAR_1_ID, &[FOO_1_ID]),
    ]);

    // GIVEN: inputs() is called for each node, resulting in the dependencies spcified in the
    // specs:
    sys.expect_inputs()
        .returning_st(|node, reg| node.add_inputs(reg));

    // GIVEN: distinct process_multiple calls for foo_1_node followed by bar_1_node.
    let mut process_sequence = mockall::Sequence::new();
    expect_process_multiple(&mut process_sequence, &mut sys, &[FOO_1_ID]);
    expect_process_multiple(&mut process_sequence, &mut sys, &[BAR_1_ID]);

    let sys = Rc::new(sys);
    let processor = TestProcessor::new(sys.clone());
    let args = TestArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    // THEN:
    expect_that!(
        outcome,
        eq(&processing::PipelineOutcome::<TestSystemError> {
            node_results: hash_map! {
                node_id(FOO_1_ID) => Ok(()),
                node_id(BAR_1_ID) => Ok(()),
            },
        }),
    );

    // THEN: the expected process_multiple calls will have been made.
    // (this is implicitly checked by the mock when dropped)
}

#[gtest]
#[test_log::test]
fn test_three_stage_processing() {
    let mut sys = MockFakeSystem::new();

    // GIVEN: nodes where BAR_2_ID will depend on BAR_1_ID which depends on FOO_1_ID.
    let node_set = TestPipeline::new(vec![
        foo_node(FOO_1_ID, &[]),
        bar_node(BAR_1_ID, &[FOO_1_ID]),
        bar_node(BAR_2_ID, &[BAR_1_ID]),
    ]);

    // GIVEN: inputs() is called for each node, resulting in the dependencies spcified in the
    // specs: BAR_2_ID -> BAR_1_ID -> FOO_1_ID.
    sys.expect_inputs()
        .returning_st(|node, reg| node.add_inputs(reg));

    // GIVEN: distinct process_multiple calls for FOO_1_ID followed by BAR_1_ID, and then
    // BAR_2_ID.
    let mut process_sequence = mockall::Sequence::new();
    expect_process_multiple(&mut process_sequence, &mut sys, &[FOO_1_ID]);
    expect_process_multiple(&mut process_sequence, &mut sys, &[BAR_1_ID]);
    expect_process_multiple(&mut process_sequence, &mut sys, &[BAR_2_ID]);

    let sys = Rc::new(sys);
    let processor = TestProcessor::new(sys.clone());
    let args = plargs::GenericArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    // THEN: the outcome reflects the successfully processed nodes.
    expect_that!(
        outcome,
        eq(&processing::PipelineOutcome {
            node_results: hash_map! {
                node_id(FOO_1_ID) => Ok(()),
                node_id(BAR_1_ID) => Ok(()),
                node_id(BAR_2_ID) => Ok(()),
            },
        }),
    );

    // THEN: the expected process_multiple calls will have been made.
    // (this is implicitly checked by the mock when dropped)
}

#[gtest]
#[test_log::test]
fn test_passes_all_runnable_nodes_together() {
    let mut sys = MockFakeSystem::new();

    // GIVEN: nodes where BAR_1_ID will depend on FOO_1_ID, and BAR_2_ID will depend on FOO_2_ID.
    let node_set = TestPipeline::new(vec![
        foo_node(FOO_1_ID, &[]),
        foo_node(FOO_2_ID, &[]),
        bar_node(BAR_1_ID, &[FOO_1_ID]),
        bar_node(BAR_2_ID, &[FOO_2_ID]),
    ]);

    // GIVEN: inputs() is called for each node, resulting in the dependencies spcified in the
    // specs:
    // BAR_1_ID -> FOO_1_ID
    // BAR_2_ID -> FOO_2_ID
    sys.expect_inputs()
        .returning_st(|node, reg| node.add_inputs(reg));

    let mut process_sequence = mockall::Sequence::new();
    // GIVEN: expected call for FOO_1_ID and FOO_2_ID.
    expect_process_multiple(&mut process_sequence, &mut sys, &[FOO_1_ID, FOO_2_ID]);
    // GIVEN: expected call for BAR_1_ID and BAR_2_ID.
    expect_process_multiple(&mut process_sequence, &mut sys, &[BAR_1_ID, BAR_2_ID]);

    let sys = Rc::new(sys);
    let processor = TestProcessor::new(sys.clone());
    let args = plargs::GenericArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    // THEN: the outcome reflects the successfully processed nodes.
    expect_that!(
        outcome,
        eq(&processing::PipelineOutcome {
            node_results: hash_map! {
                node_id(FOO_1_ID) => Ok(()),
                node_id(FOO_2_ID) => Ok(()),
                node_id(BAR_1_ID) => Ok(()),
                node_id(BAR_2_ID) => Ok(()),
            },
        }),
    );

    // THEN: the expected process_multiple calls will have been made.
    // (this is implicitly checked by the mock when dropped)
}

#[gtest]
#[test_log::test]
fn test_handles_direct_loop() {
    let mut sys = MockFakeSystem::new();

    // GIVEN: nodes where FOO_1_ID depends on itself.
    let node_set = TestPipeline::new(vec![foo_node(FOO_1_ID, &[FOO_1_ID])]);

    // GIVEN: inputs() is called for each node, resulting in the dependencies spcified in the
    // specs: FOO_1_ID -> FOO_1_ID
    sys.expect_inputs()
        .returning_st(|node, reg| node.add_inputs(reg));

    // GIVEN: No expected processing calls.

    let sys = Rc::new(sys);
    let processor = TestProcessor::new(sys.clone());
    let args = plargs::GenericArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    // THEN: the outcome reflects the unprocessed dependency loop.
    expect_that!(
        outcome,
        eq(&processing::PipelineOutcome {
            node_results: hash_map! {
                node_id(FOO_1_ID) => Err(NodeError::Unprocessed(NodeUnprocessedReason {
                    unprocessed_dependencies: hash_map! {
                        node_id(FOO_1_ID) => UnprocessedDependencyReason::Unprocessed,
                    }
                })),
            },
        }),
    );

    // THEN: no process_multiple calls will have been made.
}

#[gtest]
#[test_log::test]
fn test_handles_indirect_loop() {
    let mut sys = MockFakeSystem::new();

    // GIVEN: nodes where FOO_1_ID depends on BAR_1_ID which depends on FOO_1_ID, forming a loop.
    let node_set = TestPipeline::new(vec![
        foo_node(FOO_1_ID, &[BAR_1_ID]),
        bar_node(BAR_1_ID, &[FOO_1_ID]),
    ]);

    // GIVEN: inputs() is called for each node, resulting in the dependencies spcified in the
    // specs:
    // BAR_1_ID -> FOO_1_ID -> BAR_1_ID
    sys.expect_inputs()
        .returning_st(|node, reg| node.add_inputs(reg));

    // GIVEN: No expected processing calls.

    let sys = Rc::new(sys);
    let processor = TestProcessor::new(sys.clone());
    let args = plargs::GenericArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    // THEN: the outcome reflects the unprocessed dependency loop.
    expect_that!(
        outcome,
        eq(&processing::PipelineOutcome {
            node_results: hash_map! {
                node_id(FOO_1_ID) => Err(NodeError::Unprocessed(NodeUnprocessedReason {
                    unprocessed_dependencies: hash_map! {
                        node_id(BAR_1_ID) => UnprocessedDependencyReason::Unprocessed,
                    }
                })),
                node_id(BAR_1_ID) => Err(NodeError::Unprocessed(NodeUnprocessedReason {
                    unprocessed_dependencies: hash_map! {
                        node_id(FOO_1_ID) => UnprocessedDependencyReason::Unprocessed,
                    }
                })),
            },
        }),
    );

    // THEN: no process_multiple calls will have been made.
}

#[gtest]
#[test_log::test]
fn test_handles_unknown_dependency() {
    const UNKNOWN_ID: &str = "unknown-id";

    let mut sys = MockFakeSystem::new();

    // GIVEN: nodes where FOO_1_ID depends on BAR_1_ID which depends on FOO_1_ID, forming a loop.
    let node_set = TestPipeline::new(vec![foo_node(FOO_1_ID, &[UNKNOWN_ID])]);

    // GIVEN: inputs() is called for each node, resulting in the dependencies spcified in the
    // specs: FOO_1_ID -> "unknown-id"
    sys.expect_inputs()
        .returning_st(|node, reg| node.add_inputs(reg));

    // GIVEN: No expected processing calls.

    let sys = Rc::new(sys);
    let processor = TestProcessor::new(sys.clone());
    let args = plargs::GenericArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    expect_that!(
        outcome,
        eq(&processing::PipelineOutcome {
            node_results: hash_map! {
                node_id(FOO_1_ID) => Err(NodeError::Unprocessed(NodeUnprocessedReason {
                    unprocessed_dependencies: hash_map! {
                        node_id(UNKNOWN_ID) => UnprocessedDependencyReason::Unknown,
                    }
                })),
            },
        }),
    );

    // THEN: no process_multiple calls will have been made.
}

#[gtest]
#[test_log::test]
fn test_handles_system_error() {
    let mut sys = MockFakeSystem::new();

    // GIVEN: nodes where FOO_1_ID depends on BAR_1_ID which depends on FOO_1_ID, forming a loop.
    let node_set = TestPipeline::new(vec![foo_node(FOO_1_ID, &[])]);

    // GIVEN: inputs() is called for each node.
    sys.expect_inputs()
        .returning_st(|node, reg| node.add_inputs(reg));

    // GIVEN: A process_multiple call that fails with an error.
    sys.expect_process_multiple()
        .with(
            ProcessNodesPredicate::new().with_node_ids(&[FOO_1_ID]),
            always(),
            always(),
        )
        .returning_st(|_, _, _| {
            vec![NodeResult {
                id: node_id(FOO_1_ID),
                value: Err(TestSystemError::ErrorOne),
            }]
        });

    let sys = Rc::new(sys);
    let processor = TestProcessor::new(sys.clone());
    let args = plargs::GenericArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    expect_that!(
        outcome,
        eq(&processing::PipelineOutcome {
            node_results: hash_map! {
                node_id(FOO_1_ID) => Err(NodeError::ProcessErrored(
                    TestSystemError::ErrorOne,
                )),
            },
        }),
    );

    // THEN: no process_multiple calls will have been made.
}

#[gtest]
#[test_log::test]
fn test_passes_intermediates() {
    let mut sys = MockFakeSystem::new();

    // GIVEN: nodes where BAR_1_ID will depend on FOO_1_ID.
    let node_set = TestPipeline::new(vec![
        foo_node(FOO_1_ID, &[]),
        bar_node(BAR_1_ID, &[FOO_1_ID]),
    ]);

    // GIVEN: inputs() is called for each node, resulting in the dependencies spcified in the
    // specs:
    sys.expect_inputs()
        .returning_st(|node, reg| node.add_inputs(reg));

    // GIVEN: distinct process_multiple calls for foo_1_node followed by bar_1_node.
    sys.expect_process_multiple()
        .once()
        .with(
            ProcessNodesPredicate::new().with_node_ids(&[FOO_1_ID]),
            always(),
            always(),
        )
        .returning_st(|_, _, _| {
            vec![NodeResult {
                id: node_id(FOO_1_ID),
                value: Ok(TestIntermediateValue::ValueOne(1)),
            }]
        });
    sys.expect_process_multiple()
        .once()
        .with(
            ProcessNodesPredicate::new().with_node_ids(&[BAR_1_ID]),
            always(),
            predicates::function::function(|interms: &TestIntermediateSet| {
                matches!(
                    interms.get(&node_id(FOO_1_ID)),
                    Some(TestIntermediateValue::ValueOne(1))
                )
            }),
        )
        .returning_st(|_, _, _| {
            vec![NodeResult {
                id: node_id(BAR_1_ID),
                value: Ok(TestIntermediateValue::NoData),
            }]
        });

    let sys = Rc::new(sys);
    let processor = TestProcessor::new(sys.clone());
    let args = plargs::GenericArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    // THEN:
    expect_that!(
        outcome,
        eq(&processing::PipelineOutcome {
            node_results: hash_map! {
                node_id(FOO_1_ID) => Ok(()),
                node_id(BAR_1_ID) => Ok(()),
            },
        }),
    );

    // THEN: the expected process_multiple calls will have been made.
    // (this is implicitly checked by the mock when dropped)
}

fn expect_process_multiple(
    process_sequence: &mut mockall::Sequence,
    sys: &mut MockFakeSystem,
    node_ids: &[&str],
) {
    let mut nodes_predicate = ProcessNodesPredicate::new();
    for id_str in node_ids {
        nodes_predicate.add_node_id(id_str);
    }

    sys.expect_process_multiple()
        .once()
        .with(nodes_predicate, always(), always())
        .returning_st(|nodes, _, _| fake_process_multiple(nodes))
        .in_sequence(process_sequence);
}

fn fake_process_multiple(nodes: &[&FakeNode]) -> Vec<NodeResult<TestPipelineTypes>> {
    nodes
        .iter()
        .map(|node| NodeResult {
            id: node.id.clone(),
            value: Ok(TestIntermediateValue::ValueOne(1)),
        })
        .collect()
}

struct ProcessNodesPredicate {
    want_ids: HashSet<node::NodeId>,
}

impl ProcessNodesPredicate {
    fn new() -> Self {
        Self {
            want_ids: HashSet::new(),
        }
    }

    fn with_node_ids(mut self, id_strs: &[&str]) -> Self {
        for id_str in id_strs {
            self.add_node_id(id_str);
        }
        self
    }

    fn add_node_id(&mut self, id_str: &str) {
        self.want_ids.insert(node_id(id_str));
    }
}

impl std::fmt::Display for ProcessNodesPredicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt_ids: Vec<_> = self
            .want_ids
            .iter()
            .map(|node_id| format!("{node_id:?}"))
            .collect();
        fmt_ids.sort();
        write!(
            f,
            "&[&FakeNode] slice containing node IDs [{}]",
            fmt_ids.join(", ")
        )
    }
}

impl predicates::reflection::PredicateReflection for ProcessNodesPredicate {}

impl mockall::Predicate<[&FakeNode]> for ProcessNodesPredicate {
    fn eval(&self, variable: &[&FakeNode]) -> bool {
        let got_ids: HashSet<node::NodeId> = variable.iter().map(|node| node.id.clone()).collect();
        got_ids == self.want_ids
    }
}

/// Per-type wrapper of a specific type of extraction configuration node.
#[derive(Debug, Eq, PartialEq)]
pub enum FakeSpec {
    Foo(FooSpec),
    Bar(BarSpec),
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

#[derive(Debug, Eq, PartialEq)]
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

#[derive(Debug, Eq, PartialEq)]
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

pub fn foo_node(id: &str, deps: &[&str]) -> FakeNode {
    FakeNode {
        id: node_id(id),
        spec: FooSpec {
            deps: deps.iter().map(|s| node_id(s)).collect(),
            ..Default::default()
        }
        .into(),
        ..Default::default()
    }
}

pub fn bar_node(id: &str, deps: &[&str]) -> FakeNode {
    FakeNode {
        id: node_id(id),
        spec: BarSpec {
            deps: deps.iter().map(|s| node_id(s)).collect(),
            ..Default::default()
        }
        .into(),
        ..Default::default()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct TestParamType {}

#[derive(Debug, Eq, PartialEq)]
pub struct TestArgValue {}

pub type TestArgSet = plargs::GenericArgSet<TestArgValue>;

#[derive(Debug, Eq, PartialEq)]
pub enum TestIntermediateValue {
    NoData,
    ValueOne(u16),
}

pub type TestIntermediateSet = intermediates::GenericIntermediateSet<TestIntermediateValue>;

pub type TestPipeline = pipeline::GenericPipeline<FakeSpec>;

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

pub type TestProcessor = processing::GenericProcessor<TestPipelineTypes>;

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
