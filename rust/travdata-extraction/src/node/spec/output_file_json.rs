use serde::{Deserialize, Serialize};

use crate::node::{self, spec_type};

/// Specifies output of JSON-encoded data to a file.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OutputFileJson {
    pub input_data: node::NodeId,
    pub filename: spec_type::OutputPathBuf,
}
