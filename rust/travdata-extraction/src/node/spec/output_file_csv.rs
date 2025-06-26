use serde::{Deserialize, Serialize};

use crate::node::core_type;

/// Specifies output of CSV-encoded data to a file.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OutputFileCsv {
    pub input_data: core_type::NodeId,
    pub filename: core_type::OutputPathBuf,
}
