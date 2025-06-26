use serde::{Deserialize, Serialize};

use crate::node::core_type;

/// Specifies output of JSON-encoded data to a file.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OutputFileJson {
    pub input_data: core_type::NodeId,
    pub filename: core_type::OutputPathBuf,
}
