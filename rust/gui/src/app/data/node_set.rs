use generic_pipeline::PipelineNode as _;
use hashbrown::HashMap;
use pipeline::NodeId;

use crate::app::{data::NodeRef, ddo};

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
        self.nodes.insert(node_ref, node);
        Ok(node_ref)
    }

    fn add_node_with_ref(&mut self, node: pipeline::Node, node_ref: NodeRef) {
        self.nodes.insert(node_ref, node);
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns a reference to a node, given its NodeRef ([[None]] if not exists).
    pub fn get(&self, node_ref: NodeRef) -> Option<&pipeline::Node> {
        self.nodes.get(&node_ref)
    }

    /// Returns a mutable reference to a node, given its NodeRef ([[None]] if not exists).
    pub fn get_mut(&mut self, node_ref: &NodeRef) -> Option<&mut pipeline::Node> {
        self.nodes.get_mut(&node_ref)
    }

    /// Returns an [Iterator] over all [NodeRef]s and their respective [pipeline::Node].
    pub fn iter(&self) -> impl Iterator<Item = (NodeRef, &pipeline::Node)> {
        self.nodes.iter().map(|(node_ref, node)| (*node_ref, node))
    }
}
