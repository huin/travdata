use strum::IntoDiscriminant as _;

use crate::generic::specs;

/// Trait for transforming between `NodeId` types.
pub trait NodeIdTransformer {
    type FromNodeId;
    type ToNodeId;
    type Error;

    fn transform_node_id(&self, node_id: Self::FromNodeId) -> Result<Self::ToNodeId, Self::Error>;
}

/// Trait for [Node]s and their components to transform from one `NodeId` type to another.
pub trait TranslateFromNodeId<FromNodeId>: Sized {
    type FromType;
    type NodeId;

    fn transform_node_ids<
        Transformer: NodeIdTransformer<FromNodeId = FromNodeId, ToNodeId = Self::NodeId>,
    >(
        from: Self::FromType,
        trn: &Transformer,
    ) -> Result<Self, Transformer::Error>;
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub struct Node<NodeId> {
    #[serde(flatten)]
    pub meta: NodeMeta<NodeId>,
    #[serde(flatten)]
    pub spec: specs::Spec<NodeId>,
}

impl<FromNodeId, NodeId> TranslateFromNodeId<FromNodeId> for Node<NodeId> {
    type FromType = Node<FromNodeId>;
    type NodeId = NodeId;

    fn transform_node_ids<
        Transformer: NodeIdTransformer<FromNodeId = FromNodeId, ToNodeId = Self::NodeId>,
    >(
        from: Self::FromType,
        trn: &Transformer,
    ) -> Result<Self, Transformer::Error> {
        Ok(Self {
            meta: NodeMeta::transform_node_ids(from.meta, trn)?,
            spec: specs::Spec::transform_node_ids(from.spec, trn)?,
        })
    }
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

impl<FromNodeId, NodeId> TranslateFromNodeId<FromNodeId> for NodeMeta<NodeId> {
    type FromType = NodeMeta<FromNodeId>;
    type NodeId = NodeId;

    fn transform_node_ids<
        Transformer: NodeIdTransformer<FromNodeId = FromNodeId, ToNodeId = Self::NodeId>,
    >(
        from: Self::FromType,
        trn: &Transformer,
    ) -> Result<Self, Transformer::Error> {
        Ok(Self {
            id: trn.transform_node_id(from.id)?,
        })
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
