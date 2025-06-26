use serde::{Deserialize, Serialize};

use crate::node::spec_type::pdf;

/// Specifies the extraction of a tabular region within a Pdf file.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PdfExtractTable {
    pub page: i32,
    pub method: pdf::TabulaExtractionMethod,
    pub rect: pdf::PdfRect,
}
