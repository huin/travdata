use googletest::prelude::*;
use map_macro::hashbrown::hash_map;
use test_casing::{TestCases, cases, test_casing};

use super::*;
use crate::{Node, NodeMeta, spec_types::pdf, testutil::*};

const CASES: TestCases<(&'static str, Node)> = cases! {
    [
        (
            r#"
id: foo-pdf
type: InputPdfFile
spec:
  description: input PDF file
            "#,
            Node{
                meta: NodeMeta::new("foo-pdf"),
                spec: Spec::InputPdfFile(InputPdfFile { description: "input PDF file".into() }),
            },
        ),
        (
            r#"
id: thingy-1-extract
type: PdfExtractTable
spec:
  pdf: foo-pdf
  page: 123
  method: stream
  rect:
    left: 24.0
    top: 110.0
    right: 58.0
    bottom: 30.0
            "#,
            Node{
                meta: NodeMeta::new("thingy-1-extract"),
                spec: Spec::PdfExtractTable(PdfExtractTable {
                    pdf: "foo-pdf".into(),
                    page: 123,
                    method: pdf::TabulaExtractionMethod::Stream,
                    rect: pdf::TabulaPdfRect {
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
                meta: NodeMeta::new("thingy-1-transform"),
                spec: Spec::JsTransform(JsTransform {
                    context: "js-context-id".into(),
                    input_data: hash_map! {
                        "foo".to_string() => "thingy-1-extract".into(),
                    },
                    code: "return foo.bar;\n".to_string(),
                }),
            },
        ),
        (
            r#"
id: thingy-1-json-out
type: OutputFileJson
spec:
  input_data: thingy-1-transform
  directory: output-directory
  filename: thingy-1.json
            "#,
            Node{
                meta: NodeMeta::new("thingy-1-json-out"),
                spec: Spec::OutputFileJson(OutputFileJson {
                    input_data: "thingy-1-transform".into(),
                    directory: "output-directory".into(),
                    filename: output_path_buf("thingy-1.json"),
                }),
            },
        ),
        (
            r#"
id: thingy-1-csv-out
type: OutputFileCsv
spec:
  input_data: thingy-1-transform
  directory: output-directory
  filename: thingy-1.csv
            "#,
            Node{
                meta: NodeMeta::new("thingy-1-csv-out"),
                spec: Spec::OutputFileCsv(OutputFileCsv {
                    input_data: "thingy-1-transform".into(),
                    directory: "output-directory".into(),
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
    let got_1: Node = serde_yaml_ng::from_str(input)?;
    expect_that!(got_1, eq(&expected));

    let reserialised = serde_yaml_ng::to_string(&got_1)?;
    let got_2: Node = serde_yaml_ng::from_str(&reserialised)?;
    expect_that!(got_2, eq(&expected));

    Ok(())
}
