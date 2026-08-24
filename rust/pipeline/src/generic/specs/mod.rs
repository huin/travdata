//! Concrete specialisations of [generic_pipeline::node::GenericNode]s.

mod input_pdf_file;
mod js_context;
mod js_transform;
mod output_directory;
mod output_file_csv;
mod output_file_json;
mod pdf_extract_table;

use serde::{Deserialize, Serialize};

pub use input_pdf_file::InputPdfFile;
pub use js_context::JsContext;
pub use js_transform::JsTransform;
pub use output_directory::OutputDirectory;
pub use output_file_csv::OutputFileCsv;
pub use output_file_json::OutputFileJson;
pub use pdf_extract_table::PdfExtractTable;

/// Per-type wrapper of a specific type of extraction configuration node.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, strum_macros::EnumDiscriminants)]
#[strum_discriminants(derive(Hash, strum::VariantNames))]
#[serde(tag = "type", content = "spec")]
pub enum Spec<NodeId> {
    InputPdfFile(InputPdfFile),
    JsContext(JsContext),
    JsTransform(JsTransform<NodeId>),
    OutputDirectory(OutputDirectory),
    OutputFileCsv(OutputFileCsv<NodeId>),
    OutputFileJson(OutputFileJson<NodeId>),
    PdfExtractTable(PdfExtractTable<NodeId>),
}

impl strum::VariantMetadata for SpecDiscriminants {
    const VARIANT_COUNT: usize = <SpecDiscriminants as strum::VariantNames>::VARIANTS.len();
    const VARIANT_NAMES: &'static [&'static str] =
        <SpecDiscriminants as strum::VariantNames>::VARIANTS;

    fn variant_name(&self) -> &'static str {
        match self {
            SpecDiscriminants::InputPdfFile => "InputPdfFile",
            SpecDiscriminants::JsContext => "JsContext",
            SpecDiscriminants::JsTransform => "JsTransform",
            SpecDiscriminants::OutputDirectory => "OutputDirectory",
            SpecDiscriminants::OutputFileCsv => "OutputFileCsv",
            SpecDiscriminants::OutputFileJson => "OutputFileJson",
            SpecDiscriminants::PdfExtractTable => "PdfExtractTable",
        }
    }
}

#[cfg(any(test, feature = "testing"))]
impl<NodeId> testutils::DefaultForTest for Spec<NodeId>
where
    NodeId: testutils::DefaultForTest,
{
    fn default_for_test() -> Self {
        Spec::InputPdfFile::<NodeId>(InputPdfFile::default_for_test())
    }
}
