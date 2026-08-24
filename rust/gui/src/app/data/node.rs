//! [pipeline::generic] types specific to the GUI.

use pipeline::generic;
use serde::{Deserialize, Serialize};

/// Remains the same for the lifetime of a [NodeEditor]. Isn't displayed, but used in maintaining
/// consistent references to other nodes regardless of their [NodeEditor::id] changing.
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    PartialOrd,
    Ord,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct NodeRef(usize);

impl NodeRef {
    pub const MIN: NodeRef = NodeRef(usize::MIN);
    pub const MAX: NodeRef = NodeRef(usize::MAX);

    pub(crate) fn next_and_inc(&mut self) -> Result<Self, String> {
        let next = *self;
        self.0 = self.0.checked_add(1).ok_or("could not allocate NodeRef")?;
        Ok(next)
    }

    #[cfg(test)]
    pub(crate) fn next_and_inc_for_test(&mut self) -> Self {
        self.next_and_inc().expect("unexpectedly failed")
    }
}

impl hashbrown::Equivalent<NodeRef> for &NodeRef {
    fn equivalent(&self, key: &NodeRef) -> bool {
        *self == key
    }
}

/// NodeId type that may be resolved to an actual [Node] (via its NodeRef), or not (then the String
/// of the unresolved NodeId).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GuiNodeId {
    /// The [pipeline::NodeId] has not (yet) been resolved to a specific [NodeRef].
    ///
    /// This may happen temporarily during import of a [pipeline::Node], or for longer if the
    /// [pipeline::NodeId] does not exist or is ambigious (multiple nodes with same ID).
    Unresolved(String),
    /// The [pipeline::NodeId] has been resolved to a specific [NodeRef].
    Resolved(NodeRef),
}

impl From<pipeline::NodeId> for GuiNodeId {
    fn from(value: pipeline::NodeId) -> Self {
        Self::Unresolved(value.0)
    }
}

#[cfg(test)]
impl testutils::DefaultForTest for GuiNodeId {
    fn default_for_test() -> Self {
        GuiNodeId::Unresolved("unresolved-node-id".into())
    }
}

pub type GuiNode = generic::node::Node<GuiNodeId>;
pub type GuiNodeMeta = generic::node::NodeMeta<GuiNodeId>;
#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct GuiNodeWithId {
    pub node_id: String,
    pub node: GuiNode,
}
pub type GuiSpec = generic::specs::Spec<GuiNodeId>;
