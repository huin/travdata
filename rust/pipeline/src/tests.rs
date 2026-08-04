use std::{path::Path, rc::Rc};

use generic_pipeline::plparams::ParamId;
use googletest::prelude::*;
use hashbrown::HashMap;
use map_macro::hashbrown::{hash_map, hash_map_e};
use serde_json::json;

use crate::{
    Node, NodeMeta,
    monomorph::{ArgSet, MetaSystem, Param, ParamKey, Pipeline},
    plargs::{self, ArgValue},
    plparams::ParamType,
    spec_types::OutputPathBuf,
    specs::{InputPdfFile, JsContext, JsTransform, OutputDirectory, OutputFileJson, Spec},
    testutil,
};

fn new_metasystem(tabula_extractor_fixture: &&testutil::TabulaExtractorFixture) -> MetaSystem {
    crate::new_metasystem(Box::new(tabula_extractor_fixture.client.clone()))
}

#[gtest]
fn test_e2e_small_pipeline(
    _tls_isolate_fixture: &testutil::TlsIsolateFixture,
    test_dir: testutil::TestDirectory,
    tabula_extractor_fixture: &&testutil::TabulaExtractorFixture,
    test_data_tables: &&testutil::TestDataTables,
) -> Result<()> {
    let system = new_metasystem(tabula_extractor_fixture);
    let processor = generic_pipeline::processing::GenericProcessor::new(Rc::new(system));

    let pipeline = Pipeline::new(vec![
        Node {
            meta: NodeMeta::new("input-pdf"),
            spec: Spec::InputPdfFile(InputPdfFile {
                description: "Input PDF.".into(),
            }),
        },
        Node {
            meta: NodeMeta::new("js-ctx"),
            spec: Spec::JsContext(JsContext),
        },
        Node {
            meta: NodeMeta::new("output-dir"),
            spec: Spec::OutputDirectory(OutputDirectory {
                description: "Output directory.".into(),
            }),
        },
        Node {
            meta: NodeMeta::new("read-table-1"),
            spec: test_data_tables
                .table_1
                .to_pdf_extract_table("input-pdf")
                .into(),
        },
        Node {
            meta: NodeMeta::new("read-table-2"),
            spec: test_data_tables
                .table_2
                .to_pdf_extract_table("input-pdf")
                .into(),
        },
        Node {
            meta: NodeMeta::new("read-table-3-1"),
            spec: test_data_tables
                .table_3_1
                .to_pdf_extract_table("input-pdf")
                .into(),
        },
        Node {
            meta: NodeMeta::new("read-table-3-2"),
            spec: test_data_tables
                .table_3_2
                .to_pdf_extract_table("input-pdf")
                .into(),
        },
        Node {
            meta: NodeMeta::new("merge-table-3"),
            spec: JsTransform {
                context: "js-ctx".into(),
                input_data: hash_map! {
                    "part_1".into() => "read-table-3-1".into(),
                    "part_2".into() => "read-table-3-2".into(),
                },
                code: r#"
                    return part_1.concat(part_2);
                "#
                .into(),
            }
            .into(),
        },
        Node {
            meta: NodeMeta::new("output-table-1"),
            spec: Spec::OutputFileJson(OutputFileJson {
                input_data: "read-table-1".into(),
                directory: "output-dir".into(),
                filename: OutputPathBuf::new(Path::new("table-1.json"))?,
            }),
        },
        Node {
            meta: NodeMeta::new("output-table-2"),
            spec: Spec::OutputFileJson(OutputFileJson {
                input_data: "read-table-2".into(),
                directory: "output-dir".into(),
                filename: OutputPathBuf::new(Path::new("table-2.json"))?,
            }),
        },
        Node {
            meta: NodeMeta::new("output-table-3"),
            spec: Spec::OutputFileJson(OutputFileJson {
                input_data: "merge-table-3".into(),
                directory: "output-dir".into(),
                filename: OutputPathBuf::new(Path::new("table-3.json"))?,
            }),
        },
    ]);

    let input_pdf_param_key = ParamKey::new("input-pdf".into(), ParamId::from_static("path"));
    let output_dir_param_key = ParamKey::new("output-dir".into(), ParamId::from_static("path"));

    let params = processor.resolve_params(&pipeline)?;
    let expected_params: HashMap<ParamKey, Param> = hash_map_e! {
        input_pdf_param_key.clone() => Param {
            description: "Input PDF.".into(),
            param_type: ParamType::InputPdf,
        },
        output_dir_param_key.clone() => Param {
            description: "Output directory.".into(),
            param_type: ParamType::OutputDirectory,
        },
    };
    assert_that!(&params.params, eq(&expected_params));

    let args = {
        let mut args = ArgSet::default();
        args.set_key(
            input_pdf_param_key,
            ArgValue::InputPdf(plargs::InputPdf(Path::new("test_data/tables.pdf").into())),
        );
        args.set_key(
            output_dir_param_key,
            ArgValue::OutputDirectory(plargs::OutputDirectory(test_dir.path().into())),
        );
        args
    };

    let outcome = processor.process(&pipeline, &args);
    for (node, node_result) in outcome.node_results {
        expect_that!(node_result, ok(()), "for node {node:?}");
    }

    let table_1_json = std::fs::read_to_string(test_dir.path().join("table-1.json"))?;
    let table_1: serde_json::Value = serde_json::from_str(&table_1_json)?;

    expect_that!(
        &table_1,
        eq(&json!([
            ["Heading 1", "Heading 2", "Heading 3"],
            ["r1c1", "r1c2", "r1c3"],
            ["r2c1", "r2c2", "r2c3"],
        ]))
    );

    let table_2_json = std::fs::read_to_string(test_dir.path().join("table-2.json"))?;
    let table_2: serde_json::Value = serde_json::from_str(&table_2_json)?;

    expect_that!(
        &table_2,
        eq(&json!([
            ["Heading 1", "Heading 2", "Heading 3"],
            ["r1c1", "r1c2", "r1c3"],
            ["r2c1", "r2c2", "r2c3"],
        ]))
    );

    let table_3_json = std::fs::read_to_string(test_dir.path().join("table-3.json"))?;
    let table_3: serde_json::Value = serde_json::from_str(&table_3_json)?;

    expect_that!(
        &table_3,
        eq(&json!([
            ["Heading 1", "Heading 2", "Heading 3"],
            ["r1c1", "r1c2", "r1c3"],
            ["r2c1", "r2c2", "r2c3"],
            ["r3c1", "r3c2", "r3c3"],
            ["r4c1", "r4c2", "r4c3"],
        ]))
    );

    Ok(())
}
