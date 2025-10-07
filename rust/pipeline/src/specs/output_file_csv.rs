use serde::{Deserialize, Serialize};

use crate::spec_types;

/// Specifies output of CSV-encoded data to a file.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OutputFileCsv {
    pub input_data: crate::NodeId,
    pub directory: crate::NodeId,
    pub filename: spec_types::OutputPathBuf,
}
