use anyhow::Result;
use generic_pipeline::systems::GenericSystem;
use googletest::prelude::*;
use test_casing::{cases, test_casing};
use testutils::DefaultForTest;

use crate::{
    NodeId, intermediates, spec_types::pdf,
    tabula_wrapper::singlethreaded::SingleThreadedTabulaExtractor, testutil::node_id,
};

use super::*;

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

const NODE_CASES: test_casing::TestCases<NodeTestCase> = cases! {
    [
        NodeTestCase {
            skip: false,
            nodes: vec![
                Node {
                    id: node_id("node-one"),
                    spec: crate::specs::PdfExtractTable {
                        pdf: node_id("pdf-file"),
                        page: 1,
                        method: TabulaExtractionMethod::Lattice,
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
                        method: TabulaExtractionMethod::Stream,
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
                        method: TabulaExtractionMethod::Lattice,
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
    ]
};

#[test]
fn test_process_uses_intermediate_values_len() {
    assert_eq!(3, NODE_CASES.into_iter().count());
}

#[test_casing(3, NODE_CASES)]
#[gtest]
fn test_process_uses_intermediate_values(test_case: NodeTestCase) -> Result<()> {
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

    let mut interms = intermediates::IntermediateSet::new();
    interms.set(
        node_id("pdf-file"),
        intermediates::InputFile("./test_data/tables.pdf".into()).into(),
    );
    let got_results = system.process_multiple(&node_refs, &Default::default(), &interms);

    let got_interims: HashMap<NodeId, intermediates::IntermediateValue> = got_results
        .into_iter()
        .map(|node_result| Ok((node_result.id, node_result.value?)))
        .collect::<anyhow::Result<HashMap<_, _>>>()?;

    expect_that!(got_interims, eq(&expected_interms));

    Ok(())
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
