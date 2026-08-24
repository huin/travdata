use serde::{Deserialize, Serialize};

#[cfg(any(test, feature = "testing"))]
use testutils::DefaultForTest;

use crate::spec_types;

/// Specifies output of CSV-encoded data to a file.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OutputFileCsv<NodeId> {
    pub input_data: NodeId,
    pub directory: NodeId,
    pub filename: spec_types::OutputPathBuf,
}

#[cfg(any(test, feature = "testing"))]
impl<NodeId> DefaultForTest for OutputFileCsv<NodeId>
where
    NodeId: DefaultForTest,
{
    fn default_for_test() -> Self {
        Self {
            input_data: NodeId::default_for_test(),
            directory: NodeId::default_for_test(),
            filename: spec_types::OutputPathBuf::default_for_test(),
        }
    }
}
