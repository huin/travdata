use anyhow::{Context, Result};
use googletest::prelude::*;
use map_macro::{hash_set, hashbrown::hash_map};
use test_casing::{TestCases, cases, test_casing};

use super::*;
use crate::{Node, spec_types::pdf, testutil::*};

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
type: JsTransform
spec:
  context: js-context-id
  input_data:
    foo: thingy-1-extract
  code: |
    return foo.bar;
"#,
            Node{
                id: node_id("thingy-1-transform"),
                tags: Default::default(),
                public: false,
                spec: Spec::JsTransform(JsTransform {
                    context: node_id("js-context-id"),
                    input_data: hash_map! {
                        "foo".to_string() => node_id("thingy-1-extract"),
                    },
                    code: "return foo.bar;\n".to_string(),
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
  directory: output-directory
  filename: thingy-1.json
            "#,
            Node{
                id: node_id("thingy-1-json-out"),
                tags: hash_set!{
                    tag("format/json"),
                    tag("thingy-1"),
                },
                public: true,
                spec: Spec::OutputFileJson(OutputFileJson {
                    input_data: node_id("thingy-1-transform"),
                    directory: node_id("output-directory"),
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
  directory: output-directory
  filename: thingy-1.csv
            "#,
            Node{
                id: node_id("thingy-1-csv-out"),
                tags: hash_set!{
                    tag("format/csv"),
                    tag("thingy-1"),
                },
                public: true,
                spec: Spec::OutputFileCsv(OutputFileCsv {
                    input_data: node_id("thingy-1-transform"),
                    directory: node_id("output-directory"),
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
