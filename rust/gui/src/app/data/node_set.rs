use hashbrown::HashMap;

use crate::app::data::{NodeRef, node::GuiNodeWithId};

/// An unordered collection of [pipeline::Node]s, each indexed by a generated [NodeRef].
#[derive(Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct NodeSet {
    next_node_ref: NodeRef,
    nodes: HashMap<NodeRef, GuiNodeWithId>,
}

impl NodeSet {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            next_node_ref: NodeRef::default(),
            nodes: HashMap::with_capacity(capacity),
        }
    }

    /// Takes ownership of the node, returning a [NodeRef] for access it later.
    pub fn add_node(&mut self, node: GuiNodeWithId) -> Result<NodeRef, String> {
        let node_ref = self.next_node_ref.next_and_inc()?;
        self.add_node_with_ref(node, node_ref);
        Ok(node_ref)
    }

    fn add_node_with_ref(&mut self, node: GuiNodeWithId, node_ref: NodeRef) {
        self.nodes.insert(node_ref, node);
    }

    /// Returns a reference to a node, given its [NodeRef] ([None] if not exists).
    pub fn get(&self, node_ref: NodeRef) -> Option<&GuiNodeWithId> {
        self.nodes.get(&node_ref)
    }

    pub(crate) fn take(&mut self, node_ref: NodeRef) -> Option<GuiNodeWithId> {
        self.nodes.remove_entry(&node_ref).map(|(_k, v)| v)
    }

    /// Returns a mutable reference to a node, given its [NodeRef] ([None] if not exists).
    pub fn get_mut(&mut self, node_ref: NodeRef) -> Option<&mut GuiNodeWithId> {
        self.nodes.get_mut(&node_ref)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut GuiNodeWithId> {
        self.nodes.values_mut()
    }
}
