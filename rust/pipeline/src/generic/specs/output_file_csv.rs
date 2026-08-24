use serde::{Deserialize, Serialize};

#[cfg(any(test, feature = "testing"))]
use testutils::DefaultForTest;

use crate::generic::node::{NodeIdTransformer, TranslateFromNodeId};
use crate::spec_types;

/// Specifies output of CSV-encoded data to a file.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OutputFileCsv<NodeId> {
    pub input_data: NodeId,
    pub directory: NodeId,
    pub filename: spec_types::OutputPathBuf,
}

impl<FromNodeId, NodeId> TranslateFromNodeId<FromNodeId> for OutputFileCsv<NodeId> {
    type FromType = OutputFileCsv<FromNodeId>;
    type NodeId = NodeId;

    fn transform_node_ids<
        Transformer: NodeIdTransformer<FromNodeId = FromNodeId, ToNodeId = Self::NodeId>,
    >(
        from: Self::FromType,
        trn: &Transformer,
    ) -> Result<Self, Transformer::Error> {
        Ok(Self {
            input_data: trn.transform_node_id(from.input_data)?,
            directory: trn.transform_node_id(from.directory)?,
            filename: from.filename,
        })
    }
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
