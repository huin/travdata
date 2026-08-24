use serde::{Deserialize, Serialize};
#[cfg(any(test, feature = "testing"))]
use testutils::DefaultForTest;

use crate::generic::node::{NodeIdTransformer, TranslateFromNodeId};
use crate::spec_types::pdf;

/// Specifies the extraction of a tabular region within a PDF file.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PdfExtractTable<NodeId> {
    pub pdf: NodeId,
    pub page: i32,
    pub method: pdf::TabulaExtractionMethod,
    pub rect: pdf::TabulaPdfRect,
}

impl<FromNodeId, NodeId> TranslateFromNodeId<FromNodeId> for PdfExtractTable<NodeId> {
    type FromType = PdfExtractTable<FromNodeId>;
    type NodeId = NodeId;

    fn transform_node_ids<
        Transformer: NodeIdTransformer<FromNodeId = FromNodeId, ToNodeId = Self::NodeId>,
    >(
        from: Self::FromType,
        trn: &Transformer,
    ) -> Result<Self, Transformer::Error> {
        Ok(Self {
            pdf: trn.transform_node_id(from.pdf)?,
            page: from.page,
            method: from.method,
            rect: from.rect,
        })
    }
}

#[cfg(any(test, feature = "testing"))]
impl<NodeId> DefaultForTest for PdfExtractTable<NodeId>
where
    NodeId: DefaultForTest,
{
    fn default_for_test() -> Self {
        Self {
            pdf: NodeId::default_for_test(),
            page: 1,
            method: pdf::TabulaExtractionMethod::Lattice,
            rect: pdf::TabulaPdfRect::default_for_test(),
        }
    }
}
