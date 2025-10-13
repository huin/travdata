use serde::{Deserialize, Serialize};

/// Specifies an input PDF file.
///
/// The actual file is set at the time of extraction.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InputPdfFile {
    /// Human readable description of the PDF file to show to the user when prompted to choose an
    /// input PDF.
    pub description: String,
}
