use anyhow::Result;
use generic_pipeline::systems::GenericSystem;
use googletest::prelude::*;
use hashbrown::{HashMap, HashSet};
use test_casing::{cases, test_casing};
use testutils::DefaultForTest;

use crate::{
    Node, NodeId, intermediates, spec_types::pdf,
    tabula_wrapper::singlethreaded::SingleThreadedTabulaExtractor, testutil::node_id,
};

use super::TabulaPdfExtractTableSystem;

use lazy_static::lazy_static;

lazy_static! {
    static ref VM: anyhow::Result<tabula::TabulaVM> =
        tabula::TabulaVM::new("../target/debug/tabula.jar", true);
}

#[derive(Debug)]
struct NodeTestCase {
    skip: bool,
    nodes: Vec<Node>,
    expected_output: &'static [NodeOutput],
}

#[derive(Debug)]
struct NodeOutput {
    node_id: &'static str,
    data: &'static [&'static [&'static str]],
}

const EXTRACTS_TABLES_TEST_CASES: test_casing::TestCases<NodeTestCase> = cases! {
    [
        NodeTestCase {
            skip: false,
            nodes: vec![
                Node {
                    id: node_id("node-one"),
                    spec: crate::specs::PdfExtractTable {
                        pdf: node_id("pdf-file"),
                        page: 1,
                        method: pdf::TabulaExtractionMethod::Lattice,
                        rect: pdf::TabulaPdfRect {
                            left: 52.0.into(),
                            top: 88.0.into(),
                            right: (52.0 + 489.0).into(),
                            bottom: (88.0 + 67.0).into(),
                        },
                    }.into(),
                    ..DefaultForTest::default_for_test()
                },
            ],
            expected_output: &[
                NodeOutput {
                    node_id: "node-one",
                    data: &[
                        &["Heading 1", "Heading 2", "Heading 3"],
                        &["r1c1", "r1c2", "r1c3"],
                        &["r2c1", "r2c2", "r2c3"],
                    ],
                },
            ],
        },
        NodeTestCase {
            skip: false,
            nodes: vec![
                Node {
                    id: node_id("node-one"),
                    spec: crate::specs::PdfExtractTable {
                        pdf: node_id("pdf-file"),
                        page: 1,
                        method: pdf::TabulaExtractionMethod::Stream,
                        rect: pdf::TabulaPdfRect {
                            left: 52.0.into(),
                            top: 186.0.into(),
                            right: (52.0 + 489.0).into(),
                            bottom: (186.0 + 67.0).into(),
                        },
                    }.into(),
                    ..DefaultForTest::default_for_test()
                },
            ],
            expected_output: &[
                NodeOutput {
                    node_id: "node-one",
                    data: &[
                        &["Heading 1", "Heading 2", "Heading 3"],
                        &["r1c1", "r1c2", "r1c3"],
                        &["r2c1", "r2c2", "r2c3"],
                    ],
                },
            ],
        },
        NodeTestCase {
            // TODO: fix this case.
            skip: true,
            nodes: vec![
                Node {
                    id: node_id("node-one"),
                    spec: crate::specs::PdfExtractTable {
                        pdf: node_id("pdf-file"),
                        page: 1,
                        method: pdf::TabulaExtractionMethod::Lattice,
                        rect: pdf::TabulaPdfRect {
                            left: 52.0.into(),
                            top: 275.0.into(),
                            right: (52.0 + 489.0).into(),
                            bottom: (275.0 + 149.0).into(),
                        },
                    }.into(),
                    ..DefaultForTest::default_for_test()
                },
            ],
            expected_output: &[
                NodeOutput {
                    node_id: "node-one",
                    data: &[
                        &["Heading 1", "Heading 2", "Heading 3"],
                        &["r1c1", "r1c2", "r1c3"],
                        &["r2c1", "r2c2", "r2c3"],
                        &["r3c1", "r3c2", "r3c3"],
                        &["r4c1", "r4c2", "r4c3"],
                    ],
                },
            ],
        },
        NodeTestCase {
            // TODO: fix this case.
            skip: true,
            nodes: vec![
                Node {
                    id: node_id("first-table-only"),
                    spec: crate::specs::PdfExtractTable {
                        pdf: node_id("pdf-file"),
                        page: 1,
                        method: pdf::TabulaExtractionMethod::Lattice,
                        rect: pdf::TabulaPdfRect {
                            left: 52.0.into(),
                            top: 275.0.into(),
                            right: (52.0 + 489.0).into(),
                            bottom: (275.0 + 65.0).into(),
                        },
                    }.into(),
                    ..DefaultForTest::default_for_test()
                },
                Node {
                    id: node_id("both-tables"),
                    spec: crate::specs::PdfExtractTable {
                        pdf: node_id("pdf-file"),
                        page: 1,
                        method: pdf::TabulaExtractionMethod::Lattice,
                        rect: pdf::TabulaPdfRect {
                            left: 52.0.into(),
                            top: 275.0.into(),
                            right: (52.0 + 489.0).into(),
                            bottom: (275.0 + 149.0).into(),
                        },
                    }.into(),
                    ..DefaultForTest::default_for_test()
                },
            ],
            expected_output: &[
                NodeOutput {
                    node_id: "first-table-only",
                    data: &[
                        &["Heading 1", "Heading 2", "Heading 3"],
                        &["r1c1", "r1c2", "r1c3"],
                        &["r2c1", "r2c2", "r2c3"],
                    ],
                },
                NodeOutput {
                    node_id: "both-tables",
                    data: &[
                        &["Heading 1", "Heading 2", "Heading 3"],
                        &["r1c1", "r1c2", "r1c3"],
                        &["r2c1", "r2c2", "r2c3"],
                        &["r3c1", "r3c2", "r3c3"],
                        &["r4c1", "r4c2", "r4c3"],
                    ],
                },
            ],
        },
        // TODO: Add test case(s) with overlapping regions.
    ]
};

#[test]
fn test_multi_process_extracts_tables_len() {
    assert_eq!(4, EXTRACTS_TABLES_TEST_CASES.into_iter().count());
}

#[test_casing(4, EXTRACTS_TABLES_TEST_CASES)]
#[gtest]
fn test_multi_process_extracts_tables(test_case: NodeTestCase) -> Result<()> {
    if test_case.skip {
        return Ok(());
    }

    let vm = VM.as_ref().unwrap();
    let env = vm.attach()?;
    let extractor = SingleThreadedTabulaExtractor::new(env);
    let system = TabulaPdfExtractTableSystem::new(&extractor);

    let node_refs: Vec<&Node> = test_case.nodes.iter().collect();

    let expected_interms: HashMap<NodeId, intermediates::IntermediateValue> = test_case
        .expected_output
        .iter()
        .map(|node_output| {
            (
                node_id(node_output.node_id),
                intermediates::JsonData(table_slice_to_to_json_value(node_output.data)).into(),
            )
        })
        .collect();

    let interms = test_data_interms();
    let got_results = system.process_multiple(&node_refs, &Default::default(), &interms);

    let got_interims: HashMap<NodeId, intermediates::IntermediateValue> = got_results
        .into_iter()
        .map(|node_result| Ok((node_result.id, node_result.value?)))
        .collect::<anyhow::Result<HashMap<_, _>>>()?;

    expect_that!(got_interims, eq(&expected_interms));

    Ok(())
}

#[derive(Debug)]
struct NodeErrorTestCase {
    skip: bool,
    nodes: Vec<Node>,
    expected_errors: &'static [NodeExpectError],
}

#[derive(Debug)]
struct NodeExpectError {
    node_id: &'static str,
    error_contains: Option<&'static str>,
}

const EXTRACTS_TABLES_ERROR_TEST_CASES: test_casing::TestCases<NodeErrorTestCase> = cases! {
    [
        NodeErrorTestCase {
            // TODO: fix this case.
            skip: true,
            nodes: vec![
                Node {
                    id: node_id("no-tables-in-region"),
                    spec: crate::specs::PdfExtractTable {
                        pdf: node_id("pdf-file"),
                        page: 1,
                        method: pdf::TabulaExtractionMethod::Lattice,
                        rect: pdf::TabulaPdfRect {
                            left: 52.0.into(),
                            top: 27.0.into(),
                            right: (52.0 + 489.0).into(),
                            bottom: (27.0 + 22.0).into(),
                        },
                    }.into(),
                    ..DefaultForTest::default_for_test()
                },
            ],
            expected_errors: &[
                NodeExpectError {
                    node_id: "no-tables-in-region",
                    error_contains: Some("no table data in region"),
                },
            ],
        },
        NodeErrorTestCase {
            // TODO: fix this case.
            skip: true,
            nodes: vec![
                Node {
                    id: node_id("two-tables-in-region"),
                    spec: crate::specs::PdfExtractTable {
                        pdf: node_id("pdf-file"),
                        page: 1,
                        method: pdf::TabulaExtractionMethod::Lattice,
                        rect: pdf::TabulaPdfRect {
                            left: 52.0.into(),
                            top: 275.0.into(),
                            right: (52.0 + 489.0).into(),
                            bottom: (275.0 + 149.0).into(),
                        },
                    }.into(),
                    ..DefaultForTest::default_for_test()
                },
            ],
            expected_errors: &[
                NodeExpectError {
                    node_id: "two-tables-in-region",
                    error_contains: Some("no table data in region"),
                },
            ],
        },
        NodeErrorTestCase {
            // TODO: fix this case.
            skip: true,
            nodes: vec![
                Node {
                    id: node_id("no-tables-in-region"),
                    spec: crate::specs::PdfExtractTable {
                        pdf: node_id("pdf-file"),
                        page: 1,
                        method: pdf::TabulaExtractionMethod::Lattice,
                        rect: pdf::TabulaPdfRect {
                            left: 52.0.into(),
                            top: 27.0.into(),
                            right: (52.0 + 489.0).into(),
                            bottom: (27.0 + 22.0).into(),
                        },
                    }.into(),
                    ..DefaultForTest::default_for_test()
                },
                Node {
                    id: node_id("two-tables-in-region"),
                    spec: crate::specs::PdfExtractTable {
                        pdf: node_id("pdf-file"),
                        page: 1,
                        method: pdf::TabulaExtractionMethod::Lattice,
                        rect: pdf::TabulaPdfRect {
                            left: 52.0.into(),
                            top: 275.0.into(),
                            right: (52.0 + 489.0).into(),
                            bottom: (275.0 + 149.0).into(),
                        },
                    }.into(),
                    ..DefaultForTest::default_for_test()
                },
            ],
            expected_errors: &[
                NodeExpectError {
                    node_id: "no-tables-in-region",
                    error_contains: Some("no table data in region"),
                },
                NodeExpectError {
                    node_id: "two-tables-in-region",
                    error_contains: Some("no table data in region"),
                },
            ],
        },
        // TODO: Add test case(s) with overlapping regions.
    ]
};

#[test]
fn test_multi_process_errors_when_multiple_tables_match_len() {
    assert_eq!(3, EXTRACTS_TABLES_ERROR_TEST_CASES.into_iter().count());
}

#[test_casing(3, EXTRACTS_TABLES_ERROR_TEST_CASES)]
#[gtest]
fn test_multi_process_errors(test_case: NodeErrorTestCase) -> Result<()> {
    if test_case.skip {
        return Ok(());
    }

    let vm = VM.as_ref().unwrap();
    let env = vm.attach()?;
    let extractor = SingleThreadedTabulaExtractor::new(env);

    let system = TabulaPdfExtractTableSystem::new(&extractor);

    let node_refs: Vec<&Node> = test_case.nodes.iter().collect();

    let expected_node_ids: HashSet<NodeId> = test_case
        .expected_errors
        .iter()
        .map(|node_error| node_id(node_error.node_id))
        .collect();

    let interms = test_data_interms();
    let got_results = system.process_multiple(&node_refs, &Default::default(), &interms);

    let got_mapped_results: HashMap<NodeId, Result<intermediates::IntermediateValue>> = got_results
        .into_iter()
        .map(|node_result| (node_result.id, node_result.value))
        .collect();

    let got_node_ids: HashSet<NodeId> = got_mapped_results.keys().cloned().collect();
    expect_that!(got_node_ids, eq(&expected_node_ids));

    for expected_node_error in test_case.expected_errors {
        let node_id = node_id(expected_node_error.node_id);
        match (
            expected_node_error.error_contains,
            got_mapped_results.get(&node_id),
        ) {
            (Some(expected_error_contains), Some(Err(got_error))) => {
                expect_that!(
                    got_error.to_string(),
                    contains_substring(expected_error_contains),
                    "for node_id {:?}",
                    node_id,
                );
            }
            (Some(_), Some(Ok(got_value))) => {
                expect_true!(
                    false,
                    "for node_id {:?}: produced unexpected success value {:?}",
                    node_id,
                    got_value,
                );
            }
            (None, Some(Err(got_error))) => {
                expect_true!(
                    false,
                    "for node_id {:?}: produced unexpected error {:?}",
                    node_id,
                    got_error,
                );
            }
            (None, Some(Ok(_))) => {
                // Expected success.
            }
            (_, None) => {
                // Failure case covered by checking presence of ID set.
            }
        }
    }

    Ok(())
}

fn test_data_interms() -> intermediates::IntermediateSet {
    let mut interms = intermediates::IntermediateSet::new();
    interms.set(
        node_id("pdf-file"),
        intermediates::InputFile("./test_data/tables.pdf".into()).into(),
    );
    interms
}

fn table_slice_to_to_json_value(table_slice: &[&[&str]]) -> serde_json::Value {
    serde_json::Value::Array(
        table_slice
            .iter()
            .map(|&static_row| {
                static_row
                    .iter()
                    .map(|&static_field| serde_json::Value::String(static_field.into()))
                    .collect()
            })
            .collect(),
    )
}
