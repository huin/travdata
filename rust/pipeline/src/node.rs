use strum::IntoDiscriminant as _;
#[cfg(test)]
use testutils::DefaultForTest;

use crate::specs::Spec;

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct NodeId(pub String);

impl<S> From<S> for NodeId
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        Self(value.into())
    }
}

/// Implementation of [generic_pipeline::PipelineNode].
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub struct Node {
    #[serde(flatten)]
    pub meta: NodeMeta,
    #[serde(flatten)]
    pub spec: Spec,
}

impl generic_pipeline::PipelineNode for Node {
    type Id = NodeId;

    fn id(&self) -> &Self::Id {
        &self.meta.id
    }
}

impl generic_pipeline::systems::TypedNode for Node {
    type NodeType = crate::specs::SpecDiscriminants;

    fn node_type(&self) -> Self::NodeType {
        self.spec.discriminant()
    }
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub struct NodeMeta {
    pub id: NodeId,
}

impl NodeMeta {
    pub fn new<Id>(id: Id) -> Self
    where
        Id: Into<NodeId>,
    {
        Self { id: id.into() }
    }
}

#[cfg(test)]
impl DefaultForTest for NodeMeta {
    fn default_for_test() -> Self {
        Self {
            id: "node-id".into(),
        }
    }
}
