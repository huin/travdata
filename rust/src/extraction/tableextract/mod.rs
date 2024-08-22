//! Extracts a single table from a PDF.

mod estransform;
pub mod groupers;
mod internal;
pub mod legacy_transform;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
/// Configures transformation of the raw data from Tabula into the output structured data.
pub enum TableTransform {
    LegacyTransformSeq(LegacyTransformSeq),
    ESTransform(ESTransform),
}

#[derive(Deserialize, Debug, Default)]
#[serde(transparent)]
/// Configures the specifics of extracting the CSV from the PDF.
pub struct LegacyTransformSeq {
    pub transforms: Vec<legacy_transform::TableTransform>,
}

#[derive(Deserialize, Debug)]
/// ECMAScript based table transformation.
pub struct ESTransform {
    pub src: String,
}
