//! Concrete specialisations of [generic_pipeline::node::GenericNode]s.

pub mod es_transform;
pub mod input_pdf_file;
pub mod output_file_csv;
pub mod output_file_json;
pub mod pdf_extract_table;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

pub use es_transform::EsTransform;
pub use input_pdf_file::InputPdfFile;
pub use output_file_csv::OutputFileCsv;
pub use output_file_json::OutputFileJson;
pub use pdf_extract_table::PdfExtractTable;

/// Per-type wrapper of a specific type of extraction configuration node.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, strum_macros::EnumDiscriminants)]
#[strum_discriminants(derive(Hash))]
#[serde(tag = "type", content = "spec")]
pub enum Spec {
    EsTransform(EsTransform),
    InputPdfFile(InputPdfFile),
    OutputFileCsv(OutputFileCsv),
    OutputFileJson(OutputFileJson),
    PdfExtractTable(PdfExtractTable),
}

impl generic_pipeline::node::SpecTrait for Spec {
    type Discrim = SpecDiscriminants;

    fn discriminant(&self) -> Self::Discrim {
        self.into()
    }
}
