use anyhow::Result;
use generic_pipeline::systems::GenericSystem;
use googletest::prelude::*;
use hashbrown::HashMap;
use test_casing::{TestCases, cases, test_casing};
use testutils::DefaultForTest;

use crate::{
    Node, NodeId, intermediates,
    spec_types::pdf,
    specs,
    systems::tabula_pdf_extract_table::grouped_non_overlapping_slices,
    tabula_wrapper::singlethreaded::SingleThreadedTabulaExtractor,
    testutil::{MatcherBox, NodeExpected, check_results, node_id},
};

use super::{NodeSpec, TabulaPdfExtractTableSystem};

use lazy_static::lazy_static;

lazy_static! {
    static ref VM: Result<tabula::TabulaVM> =
        tabula::TabulaVM::new("../target/debug/tabula.jar", true);
}

#[gtest]
fn test_extracts_single_table_lattice() -> Result<()> {
    let node_expecteds = vec![NodeExpected {
        node: Node {
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
            }
            .into(),
            ..DefaultForTest::default_for_test()
        },
        expected: MatcherBox::new(ok(eq(table_slice_to_to_intermediates_json_data(&[
            &["Heading 1", "Heading 2", "Heading 3"],
            &["r1c1", "r1c2", "r1c3"],
            &["r2c1", "r2c2", "r2c3"],
        ])))),
    }];

    let actual_results_map = do_multi_process(&node_expecteds)?;
    check_results(&actual_results_map, node_expecteds);
    Ok(())
}

#[gtest]
fn test_extracts_single_table_stream() -> Result<()> {
    let node_expecteds = vec![NodeExpected {
        node: Node {
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
            }
            .into(),
            ..DefaultForTest::default_for_test()
        },
        expected: MatcherBox::new(ok(eq(table_slice_to_to_intermediates_json_data(&[
            &["Heading 1", "Heading 2", "Heading 3"],
            &["r1c1", "r1c2", "r1c3"],
            &["r2c1", "r2c2", "r2c3"],
        ])))),
    }];

    let actual_results_map = do_multi_process(&node_expecteds)?;
    check_results(&actual_results_map, node_expecteds);
    Ok(())
}

#[gtest]
fn test_rejects_lattice_in_two_parts() -> Result<()> {
    let node_expecteds = vec![NodeExpected {
        node: Node {
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
            }
            .into(),
            ..DefaultForTest::default_for_test()
        },
        expected: MatcherBox::new(err(displays_as(contains_substring(
            "multiple (2) tables in region",
        )))),
    }];

    let actual_results_map = do_multi_process(&node_expecteds)?;
    check_results(&actual_results_map, node_expecteds);
    Ok(())
}

#[gtest]
fn test_extracts_single_table_and_rejects_overlapping_split_table() -> Result<()> {
    let node_expecteds = vec![
        NodeExpected {
            node: Node {
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
                }
                .into(),
                ..DefaultForTest::default_for_test()
            },
            expected: MatcherBox::new(ok(eq(table_slice_to_to_intermediates_json_data(&[
                &["Heading 1", "Heading 2", "Heading 3"],
                &["r1c1", "r1c2", "r1c3"],
                &["r2c1", "r2c2", "r2c3"],
            ])))),
        },
        NodeExpected {
            node: Node {
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
                }
                .into(),
                ..DefaultForTest::default_for_test()
            },
            expected: MatcherBox::new(err(displays_as(contains_substring(
                "multiple (2) tables in region",
            )))),
        },
    ];

    let actual_results_map = do_multi_process(&node_expecteds)?;
    check_results(&actual_results_map, node_expecteds);
    Ok(())
}

#[gtest]
fn test_rejects_single_empty_region() -> Result<()> {
    let node_expecteds = vec![NodeExpected {
        node: Node {
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
            }
            .into(),
            ..DefaultForTest::default_for_test()
        },
        expected: MatcherBox::new(err(displays_as(contains_substring("no table in region")))),
    }];

    let actual_results_map = do_multi_process(&node_expecteds)?;
    check_results(&actual_results_map, node_expecteds);
    Ok(())
}

#[gtest]
fn test_rejects_single_region_with_multiple_tables() -> Result<()> {
    let node_expecteds = vec![NodeExpected {
        node: Node {
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
            }
            .into(),
            ..DefaultForTest::default_for_test()
        },
        expected: MatcherBox::new(err(displays_as(contains_substring(
            "multiple (2) tables in region",
        )))),
    }];

    let actual_results_map = do_multi_process(&node_expecteds)?;
    check_results(&actual_results_map, node_expecteds);
    Ok(())
}

#[gtest]
fn test_multiple_tables_with_overlaps() -> Result<()> {
    // All nodes extract different subsets of table 1.
    let node_expecteds = vec![
        NodeExpected {
            node: Node {
                id: node_id("table-1-complete"),
                spec: crate::specs::PdfExtractTable {
                    pdf: node_id("pdf-file"),
                    page: 1,
                    method: pdf::TabulaExtractionMethod::Lattice,
                    rect: pdf::TabulaPdfRect {
                        left: 52.0.into(),
                        top: 88.0.into(),
                        right: (52.0 + 489.0).into(),
                        bottom: (88.0 + 65.0).into(),
                    },
                }
                .into(),
                ..DefaultForTest::default_for_test()
            },
            expected: MatcherBox::new(ok(eq(table_slice_to_to_intermediates_json_data(&[
                &["Heading 1", "Heading 2", "Heading 3"],
                &["r1c1", "r1c2", "r1c3"],
                &["r2c1", "r2c2", "r2c3"],
            ])))),
        },
        NodeExpected {
            node: Node {
                id: node_id("table-1-headings-only"),
                spec: crate::specs::PdfExtractTable {
                    pdf: node_id("pdf-file"),
                    page: 1,
                    method: pdf::TabulaExtractionMethod::Lattice,
                    rect: pdf::TabulaPdfRect {
                        left: 52.0.into(),
                        top: 88.0.into(),
                        right: (52.0 + 489.0).into(),
                        bottom: (88.0 + 27.0).into(),
                    },
                }
                .into(),
                ..DefaultForTest::default_for_test()
            },
            expected: MatcherBox::new(ok(eq(table_slice_to_to_intermediates_json_data(&[&[
                "Heading 1",
                "Heading 2",
                "Heading 3",
            ]])))),
        },
        NodeExpected {
            node: Node {
                id: node_id("table-1-data-rows-only"),
                spec: crate::specs::PdfExtractTable {
                    pdf: node_id("pdf-file"),
                    page: 1,
                    method: pdf::TabulaExtractionMethod::Lattice,
                    rect: pdf::TabulaPdfRect {
                        left: 52.0.into(),
                        top: 109.0.into(),
                        right: (52.0 + 489.0).into(),
                        bottom: (109.0 + 46.0).into(),
                    },
                }
                .into(),
                ..DefaultForTest::default_for_test()
            },
            expected: MatcherBox::new(ok(eq(table_slice_to_to_intermediates_json_data(&[
                &["r1c1", "r1c2", "r1c3"],
                &["r2c1", "r2c2", "r2c3"],
            ])))),
        },
    ];

    let actual_results_map = do_multi_process(&node_expecteds)?;
    check_results(&actual_results_map, node_expecteds);
    Ok(())
}

#[gtest]
fn test_rejects_two_overlapping_regions_with_zero_and_two_tables_respectively() -> Result<()> {
    // The intent of this test is that the two tables matched by the second node do not get
    // mistakenly attributed one each to the nodes.
    let node_expecteds = vec![
        NodeExpected {
            node: Node {
                id: node_id("no-tables-in-region"),
                spec: crate::specs::PdfExtractTable {
                    pdf: node_id("pdf-file"),
                    page: 1,
                    method: pdf::TabulaExtractionMethod::Lattice,
                    rect: pdf::TabulaPdfRect {
                        left: 52.0.into(),
                        top: 275.0.into(),
                        right: (52.0 + 489.0).into(),
                        bottom: (275.0 + 22.0).into(),
                    },
                }
                .into(),
                ..DefaultForTest::default_for_test()
            },
            expected: MatcherBox::new(err(displays_as(contains_substring("no table in region")))),
        },
        NodeExpected {
            node: Node {
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
                }
                .into(),
                ..DefaultForTest::default_for_test()
            },
            expected: MatcherBox::new(err(displays_as(contains_substring(
                "multiple (2) tables in region",
            )))),
        },
    ];

    let actual_results_map = do_multi_process(&node_expecteds)?;
    check_results(&actual_results_map, node_expecteds);
    Ok(())
}

fn table_slice_to_to_intermediates_json_data(
    table_slice: &[&[&str]],
) -> intermediates::IntermediateValue {
    intermediates::JsonData(serde_json::Value::Array(
        table_slice
            .iter()
            .map(|&static_row| {
                static_row
                    .iter()
                    .map(|&static_field| serde_json::Value::String(static_field.into()))
                    .collect()
            })
            .collect(),
    ))
    .into()
}

fn do_multi_process<'a, 'm>(
    node_expecteds: &'m [NodeExpected<'a>],
) -> Result<HashMap<NodeId, Result<intermediates::IntermediateValue>>>
where
    'a: 'm,
{
    let vm = VM.as_ref().unwrap();
    let env = vm.attach()?;
    let extractor = SingleThreadedTabulaExtractor::new(env);
    let system = TabulaPdfExtractTableSystem::new(&extractor);

    let node_refs: Vec<&Node> = node_expecteds
        .iter()
        .map(|node_expected| &node_expected.node)
        .collect();

    let interms = test_data_interms();
    let actual_results = system.process_multiple(&node_refs, &Default::default(), &interms);

    let actual_results_map: HashMap<NodeId, Result<intermediates::IntermediateValue>> =
        actual_results
            .into_iter()
            .map(|node_result| (node_result.id, node_result.value))
            .collect::<HashMap<_, _>>();

    Ok(actual_results_map)
}

fn test_data_interms() -> intermediates::IntermediateSet {
    let mut interms = intermediates::IntermediateSet::new();
    interms.set(
        node_id("pdf-file"),
        intermediates::InputFile("./test_data/tables.pdf".into()).into(),
    );
    interms
}

const TEST_GROUPED_NON_OVERLAPPING_REGIONS_CASES: TestCases<[usize; 3]> = cases! {
    [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ]
};

#[test]
fn test_grouped_non_overlapping_regions_cases() {
    assert_eq!(
        6,
        TEST_GROUPED_NON_OVERLAPPING_REGIONS_CASES
            .into_iter()
            .count()
    );
}

#[test_casing(6, TEST_GROUPED_NON_OVERLAPPING_REGIONS_CASES)]
#[gtest]
fn test_grouped_non_overlapping_regions(node_order: [usize; 3]) -> Result<()> {
    let node_a_id = node_id("node-a");
    let node_a = Node {
        id: node_a_id.clone(),
        spec: crate::specs::PdfExtractTable {
            rect: pdf::TabulaPdfRect {
                left: 5.0.into(),
                top: 5.0.into(),
                right: 10.0.into(),
                bottom: 10.0.into(),
            },
            ..DefaultForTest::default_for_test()
        }
        .into(),
        ..DefaultForTest::default_for_test()
    };
    // node_b overlaps with node_a.
    let node_b_id = node_id("node-b");
    let node_b = Node {
        id: node_b_id.clone(),
        spec: crate::specs::PdfExtractTable {
            rect: pdf::TabulaPdfRect {
                left: 7.0.into(),
                top: 7.0.into(),
                right: 12.0.into(),
                bottom: 12.0.into(),
            },
            ..DefaultForTest::default_for_test()
        }
        .into(),
        ..DefaultForTest::default_for_test()
    };
    // node_c overlaps with neither.
    let node_c_id = node_id("node-c");
    let node_c = Node {
        id: node_c_id.clone(),
        spec: crate::specs::PdfExtractTable {
            rect: pdf::TabulaPdfRect {
                left: 5.0.into(),
                top: 20.0.into(),
                right: 10.0.into(),
                bottom: 25.0.into(),
            },
            ..DefaultForTest::default_for_test()
        }
        .into(),
        ..DefaultForTest::default_for_test()
    };

    let nodes = vec![node_a, node_b, node_c];
    let node_specs: Vec<NodeSpec> = node_order
        .iter()
        .map(|node_index| {
            let node = &nodes[*node_index];
            Ok(NodeSpec {
                node,
                spec: <&specs::PdfExtractTable>::try_from(&node.spec)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let mut all_node_ids_seen: Vec<NodeId> = Vec::new();
    let mut all_groups_seen: Vec<Vec<NodeId>> = Vec::new();
    grouped_non_overlapping_slices(node_specs, |node_specs_group| {
        let group: Vec<_> = node_specs_group
            .iter()
            .map(|node_spec| node_spec.node.id.clone())
            .collect();
        all_node_ids_seen.extend(group.iter().cloned());
        all_groups_seen.push(group);
    });

    expect_that!(
        all_node_ids_seen,
        unordered_elements_are!(eq(&node_a_id), eq(&node_b_id), eq(&node_c_id)),
        "expected nodes must appear exactly once each",
    );
    expect_that!(
        all_groups_seen,
        each(any!(
            all!(contains(eq(&node_a_id)), not(contains(eq(&node_b_id)))),
            all!(contains(eq(&node_b_id)), not(contains(eq(&node_a_id)))),
        )),
        "node-a and node-b cannot be in the same grouping",
    );
    expect_that!(
        all_groups_seen,
        contains(all!(contains(eq(&node_c_id)), not(len(lt(2))))),
        "node-c should always be grouped with another node for efficiency",
    );

    Ok(())
}
