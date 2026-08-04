use serde::{Deserialize, Serialize};
#[cfg(test)]
use testutils::DefaultForTest;

use crate::spec_types::pdf;

/// Specifies the extraction of a tabular region within a PDF file.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PdfExtractTable {
    pub pdf: crate::NodeId,
    pub page: i32,
    pub method: pdf::TabulaExtractionMethod,
    pub rect: pdf::TabulaPdfRect,
}

#[cfg(test)]
impl DefaultForTest for PdfExtractTable {
    fn default_for_test() -> Self {
        Self {
            pdf: "node-id".into(),
            page: 1,
            method: pdf::TabulaExtractionMethod::Lattice,
            rect: pdf::TabulaPdfRect::default_for_test(),
        }
    }
}
