use crate::generic;

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

pub type Node = generic::node::Node<NodeId>;

pub type NodeMeta = generic::node::NodeMeta<NodeId>;
