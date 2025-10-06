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

use anyhow::{Result, anyhow};
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

pub trait TryCastSpec<T> {
    fn try_cast_spec(&self) -> Result<&T>;
}

fn cast_error(spec: &Spec, expected_type_name: &str) -> anyhow::Error {
    anyhow!("node is not of type {}, got {:?}", expected_type_name, spec)
}

macro_rules! impl_try_cast_for {
    ($variant_and_type:ident) => {
        impl TryCastSpec<$variant_and_type> for Spec {
            fn try_cast_spec(&self) -> Result<&$variant_and_type> {
                match self {
                    Spec::$variant_and_type(spec) => Ok(spec),
                    _ => Err(cast_error(self, stringify!($variant_and_type))),
                }
            }
        }
    };
}

impl_try_cast_for!(InputPdfFile);
impl_try_cast_for!(JsContext);
impl_try_cast_for!(JsTransform);
impl_try_cast_for!(OutputDirectory);
impl_try_cast_for!(OutputFileCsv);
impl_try_cast_for!(OutputFileJson);
impl_try_cast_for!(PdfExtractTable);
