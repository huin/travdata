use serde::{Deserialize, Serialize};

/// Specifies directory for output files to be stored into.
///
/// Does not specify the directory itself, which is instead provided at the time of executing the
/// pipeline.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OutputDirectory {
    /// Human readable description of the directory to show to the user when prompted to choose an
    /// output directory.
    pub description: String,
}
