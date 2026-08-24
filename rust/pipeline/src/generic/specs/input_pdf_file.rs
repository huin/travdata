use serde::{Deserialize, Serialize};
#[cfg(any(test, feature = "testing"))]
use testutils::DefaultForTest;

/// Specifies an input PDF file.
///
/// The actual file is set at the time of extraction.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InputPdfFile {
    /// Human readable description of the PDF file to show to the user when prompted to choose an
    /// input PDF.
    pub description: String,
}

#[cfg(any(test, feature = "testing"))]
impl DefaultForTest for InputPdfFile {
    fn default_for_test() -> Self {
        Self {
            description: "Test default InputPdfFile description.".into(),
        }
    }
}
