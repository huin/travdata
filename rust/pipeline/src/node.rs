use strum::IntoDiscriminant as _;

use crate::specs;

#[derive(
    Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Deserialize, serde::Serialize,
)]
pub struct NodeId(pub String);

impl<S> From<S> for NodeId
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        Self(value.into())
    }
}

#[cfg(any(test, feature = "testing"))]
impl testutils::DefaultForTest for NodeId {
    fn default_for_test() -> Self {
        Self("default-node-id".into())
    }
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub struct Node {
    #[serde(flatten)]
    pub meta: NodeMeta,
    #[serde(flatten)]
    pub spec: specs::Spec,
}

impl generic_pipeline::systems::TypedNode for Node {
    type NodeType = specs::SpecDiscriminants;

    fn node_type(&self) -> Self::NodeType {
        self.spec.discriminant()
    }
}

/// Optional implementation of [generic_pipeline::PipelineNode].
impl generic_pipeline::PipelineNode for Node {
    type Id = NodeId;

    fn id(&self) -> &Self::Id {
        &self.meta.id
    }
}

#[cfg(any(test, feature = "testing"))]
impl testutils::DefaultForTest for Node {
    fn default_for_test() -> Self {
        Self {
            meta: NodeMeta::default_for_test(),
            spec: specs::Spec::default_for_test(),
        }
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

#[cfg(any(test, feature = "testing"))]
impl testutils::DefaultForTest for NodeMeta {
    fn default_for_test() -> Self {
        Self {
            id: NodeId::default_for_test(),
        }
    }
}
