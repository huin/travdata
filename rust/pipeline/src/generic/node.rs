use strum::IntoDiscriminant as _;

use crate::generic::specs;

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub struct Node<NodeId> {
    #[serde(flatten)]
    pub meta: NodeMeta<NodeId>,
    #[serde(flatten)]
    pub spec: specs::Spec<NodeId>,
}

impl<NodeId> generic_pipeline::systems::TypedNode for Node<NodeId> {
    type NodeType = specs::SpecDiscriminants;

    fn node_type(&self) -> Self::NodeType {
        self.spec.discriminant()
    }
}

/// Optional implementation of [generic_pipeline::PipelineNode].
impl<NodeId> generic_pipeline::PipelineNode for Node<NodeId>
where
    NodeId: generic_pipeline::PipelineNodeId,
{
    type Id = NodeId;

    fn id(&self) -> &Self::Id {
        &self.meta.id
    }
}

#[cfg(any(test, feature = "testing"))]
impl<NodeId> testutils::DefaultForTest for Node<NodeId>
where
    NodeId: testutils::DefaultForTest,
{
    fn default_for_test() -> Self {
        Self {
            meta: NodeMeta::default_for_test(),
            spec: specs::Spec::default_for_test(),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub struct NodeMeta<NodeId> {
    pub id: NodeId,
}

impl<NodeId> NodeMeta<NodeId> {
    pub fn new<Id>(id: Id) -> Self
    where
        Id: Into<NodeId>,
    {
        Self { id: id.into() }
    }
}

#[cfg(any(test, feature = "testing"))]
impl<NodeId> testutils::DefaultForTest for NodeMeta<NodeId>
where
    NodeId: testutils::DefaultForTest,
{
    fn default_for_test() -> Self {
        Self {
            id: NodeId::default_for_test(),
        }
    }
}
