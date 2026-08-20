use hashbrown::HashMap;

use crate::app::data::NodeRef;

/// An unordered collection of [pipeline::Node]s, each indexed by a generated [NodeRef].
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct NodeSet {
    next_node_ref: NodeRef,
    nodes: HashMap<NodeRef, pipeline::Node>,
}

impl NodeSet {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            next_node_ref: NodeRef::default(),
            nodes: HashMap::with_capacity(capacity),
        }
    }

    /// Takes ownership of the node, returning a [NodeRef] for access it later.
    pub fn add_node(&mut self, node: pipeline::Node) -> Result<NodeRef, String> {
        let node_ref = self.next_node_ref.next_and_inc()?;
        self.add_node_with_ref(node, node_ref);
        Ok(node_ref)
    }

    fn add_node_with_ref(&mut self, node: pipeline::Node, node_ref: NodeRef) {
        self.nodes.insert(node_ref, node);
    }

    /// Returns a reference to a node, given its NodeRef ([[None]] if not exists).
    pub fn get(&self, node_ref: NodeRef) -> Option<&pipeline::Node> {
        self.nodes.get(&node_ref)
    }

    /// Returns a mutable reference to a node, given its NodeRef ([[None]] if not exists).
    pub fn get_mut(&mut self, node_ref: NodeRef) -> Option<&mut pipeline::Node> {
        self.nodes.get_mut(&node_ref)
    }
}
