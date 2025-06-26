use std::{ffi::OsStr, path::Path};

use anyhow::{Context, Result};
use googletest::prelude::*;
use mapro::set;
use test_casing::{TestCases, cases, test_casing};

use super::{
    spec::{
        es_transform::EsTransform, input_pdf_file::InputPdfFile, output_file_csv::OutputFileCsv,
        output_file_json::OutputFileJson, pdf_extract_table::PdfExtractTable, *,
    },
    spec_type::pdf,
    *,
};

const CASES: TestCases<(&'static str, Node)> = cases! {
    [
        (
            r#"
id: foo-pdf
type: InputPdfFile
spec: {}
            "#,
            Node{
                id: node_id("foo-pdf"),
                tags: Default::default(),
                public: false,
                spec: Spec::InputPdfFile(InputPdfFile),
            },
        ),
        (
            r#"
id: thingy-1-extract
type: PdfExtractTable
spec:
  input_pdf: foo-pdf
  page: 123
  method: stream
  rect:
    left: 24.0
    top: 110.0
    right: 58.0
    bottom: 30.0
            "#,
            Node{
                id: node_id("thingy-1-extract"),
                tags: Default::default(),
                public: false,
                spec: Spec::PdfExtractTable(PdfExtractTable {
                    page: 123,
                    method: pdf::TabulaExtractionMethod::Stream,
                    rect: pdf::PdfRect {
                        left: pdf::PdfPoints::from_f32(24.0),
                        top: pdf::PdfPoints::from_f32(110.0),
                        right: pdf::PdfPoints::from_f32(58.0),
                        bottom: pdf::PdfPoints::from_f32(30.0),
                    },
                }),
            },
        ),
        (
r#"
id: thingy-1-transform
type: EsTransform
spec:
  input_data: thingy-1-extract
  code: |
    // Some code.
"#,
            Node{
                id: node_id("thingy-1-transform"),
                tags: Default::default(),
                public: false,
                spec: Spec::EsTransform(EsTransform {
                    input_data: node_id("thingy-1-extract"),
                    code: "// Some code.\n".to_string(),
                }),
            },
        ),
        (
            r#"
id: thingy-1-json-out
tags: [thingy-1, format/json]
type: OutputFileJson
public: true
spec:
  input_data: thingy-1-transform
  filename: thingy-1.json
            "#,
            Node{
                id: node_id("thingy-1-json-out"),
                tags: set!{
                    tag("format/json"),
                    tag("thingy-1"),
                },
                public: true,
                spec: Spec::OutputFileJson(OutputFileJson {
                    input_data: node_id("thingy-1-transform"),
                    filename: output_path_buf("thingy-1.json"),
                }),
            },
        ),
        (
            r#"
id: thingy-1-csv-out
tags: [thingy-1, format/csv]
type: OutputFileCsv
public: true
spec:
  input_data: thingy-1-transform
  filename: thingy-1.csv
            "#,
            Node{
                id: node_id("thingy-1-csv-out"),
                tags: set!{
                    tag("format/csv"),
                    tag("thingy-1"),
                },
                public: true,
                spec: Spec::OutputFileCsv(OutputFileCsv {
                    input_data: node_id("thingy-1-transform"),
                    filename: output_path_buf("thingy-1.csv"),
                }),
            },
        ),
    ]
};

#[test]
fn test_cases_len() {
    assert_eq!(5, CASES.into_iter().count());
}

#[test_casing(5, CASES)]
#[gtest]
fn test_reserialise_case(input: &'static str, expected: Node) -> Result<()> {
    let got_1: Node = serde_yaml_ng::from_str(input).context("deserialising original input")?;
    expect_that!(got_1, eq(&expected));

    let reserialised = serde_yaml_ng::to_string(&got_1).context("serialising got_1")?;
    let got_2: Node =
        serde_yaml_ng::from_str(&reserialised).context("deserialising reserialised data")?;
    expect_that!(got_2, eq(&expected));

    Ok(())
}

// This approach may never be used (might use a more compact representation than YAML), but for now
// keeping it as a reference example.
#[gtest]
fn test_deserialise_multi_doc() -> Result<()> {
    const INPUT: &str = r#"
id: foo-pdf
type: InputPdfFile
spec: {}
---
id: thingy-1-extract
type: PdfExtractTable
spec:
  input_pdf: foo-pdf
  page: 123
  method: stream
  rect:
    left: 24.0
    right: 58.0
    top: 110.0
    bottom: 30.0
---
id: thingy-1-transform
type: EsTransform
spec:
  input_data: thingy-1-extract
  code: |
    // Some code.
---
id: thingy-1-json-out
tags: [thingy-1, format/json]
type: OutputFileJson
public: true
spec:
  input_data: thingy-1-transform
  filename: thingy-1.json
---
id: thingy-1-csv-out
tags: [thingy-1, format/csv]
type: OutputFileCsv
public: true
spec:
  input_data: thingy-1-transform
  filename: thingy-1.csv
"#;

    for document in serde_yaml_ng::Deserializer::from_str(INPUT) {
        let _node = Node::deserialize(document)?;
    }

    Ok(())
}

fn node_id(s: &str) -> core_type::NodeId {
    s.to_string().try_into().expect("expected valid Id value")
}

fn output_path_buf<S: AsRef<OsStr> + ?Sized>(s: &S) -> core_type::OutputPathBuf {
    Path::new(s)
        .to_owned()
        .try_into()
        .expect("expected valid OutputPathBufValue")
}

fn tag(s: &str) -> core_type::Tag {
    s.to_string().try_into().expect("expected valid Tag value")
}
