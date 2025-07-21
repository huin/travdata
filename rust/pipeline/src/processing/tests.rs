use std::rc::Rc;

use anyhow::anyhow;
use googletest::prelude::*;
use hashbrown::HashSet;
use map_macro::hashbrown::hash_map;
use predicates::constant::always;

use crate::{
    intermediates::{self, IntermediateSet},
    node, processargs,
    processing::{self, NodeProcessOutcome, NodeUnprocessedReason, UnprocessedDependencyReason},
    testutil::*,
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
    let node_set = TestNodeSet::new(vec![
        foo_node(FOO_1_ID, &[]),
        bar_node(BAR_1_ID, &[FOO_1_ID]),
    ]);

    // GIVEN: inputs() is called for each node, resulting in the dependencies spcified in the
    // specs:
    sys.expect_inputs().returning_st(|node| node.deps());

    // GIVEN: distinct process_multiple calls for foo_1_node followed by bar_1_node.
    let mut process_sequence = mockall::Sequence::new();
    expect_process_multiple(&mut process_sequence, &mut sys, &[FOO_1_ID]);
    expect_process_multiple(&mut process_sequence, &mut sys, &[BAR_1_ID]);

    let sys = Rc::new(sys);
    let processor = TestProcessor::new(sys.clone());
    let args = processargs::ArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    // THEN:
    expect_that!(
        outcome,
        eq(&processing::ProcessOutcome {
            node_outcomes: hash_map! {
                node_id(FOO_1_ID) => NodeProcessOutcome::Success,
                node_id(BAR_1_ID) => NodeProcessOutcome::Success,
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
    let node_set = TestNodeSet::new(vec![
        foo_node(FOO_1_ID, &[]),
        bar_node(BAR_1_ID, &[FOO_1_ID]),
        bar_node(BAR_2_ID, &[BAR_1_ID]),
    ]);

    // GIVEN: inputs() is called for each node, resulting in the dependencies spcified in the
    // specs: BAR_2_ID -> BAR_1_ID -> FOO_1_ID.
    sys.expect_inputs().returning_st(|node| node.deps());

    // GIVEN: distinct process_multiple calls for FOO_1_ID followed by BAR_1_ID, and then
    // BAR_2_ID.
    let mut process_sequence = mockall::Sequence::new();
    expect_process_multiple(&mut process_sequence, &mut sys, &[FOO_1_ID]);
    expect_process_multiple(&mut process_sequence, &mut sys, &[BAR_1_ID]);
    expect_process_multiple(&mut process_sequence, &mut sys, &[BAR_2_ID]);

    let sys = Rc::new(sys);
    let processor = TestProcessor::new(sys.clone());
    let args = processargs::ArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    // THEN: the outcome reflects the successfully processed nodes.
    expect_that!(
        outcome,
        eq(&processing::ProcessOutcome {
            node_outcomes: hash_map! {
                node_id(FOO_1_ID) => NodeProcessOutcome::Success,
                node_id(BAR_1_ID) => NodeProcessOutcome::Success,
                node_id(BAR_2_ID) => NodeProcessOutcome::Success,
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
    let node_set = TestNodeSet::new(vec![
        foo_node(FOO_1_ID, &[]),
        foo_node(FOO_2_ID, &[]),
        bar_node(BAR_1_ID, &[FOO_1_ID]),
        bar_node(BAR_2_ID, &[FOO_2_ID]),
    ]);

    // GIVEN: inputs() is called for each node, resulting in the dependencies spcified in the
    // specs:
    // BAR_1_ID -> FOO_1_ID
    // BAR_2_ID -> FOO_2_ID
    sys.expect_inputs().returning_st(|node| node.deps());

    let mut process_sequence = mockall::Sequence::new();
    // GIVEN: expected call for FOO_1_ID and FOO_2_ID.
    expect_process_multiple(&mut process_sequence, &mut sys, &[FOO_1_ID, FOO_2_ID]);
    // GIVEN: expected call for BAR_1_ID and BAR_2_ID.
    expect_process_multiple(&mut process_sequence, &mut sys, &[BAR_1_ID, BAR_2_ID]);

    let sys = Rc::new(sys);
    let processor = TestProcessor::new(sys.clone());
    let args = processargs::ArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    // THEN: the outcome reflects the successfully processed nodes.
    expect_that!(
        outcome,
        eq(&processing::ProcessOutcome {
            node_outcomes: hash_map! {
                node_id(FOO_1_ID) => NodeProcessOutcome::Success,
                node_id(FOO_2_ID) => NodeProcessOutcome::Success,
                node_id(BAR_1_ID) => NodeProcessOutcome::Success,
                node_id(BAR_2_ID) => NodeProcessOutcome::Success,
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
    let node_set = TestNodeSet::new(vec![foo_node(FOO_1_ID, &[FOO_1_ID])]);

    // GIVEN: inputs() is called for each node, resulting in the dependencies spcified in the
    // specs: FOO_1_ID -> FOO_1_ID
    sys.expect_inputs().returning_st(|node| node.deps());

    // GIVEN: No expected processing calls.

    let sys = Rc::new(sys);
    let processor = TestProcessor::new(sys.clone());
    let args = processargs::ArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    // THEN: the outcome reflects the unprocessed dependency loop.
    expect_that!(
        outcome,
        eq(&processing::ProcessOutcome {
            node_outcomes: hash_map! {
                node_id(FOO_1_ID) => NodeProcessOutcome::Unprocessed(NodeUnprocessedReason {
                    unprocessed_dependencies: hash_map! {
                        node_id(FOO_1_ID) => UnprocessedDependencyReason::Unprocessed,
                    }
                }),
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
    let node_set = TestNodeSet::new(vec![
        foo_node(FOO_1_ID, &[BAR_1_ID]),
        bar_node(BAR_1_ID, &[FOO_1_ID]),
    ]);

    // GIVEN: inputs() is called for each node, resulting in the dependencies spcified in the
    // specs:
    // BAR_1_ID -> FOO_1_ID -> BAR_1_ID
    sys.expect_inputs().returning_st(|node| node.deps());

    // GIVEN: No expected processing calls.

    let sys = Rc::new(sys);
    let processor = TestProcessor::new(sys.clone());
    let args = processargs::ArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    // THEN: the outcome reflects the unprocessed dependency loop.
    expect_that!(
        outcome,
        eq(&processing::ProcessOutcome {
            node_outcomes: hash_map! {
                node_id(FOO_1_ID) => NodeProcessOutcome::Unprocessed(NodeUnprocessedReason {
                    unprocessed_dependencies: hash_map! {
                        node_id(BAR_1_ID) => UnprocessedDependencyReason::Unprocessed,
                    }
                }),
                node_id(BAR_1_ID) => NodeProcessOutcome::Unprocessed(NodeUnprocessedReason {
                    unprocessed_dependencies: hash_map! {
                        node_id(FOO_1_ID) => UnprocessedDependencyReason::Unprocessed,
                    }
                }),
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
    let node_set = TestNodeSet::new(vec![foo_node(FOO_1_ID, &[UNKNOWN_ID])]);

    // GIVEN: inputs() is called for each node, resulting in the dependencies spcified in the
    // specs: FOO_1_ID -> "unknown-id"
    sys.expect_inputs().returning_st(|node| node.deps());

    // GIVEN: No expected processing calls.

    let sys = Rc::new(sys);
    let processor = TestProcessor::new(sys.clone());
    let args = processargs::ArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    expect_that!(
        outcome,
        eq(&processing::ProcessOutcome {
            node_outcomes: hash_map! {
                node_id(FOO_1_ID) => NodeProcessOutcome::Unprocessed(NodeUnprocessedReason {
                    unprocessed_dependencies: hash_map! {
                        node_id(UNKNOWN_ID) => UnprocessedDependencyReason::Unknown,
                    }
                }),
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
    let node_set = TestNodeSet::new(vec![foo_node(FOO_1_ID, &[])]);

    // GIVEN: inputs() is called for each node.
    sys.expect_inputs().returning_st(|node| node.deps());

    // GIVEN: A process_multiple call that fails with an error.
    sys.expect_process_multiple()
        .with(
            ProcessNodesPredicate::new().with_node_ids(&[FOO_1_ID]),
            always(),
            always(),
        )
        .returning_st(|_, _, _| vec![(node_id(FOO_1_ID), Err(anyhow!("some error")))]);

    let sys = Rc::new(sys);
    let processor = TestProcessor::new(sys.clone());
    let args = processargs::ArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    expect_that!(
        outcome,
        eq(&processing::ProcessOutcome {
            node_outcomes: hash_map! {
                node_id(FOO_1_ID) => NodeProcessOutcome::ProcessErrored(
                    anyhow::Error::msg("error message, content does not matter"),
                ),
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
    let node_set = TestNodeSet::new(vec![
        foo_node(FOO_1_ID, &[]),
        bar_node(BAR_1_ID, &[FOO_1_ID]),
    ]);

    // GIVEN: inputs() is called for each node, resulting in the dependencies spcified in the
    // specs:
    sys.expect_inputs().returning_st(|node| node.deps());

    // GIVEN: distinct process_multiple calls for foo_1_node followed by bar_1_node.
    sys.expect_process_multiple()
        .once()
        .with(
            ProcessNodesPredicate::new().with_node_ids(&[FOO_1_ID]),
            always(),
            always(),
        )
        .returning_st(|_, _, _| {
            vec![(
                node_id(FOO_1_ID),
                Ok(intermediates::Intermediate::JsonData(
                    serde_json::Value::String("some string".into()),
                )),
            )]
        });
    sys.expect_process_multiple()
        .once()
        .with(
            ProcessNodesPredicate::new().with_node_ids(&[BAR_1_ID]),
            always(),
            predicates::function::function(|interms: &IntermediateSet| {
                match interms.get(&node_id(FOO_1_ID)) {
                    Some(intermediates::Intermediate::JsonData(serde_json::Value::String(
                        str_value,
                    ))) => str_value == "some string",
                    _ => false,
                }
            }),
        )
        .returning_st(|_, _, _| vec![(node_id(BAR_1_ID), Ok(intermediates::Intermediate::NoData))]);

    let sys = Rc::new(sys);
    let processor = TestProcessor::new(sys.clone());
    let args = processargs::ArgSet::default();

    // WHEN: processing is requested on the nodes.
    let outcome = processor.process(&node_set, &args);

    // THEN:
    expect_that!(
        outcome,
        eq(&processing::ProcessOutcome {
            node_outcomes: hash_map! {
                node_id(FOO_1_ID) => NodeProcessOutcome::Success,
                node_id(BAR_1_ID) => NodeProcessOutcome::Success,
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

fn fake_process_multiple(
    nodes: &[&FakeNode],
) -> Vec<(node::NodeId, anyhow::Result<intermediates::Intermediate>)> {
    nodes
        .iter()
        .map(|node| (node.id.clone(), Ok(intermediates::Intermediate::NoData)))
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
