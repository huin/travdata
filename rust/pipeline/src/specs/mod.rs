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

use crate::{StringError, SystemError, SystemResult, impl_enum_conversions};
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
pub enum Spec {
    InputPdfFile(InputPdfFile),
    JsContext(JsContext),
    JsTransform(JsTransform),
    OutputDirectory(OutputDirectory),
    OutputFileCsv(OutputFileCsv),
    OutputFileJson(OutputFileJson),
    PdfExtractTable(PdfExtractTable),
}

impl Spec {
    pub fn downcast<'s, S>(&'s self) -> SystemResult<&'s S>
    where
        &'s S: TryFrom<&'s Spec, Error = StringError>,
    {
        self.try_into().map_err(SystemError::map_spec())
    }
}

impl generic_pipeline::systems::DiscriminatedSpec for Spec {
    type Discrim = SpecDiscriminants;

    fn discriminant(&self) -> Self::Discrim {
        self.into()
    }
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

impl_enum_conversions!(Spec, InputPdfFile, "node");
impl_enum_conversions!(Spec, JsContext, "node");
impl_enum_conversions!(Spec, JsTransform, "node");
impl_enum_conversions!(Spec, OutputDirectory, "node");
impl_enum_conversions!(Spec, OutputFileCsv, "node");
impl_enum_conversions!(Spec, OutputFileJson, "node");
impl_enum_conversions!(Spec, PdfExtractTable, "node");
