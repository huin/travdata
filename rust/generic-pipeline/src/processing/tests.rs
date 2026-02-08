use std::rc::Rc;
use std::result::Result;

use googletest::prelude::*;
use hashbrown::HashSet;
use map_macro::hashbrown::{hash_map, hash_set};

use crate::{
    intermediates::{self, IntermediateError},
    node, pipeline, plargs, plinputs, plparams,
    processing::{self, NodeError, NodeUnprocessedReason, UnprocessedDependencyReason},
    systems::{self, NodeResult},
    testutil::node_id,
};

const NODE_1_ID: &str = "node-1-id";
const NODE_2_ID: &str = "node-2-id";
const NODE_3_ID: &str = "node-3-id";
const NODE_4_ID: &str = "node-4-id";

#[gtest]
#[test_log::test]
fn test_feeds_processed_dependency() {
    let sys = FakeSystem::new();

    let stored_values = StoredValues::new();

    // GIVEN: nodes where NODE_3_ID will depend on NODE_1_ID.
    let node_set = TestPipeline::new(vec![
        value_node(NODE_1_ID, "dependency_value"),
        store_node(NODE_2_ID, NODE_1_ID, stored_values.clone()),
    ]);

    let processor = TestProcessor::new(sys);
    let args = TestArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    // THEN: all nodes should be reported as successful.
    // TODO: Refactor this common value where every node ID is successful.
    expect_that!(
        outcome,
        eq(&processing::PipelineOutcome::<TestSystemError> {
            node_results: hash_map! {
                node_id(NODE_1_ID) => Ok(()),
                node_id(NODE_2_ID) => Ok(()),
            },
        }),
    );

    // THEN: the dependency value should have been stored.
    expect_that!(
        *stored_values.0.borrow(),
        eq(&vec!["dependency_value".to_string()])
    );
}

#[gtest]
#[test_log::test]
fn test_feeds_argument() {
    let sys = FakeSystem::new();

    let stored_values = StoredValues::new();

    // GIVEN: nodes where NODE_3_ID will depend on NODE_1_ID.
    let node_set = TestPipeline::new(vec![
        value_from_arg_node(NODE_1_ID),
        store_node(NODE_2_ID, NODE_1_ID, stored_values.clone()),
    ]);

    let processor = TestProcessor::new(sys);
    let mut args = TestArgSet::default();
    args.set(
        node_id(NODE_1_ID),
        plparams::ParamId::from_static(FakeSystem::PARAM_NAME),
        TestArgValue("arg_value".into()),
    );

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    // THEN: all nodes should be reported as successful.
    expect_that!(
        outcome,
        eq(&processing::PipelineOutcome::<TestSystemError> {
            node_results: hash_map! {
                node_id(NODE_1_ID) => Ok(()),
                node_id(NODE_2_ID) => Ok(()),
            },
        }),
    );

    // THEN: the dependency value should have been stored.
    expect_that!(
        *stored_values.0.borrow(),
        eq(&vec!["arg_value".to_string()])
    );
}

#[gtest]
#[test_log::test]
fn test_errors_on_missing_argument() {
    let sys = FakeSystem::new();

    let stored_values = StoredValues::new();

    // GIVEN: nodes where NODE_3_ID will depend on NODE_1_ID.
    let node_set = TestPipeline::new(vec![
        value_from_arg_node(NODE_1_ID),
        store_node(NODE_2_ID, NODE_1_ID, stored_values.clone()),
    ]);

    let processor = TestProcessor::new(sys);
    let args = TestArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    // THEN: all nodes should be reported as successful.
    expect_that!(
        outcome,
        eq(&processing::PipelineOutcome::<TestSystemError> {
            node_results: hash_map! {
                node_id(NODE_1_ID) => Err(NodeError::ProcessErrored(TestSystemError::Arg(plargs::ArgError::NotFound{
                    node_id: node_id(NODE_1_ID),
                    param_id: plparams::ParamId::from_static(FakeSystem::PARAM_NAME),
                }))),
                node_id(NODE_2_ID) => Err(NodeError::Unprocessed(NodeUnprocessedReason{
                    unprocessed_dependencies: hash_map![node_id(NODE_1_ID) => UnprocessedDependencyReason::Unprocessed],
                })),
            },
        }),
    );
}

#[gtest]
#[test_log::test]
fn test_three_stage_processing() {
    let sys = FakeSystem::new();

    let stored_values = StoredValues::new();

    // GIVEN: nodes where NODE_3_ID will depend on NODE_1_ID.
    let node_set = TestPipeline::new(vec![
        value_node(NODE_1_ID, "foo"),
        value_node(NODE_2_ID, "bar"),
        concat_node(NODE_3_ID, &[NODE_1_ID, NODE_2_ID]),
        store_node(NODE_4_ID, NODE_3_ID, stored_values.clone()),
    ]);

    let processor = TestProcessor::new(sys);
    let args = TestArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    // THEN: all nodes should be reported as successful.
    expect_that!(
        outcome,
        eq(&processing::PipelineOutcome::<TestSystemError> {
            node_results: hash_map! {
                node_id(NODE_1_ID) => Ok(()),
                node_id(NODE_2_ID) => Ok(()),
                node_id(NODE_3_ID) => Ok(()),
                node_id(NODE_4_ID) => Ok(()),
            },
        }),
    );

    // THEN: the concatenated values should have been stored.
    expect_that!(*stored_values.0.borrow(), eq(&vec!["foo,bar".to_string()]));
}

#[gtest]
#[test_log::test]
fn test_passes_all_runnable_nodes_together() {
    let sys = FakeSystem::new();

    // GIVEN: nodes where NODE_3_ID will depend on NODE_1_ID, and NODE_4_ID will depend on NODE_2_ID.
    let node_set = TestPipeline::new(vec![
        value_node(NODE_1_ID, "foo"),
        value_node(NODE_2_ID, "bar"),
        store_node(NODE_3_ID, NODE_1_ID, StoredValues::new()),
        store_node(NODE_4_ID, NODE_2_ID, StoredValues::new()),
    ]);

    let processor = TestProcessor::new(sys.clone());
    let args = plargs::GenericArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    // THEN: the outcome reflects the successfully processed nodes.
    expect_that!(
        outcome,
        eq(&processing::PipelineOutcome {
            node_results: hash_map! {
                node_id(NODE_1_ID) => Ok(()),
                node_id(NODE_2_ID) => Ok(()),
                node_id(NODE_3_ID) => Ok(()),
                node_id(NODE_4_ID) => Ok(()),
            },
        }),
    );

    // THEN: the calls should be grouped as expected.
    expect_that!(
        *sys.process_sets.borrow(),
        eq(&vec![
            hash_set![node_id(NODE_1_ID), node_id(NODE_2_ID)],
            hash_set![node_id(NODE_3_ID), node_id(NODE_4_ID)],
        ])
    );
}

#[gtest]
#[test_log::test]
fn test_handles_direct_loop() {
    let sys = FakeSystem::new();

    // GIVEN: nodes where NODE_1_ID depends on itself.
    let node_set = TestPipeline::new(vec![concat_node(NODE_1_ID, &[NODE_1_ID])]);

    let processor = TestProcessor::new(sys.clone());
    let args = plargs::GenericArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    // THEN: the outcome reflects the unprocessed dependency loop.
    expect_that!(
        outcome,
        eq(&processing::PipelineOutcome {
            node_results: hash_map! {
                node_id(NODE_1_ID) => Err(NodeError::Unprocessed(NodeUnprocessedReason {
                    unprocessed_dependencies: hash_map! {
                        node_id(NODE_1_ID) => UnprocessedDependencyReason::Unprocessed,
                    }
                })),
            },
        }),
    );

    // THEN: no process* calls have been made.
    expect_that!(*sys.process_sets.borrow(), is_empty());
}

#[gtest]
#[test_log::test]
fn test_handles_indirect_loop() {
    let sys = FakeSystem::new();

    // GIVEN: nodes where NODE_1_ID depends on NODE_3_ID which depends on NODE_1_ID, forming a loop.
    let node_set = TestPipeline::new(vec![
        concat_node(NODE_1_ID, &[NODE_3_ID]),
        concat_node(NODE_3_ID, &[NODE_1_ID]),
    ]);

    let processor = TestProcessor::new(sys.clone());
    let args = plargs::GenericArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    // THEN: the outcome reflects the unprocessed dependency loop.
    expect_that!(
        outcome,
        eq(&processing::PipelineOutcome {
            node_results: hash_map! {
                node_id(NODE_1_ID) => Err(NodeError::Unprocessed(NodeUnprocessedReason {
                    unprocessed_dependencies: hash_map! {
                        node_id(NODE_3_ID) => UnprocessedDependencyReason::Unprocessed,
                    }
                })),
                node_id(NODE_3_ID) => Err(NodeError::Unprocessed(NodeUnprocessedReason {
                    unprocessed_dependencies: hash_map! {
                        node_id(NODE_1_ID) => UnprocessedDependencyReason::Unprocessed,
                    }
                })),
            },
        }),
    );

    // THEN: no process* calls have been made.
    expect_that!(*sys.process_sets.borrow(), is_empty());
}

#[gtest]
#[test_log::test]
fn test_handles_unknown_dependency() {
    const UNKNOWN_ID: &str = "unknown-id";

    let sys = FakeSystem::new();

    // GIVEN: nodes where NODE_1_ID depends on NODE_3_ID which depends on NODE_1_ID, forming a loop.
    let node_set = TestPipeline::new(vec![concat_node(NODE_1_ID, &[UNKNOWN_ID])]);

    let processor = TestProcessor::new(sys.clone());
    let args = plargs::GenericArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    expect_that!(
        outcome,
        eq(&processing::PipelineOutcome {
            node_results: hash_map! {
                node_id(NODE_1_ID) => Err(NodeError::Unprocessed(NodeUnprocessedReason {
                    unprocessed_dependencies: hash_map! {
                        node_id(UNKNOWN_ID) => UnprocessedDependencyReason::Unknown,
                    }
                })),
            },
        }),
    );

    // THEN: no process* calls have been made.
    expect_that!(*sys.process_sets.borrow(), is_empty());
}

#[gtest]
#[test_log::test]
fn test_handles_system_error() {
    let sys = FakeSystem::new();

    // GIVEN: nodes that will error when processed.
    let node_set = TestPipeline::new(vec![
        // TODO: uncomment these and fix the implementation.
        // error_node(NODE_1_ID, SystemErrorWhen::Params),
        // error_node(NODE_2_ID, SystemErrorWhen::Inputs),
        error_node(NODE_3_ID, SystemErrorWhen::Process),
    ]);

    let processor = TestProcessor::new(sys.clone());
    let args = plargs::GenericArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    expect_that!(
        outcome,
        eq(&processing::PipelineOutcome {
            node_results: hash_map! {
                // TODO: uncomment these and fix the implementation.
                // node_id(NODE_1_ID) => Err(NodeError::ProcessErrored(
                //     TestSystemError::SystemError,
                // )),
                // node_id(NODE_2_ID) => Err(NodeError::ProcessErrored(
                //     TestSystemError::SystemError,
                // )),
                node_id(NODE_3_ID) => Err(NodeError::ProcessErrored(
                    TestSystemError::System,
                )),
            },
        }),
    );

    // THEN: only NODE_3_ID should have been processed, as the other two failed on preconditions to
    // process.
    expect_that!(
        *sys.process_sets.borrow(),
        eq(&vec![hash_set![node_id(NODE_3_ID)],])
    );
}

/// Per-type wrapper of a specific type of extraction configuration node.
#[derive(Debug)]
enum FakeSpec {
    Value(ValueSpec),
    ValueFromArg(ValueFromArgSpec),
    Concat(ConcatSpec),
    Store(StoreSpec),
    Error(SystemErrorWhen),
}

impl From<ValueSpec> for FakeSpec {
    fn from(value: ValueSpec) -> Self {
        Self::Value(value)
    }
}

impl From<ValueFromArgSpec> for FakeSpec {
    fn from(value: ValueFromArgSpec) -> Self {
        Self::ValueFromArg(value)
    }
}

impl From<ConcatSpec> for FakeSpec {
    fn from(value: ConcatSpec) -> Self {
        Self::Concat(value)
    }
}

impl From<StoreSpec> for FakeSpec {
    fn from(value: StoreSpec) -> Self {
        Self::Store(value)
    }
}

#[derive(Debug, Eq, PartialEq)]
struct ValueSpec {
    value: String,
}

impl Default for ValueSpec {
    fn default() -> Self {
        Self {
            value: "value".into(),
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
struct ValueFromArgSpec;

#[derive(Debug, Default, Eq, PartialEq)]
struct ConcatSpec {
    deps: Vec<node::NodeId>,
}

#[derive(Debug, Eq, PartialEq)]
struct StoredValues(std::cell::RefCell<Vec<String>>);

impl StoredValues {
    fn new() -> Rc<Self> {
        Rc::new(StoredValues(std::cell::RefCell::new(Vec::with_capacity(1))))
    }
}

#[derive(Debug, Eq, PartialEq)]
struct StoreSpec {
    dep: node::NodeId,
    stored_values: Rc<StoredValues>,
}

#[derive(Debug)]
#[expect(dead_code)]
enum SystemErrorWhen {
    Params,
    Inputs,
    Process,
}

type FakeNode = node::GenericNode<FakeSpec>;

impl Default for FakeNode {
    fn default() -> Self {
        Self {
            id: node_id("default-node-id"),
            tags: Default::default(),
            public: Default::default(),
            spec: ValueSpec::default().into(),
        }
    }
}

fn value_node(id: &str, value: &str) -> FakeNode {
    FakeNode {
        id: node_id(id),
        spec: ValueSpec {
            value: value.into(),
        }
        .into(),
        ..Default::default()
    }
}

fn value_from_arg_node(id: &str) -> FakeNode {
    FakeNode {
        id: node_id(id),
        spec: ValueFromArgSpec.into(),
        ..Default::default()
    }
}

fn concat_node(id: &str, deps: &[&str]) -> FakeNode {
    FakeNode {
        id: node_id(id),
        spec: ConcatSpec {
            deps: deps.iter().map(|s| node_id(s)).collect(),
        }
        .into(),
        ..Default::default()
    }
}

fn store_node(id: &str, dep: &str, stored_values: Rc<StoredValues>) -> FakeNode {
    FakeNode {
        id: node_id(id),
        spec: StoreSpec {
            dep: node_id(dep),
            stored_values,
        }
        .into(),
        ..Default::default()
    }
}

fn error_node(id: &str, when: SystemErrorWhen) -> FakeNode {
    FakeNode {
        id: node_id(id),
        spec: FakeSpec::Error(when),
        ..Default::default()
    }
}

#[derive(Debug, Eq, PartialEq)]
struct TestParamType;

#[derive(Debug, Eq, PartialEq)]
struct TestArgValue(String);

type TestArgSet = plargs::GenericArgSet<TestArgValue>;

#[derive(Debug, Eq, PartialEq)]
struct TestIntermediateValue(String);

type TestIntermediateSet = intermediates::GenericIntermediateSet<TestIntermediateValue>;

type TestPipeline = pipeline::GenericPipeline<FakeSpec>;

struct TestPipelineTypes;

impl crate::PipelineTypes for TestPipelineTypes {
    type Spec = FakeSpec;

    type ParamType = TestParamType;

    type ArgValue = TestArgValue;

    type IntermediateValue = TestIntermediateValue;

    type SystemError = TestSystemError;
}

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
enum TestSystemError {
    // NeverError is for cases that should never occur (where the processor has broken a basic
    // contract).
    Never(NeverReason),
    System,
    Arg(#[from] plargs::ArgError),
    Intermediate(#[from] intermediates::IntermediateError),
}

#[derive(Debug, Eq, PartialEq)]
enum NeverReason {
    MissingInput(IntermediateError),
    OutOfOrder,
}

impl std::fmt::Display for TestSystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

type TestProcessor = processing::GenericProcessor<TestPipelineTypes>;

struct FakeSystem {
    process_sets: std::cell::RefCell<Vec<HashSet<node::NodeId>>>,
}

impl FakeSystem {
    const PARAM_NAME: &str = "test_param";

    fn new() -> Rc<Self> {
        Rc::new(Self {
            process_sets: Default::default(),
        })
    }

    fn do_process(
        &self,
        node: &FakeNode,
        args: &TestArgSet,
        intermediates: &TestIntermediateSet,
    ) -> Result<TestIntermediateValue, TestSystemError> {
        use FakeSpec::*;
        Ok(match &node.spec {
            Value(value_spec) => TestIntermediateValue(value_spec.value.clone()),
            ValueFromArg(_) => {
                let arg =
                    args.require(&node.id, &plparams::ParamId::from_static(Self::PARAM_NAME))?;
                TestIntermediateValue(arg.0.clone())
            }
            Concat(concat_spec) => {
                let parts: Vec<&str> = concat_spec
                    .deps
                    .iter()
                    .map(|dep| {
                        intermediates
                            .require(dep)
                            .map(|intermediate| intermediate.0.as_str())
                            .map_err(NeverReason::MissingInput)
                            .map_err(TestSystemError::Never)
                    })
                    .collect::<Result<Vec<&str>, TestSystemError>>()?;

                TestIntermediateValue(parts.join(","))
            }
            Store(store_spec) => {
                let value = intermediates.require(&store_spec.dep)?;
                store_spec
                    .stored_values
                    .0
                    .borrow_mut()
                    .push(value.0.clone());
                TestIntermediateValue("".into())
            }
            Error(SystemErrorWhen::Process) => {
                return Err(TestSystemError::System);
            }
            Error(_) => {
                // The processor should never reach this point, it should have probed all other
                // "whens" before getting to processing.
                return Err(TestSystemError::Never(NeverReason::OutOfOrder));
            }
        })
    }
}

impl systems::GenericSystem<TestPipelineTypes> for FakeSystem {
    fn params<'a>(
        &self,
        node: &FakeNode,
        params: &'a mut plparams::GenericNodeParamsRegistrator<'a, TestParamType>,
    ) -> Result<(), TestSystemError> {
        use FakeSpec::*;
        match &node.spec {
            Value(_) => {}
            ValueFromArg(_) => {
                params.add_param(
                    plparams::ParamId::from_static(Self::PARAM_NAME),
                    TestParamType,
                    "Test parameter description.".to_string(),
                );
            }
            Concat(_) => {}
            Store(_) => {}
            Error(SystemErrorWhen::Params) => {
                return Err(TestSystemError::System);
            }
            Error(_) => {}
        }
        Ok(())
    }

    fn inputs<'a>(
        &self,
        node: &FakeNode,
        reg: &'a mut plinputs::NodeInputsRegistrator<'a>,
    ) -> Result<(), TestSystemError> {
        use FakeSpec::*;
        match &node.spec {
            Value(_) => {}
            ValueFromArg(_) => {}
            Concat(concat_spec) => {
                for dep in &concat_spec.deps {
                    reg.add_input(dep);
                }
            }
            Store(store_spec) => {
                reg.add_input(&store_spec.dep);
            }
            Error(SystemErrorWhen::Inputs) => {
                return Err(TestSystemError::System);
            }
            Error(_) => {}
        }
        Ok(())
    }

    fn process(
        &self,
        node: &FakeNode,
        args: &TestArgSet,
        intermediates: &TestIntermediateSet,
    ) -> Result<TestIntermediateValue, TestSystemError> {
        self.process_sets
            .borrow_mut()
            .push(hash_set![node.id.clone()]);
        self.do_process(node, args, intermediates)
    }

    fn process_multiple<'a>(
        &self,
        nodes: &'a [&'a node::GenericNode<<TestPipelineTypes as crate::PipelineTypes>::Spec>],
        args: &plargs::GenericArgSet<<TestPipelineTypes as crate::PipelineTypes>::ArgValue>,
        intermediates: &intermediates::GenericIntermediateSet<
            <TestPipelineTypes as crate::PipelineTypes>::IntermediateValue,
        >,
    ) -> Vec<NodeResult<TestPipelineTypes>> {
        self.process_sets
            .borrow_mut()
            .push(nodes.iter().map(|node| &node.id).cloned().collect());

        nodes
            .iter()
            .map(|node| {
                let value = self.do_process(node, args, intermediates);
                NodeResult {
                    id: node.id.clone(),
                    value,
                }
            })
            .collect()
    }
}
