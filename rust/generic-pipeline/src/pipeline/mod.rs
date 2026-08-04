use hashbrown::HashMap;

use crate::{PipelineTypes, node::PipelineNode};

/// Immutable set of [crate::node::PipelineNode]s, indexed for processing.
pub struct GenericPipeline<P: PipelineTypes> {
    id_to_node: HashMap<P::NodeId, P::Node>,
}

impl<P> GenericPipeline<P>
where
    P: PipelineTypes,
{
    pub fn new(nodes: impl IntoIterator<Item = P::Node>) -> Self {
        let id_to_node = nodes
            .into_iter()
            .map(|node| (node.id().clone(), node))
            .collect();
        Self { id_to_node }
    }

    pub fn is_empty(&self) -> bool {
        self.id_to_node.is_empty()
    }

    pub fn len(&self) -> usize {
        self.id_to_node.len()
    }

    /// Returns an [Iterator] over all nodes in the set.
    pub fn nodes(&self) -> impl Iterator<Item = &P::Node> {
        self.id_to_node.values()
    }

    /// Returns the node for the given ID.
    pub fn get(&self, node_id: &P::NodeId) -> Option<&P::Node> {
        self.id_to_node.get(node_id)
    }
}
