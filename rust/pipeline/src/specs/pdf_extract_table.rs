use serde::{Deserialize, Serialize};

use crate::spec_types::pdf;

/// Specifies the extraction of a tabular region within a PDF file.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PdfExtractTable {
    pub pdf: crate::NodeId,
    pub page: i32,
    pub method: pdf::TabulaExtractionMethod,
    pub rect: pdf::TabulaPdfRect,
}
