//! [pipeline::generic] types specific to the GUI.

use pipeline::generic;
use serde::{Deserialize, Serialize};

slotmap::new_key_type! {
    /// Remains the same for the lifetime of an [EditablePipeline]. Isn't displayed, but used in
    /// maintaining consistent references to other nodes regardless of their [NodeEditor::id]
    /// changing.
    pub struct NodeRef;
}

impl hashbrown::Equivalent<NodeRef> for &NodeRef {
    fn equivalent(&self, key: &NodeRef) -> bool {
        *self == key
    }
}

/// NodeId type that may be resolved to an actual [Node] (via its NodeRef), or not (then the String
/// of the unresolved NodeId).
#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
pub enum GuiNodeId {
    /// The [pipeline::NodeId] has not (yet) been resolved to a specific [NodeRef].
    ///
    /// This may happen temporarily during import of a [pipeline::Node], or for longer if the
    /// [pipeline::NodeId] does not exist or is ambigious (multiple nodes with same ID).
    Unresolved(String),
    /// The [pipeline::NodeId] has been resolved to a specific [NodeRef].
    Resolved(NodeRef),
}

impl GuiNodeId {
    pub fn is_resolved(&self) -> bool {
        matches!(self, Self::Resolved(_))
    }
}

impl From<pipeline::NodeId> for GuiNodeId {
    fn from(value: pipeline::NodeId) -> Self {
        Self::Unresolved(value.0)
    }
}

#[cfg(test)]
impl testutils::DefaultForTest for GuiNodeId {
    fn default_for_test() -> Self {
        use slotmap::Key as _;

        GuiNodeId::Resolved(NodeRef::null())
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
