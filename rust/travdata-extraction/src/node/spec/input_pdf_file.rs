use serde::{Deserialize, Serialize};

/// Specifies an input PDF file.
///
/// The actual file is set at the time of extraction.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InputPdfFile;
