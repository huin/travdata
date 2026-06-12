use std::{path::Path, rc::Rc};

use anyhow::anyhow;
use generic_pipeline::{
    plparams::{ParamId, ParamKey},
    systems::GenericSystem,
};
use googletest::prelude::*;
use hashbrown::HashMap;
use map_macro::hashbrown::hash_map_e;
use serde_json::json;
use v8wrapper::TlsIsolate;

use crate::{
    MetaSystem, Node, PipelineTypes,
    plargs::{self, ArgSet, ArgValue},
    plparams::{Param, ParamType},
    spec_types::{OutputPathBuf, pdf},
    specs::{
        InputPdfFile, JsContext, OutputDirectory, OutputFileJson, PdfExtractTable, Spec,
        SpecDiscriminants,
    },
    systems,
    testutil::{self, node_id},
};
use testutils::{DefaultForTest, WrapError};

fn new_metasystem(tabula_extractor_fixture: &&testutil::TabulaExtractorFixture) -> MetaSystem {
    use crate::specs::SpecDiscriminants::*;

    let systems: HashMap<SpecDiscriminants, Rc<dyn GenericSystem<PipelineTypes>>> = hash_map_e! {
        InputPdfFile => Rc::new(systems::InputPdfFileSystem),
        JsContext => Rc::new(systems::JsContextSystem),
        JsTransform => Rc::new(systems::JsTransformSystem),
        OutputDirectory => Rc::new(systems::OutputDirectorySystem),
        OutputFileCsv => Rc::new(systems::OutputFileCsvSystem),
        OutputFileJson => Rc::new(systems::OutputFileJsonSystem),
        PdfExtractTable => Rc::new(systems::TabulaPdfExtractTableSystem::new(Box::new(tabula_extractor_fixture.client.clone()))),
    };

    MetaSystem::new(
        systems,
        Box::new(|discrim| anyhow!("no system found for {discrim:?}")),
    )
}

#[gtest]
fn test_e2e_small_pipeline(
    test_dir: testutil::TestDirectory,
    tabula_extractor_fixture: &&testutil::TabulaExtractorFixture,
) -> Result<()> {
    v8wrapper::init_v8_for_testing();
    let tls_isolate = TlsIsolate::for_current_thread().wrap_error()?;

    let system = new_metasystem(tabula_extractor_fixture);
    let processor = generic_pipeline::processing::GenericProcessor::new(Rc::new(system));

    let pipeline = crate::Pipeline::new(vec![
        Node {
            id: node_id("input-pdf"),
            spec: Spec::InputPdfFile(InputPdfFile {
                description: "Input PDF.".into(),
            }),
            ..DefaultForTest::default_for_test()
        },
        Node {
            id: node_id("js-ctx"),
            spec: Spec::JsContext(JsContext),
            ..DefaultForTest::default_for_test()
        },
        Node {
            id: node_id("output-dir"),
            spec: Spec::OutputDirectory(OutputDirectory {
                description: "Output directory.".into(),
            }),
            ..DefaultForTest::default_for_test()
        },
        Node {
            id: node_id("read-table-1"),
            spec: Spec::PdfExtractTable(PdfExtractTable {
                pdf: node_id("input-pdf"),
                page: 1,
                method: pdf::TabulaExtractionMethod::Lattice,
                rect: pdf::TabulaPdfRect {
                    left: 52.0.into(),
                    top: 88.0.into(),
                    right: (52.0 + 489.0).into(),
                    bottom: (88.0 + 67.0).into(),
                },
            }),
            ..DefaultForTest::default_for_test()
        },
        Node {
            id: node_id("output-table-1"),
            spec: Spec::OutputFileJson(OutputFileJson {
                input_data: node_id("read-table-1"),
                directory: node_id("output-dir"),
                filename: OutputPathBuf::new(Path::new("table-1.json")).wrap_error()?,
            }),
            ..DefaultForTest::default_for_test()
        },
    ]);

    let input_pdf_param_key = ParamKey::new(node_id("input-pdf"), ParamId::from_static("path"));
    let output_dir_param_key = ParamKey::new(node_id("output-dir"), ParamId::from_static("path"));

    let params = processor.resolve_params(&pipeline).wrap_error()?;
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
            ["r2c1", "r2c2", "r2c3"]
        ]))
    );

    drop(tls_isolate);

    Ok(())
}
