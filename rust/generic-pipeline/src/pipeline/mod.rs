use hashbrown::HashMap;

use crate::node;

/// Immutable set of [node::GenericNode]s, indexed for processing.
pub struct GenericPipeline<S> {
    id_to_node: HashMap<node::NodeId, node::GenericNode<S>>,
}

impl<S> GenericPipeline<S> {
    pub fn new(nodes: impl IntoIterator<Item = node::GenericNode<S>>) -> Self {
        let id_to_node = nodes
            .into_iter()
            .map(|node| (node.id.clone(), node))
            .collect();
        Self { id_to_node }
    }

    pub fn is_empty(&self) -> bool {
        self.id_to_node.is_empty()
    }

    pub fn len(&self) -> usize {
        self.id_to_node.len()
    }

    /// Returns an [Iterator] over all [node::GenericNode]s in the set.
    pub fn nodes(&self) -> impl Iterator<Item = &node::GenericNode<S>> {
        self.id_to_node.values()
    }

    /// Returns the [node::GenericNode] for the given [node::NodeId].
    pub fn get(&self, node_id: &node::NodeId) -> Option<&node::GenericNode<S>> {
        self.id_to_node.get(node_id)
    }
}
