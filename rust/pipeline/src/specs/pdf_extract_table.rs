use serde::{Deserialize, Serialize};
#[cfg(any(test, feature = "testing"))]
use testutils::DefaultForTest;

use crate::{NodeId, spec_types::pdf};

/// Specifies the extraction of a tabular region within a PDF file.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PdfExtractTable {
    pub pdf: NodeId,
    pub page: i32,
    pub method: pdf::TabulaExtractionMethod,
    pub rect: pdf::TabulaPdfRect,
}

#[cfg(any(test, feature = "testing"))]
impl DefaultForTest for PdfExtractTable {
    fn default_for_test() -> Self {
        Self {
            pdf: NodeId::default_for_test(),
            page: 1,
            method: pdf::TabulaExtractionMethod::Lattice,
            rect: pdf::TabulaPdfRect::default_for_test(),
        }
    }
}
