//! Concrete specialisations of [generic_pipeline::node::GenericNode]s.

mod input_pdf_file;
mod js_context;
mod js_transform;
mod output_directory;
mod output_file_csv;
mod output_file_json;
mod pdf_extract_table;
#[cfg(test)]
mod test_defaults;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

pub use input_pdf_file::InputPdfFile;
pub use js_context::JsContext;
pub use js_transform::JsTransform;
pub use output_directory::OutputDirectory;
pub use output_file_csv::OutputFileCsv;
pub use output_file_json::OutputFileJson;
pub use pdf_extract_table::PdfExtractTable;

/// Per-type wrapper of a specific type of extraction configuration node.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, strum_macros::EnumDiscriminants)]
#[strum_discriminants(derive(Hash))]
#[serde(tag = "type", content = "spec")]
pub enum Spec {
    InputPdfFile(InputPdfFile),
    JsContext(JsContext),
    JsTransform(JsTransform),
    OutputDirectory(OutputDirectory),
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
