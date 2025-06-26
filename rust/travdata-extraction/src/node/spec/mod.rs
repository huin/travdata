pub mod es_transform;
pub mod input_pdf_file;
pub mod output_file_csv;
pub mod output_file_json;
pub mod pdf_extract_table;

use serde::{Deserialize, Serialize};

/// Per-type wrapper of a specific type of extraction configuration node.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, strum_macros::EnumDiscriminants)]
#[serde(tag = "type", content = "spec")]
pub enum Spec {
    EsTransform(es_transform::EsTransform),
    InputPdfFile(input_pdf_file::InputPdfFile),
    OutputFileCsv(output_file_csv::OutputFileCsv),
    OutputFileJson(output_file_json::OutputFileJson),
    PdfExtractTable(pdf_extract_table::PdfExtractTable),
}
