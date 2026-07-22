use generic_pipeline::PipelineNode as _;
use hashbrown::HashMap;
use pipeline::NodeId;

use crate::app::{data::NodeRef, ddo};

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct OrderedNodeSet {
    next_node_ref: NodeRef,
    nodes: HashMap<NodeRef, pipeline::Node>,
    order: Vec<NodeRef>,
}

impl OrderedNodeSet {
    pub fn new(pipeline: ddo::PipelineNodes) -> Result<Self, String> {
        // First determine the NodeRef for each node, so that EditableNode will be able to resolve the
        // NodeRef for any known NodeId during initialisation. This mapping from node ID to NodeRef
        // will be discarded after initialisation, as it will not be maintained afterwards.
        let mut node_id_to_node_ref: HashMap<NodeId, NodeRef> =
            HashMap::with_capacity(pipeline.len());
        let mut next_node_ref = NodeRef::default();
        for node in &pipeline {
            use hashbrown::hash_map::Entry::*;
            match node_id_to_node_ref.entry(node.id().clone()) {
                Occupied(_occupied_entry) => {
                    return Err(format!("multiple nodes share id {:?}", node.id()));
                }
                Vacant(vacant_entry) => {
                    let node_ref = next_node_ref.next_and_inc()?;
                    vacant_entry.insert(node_ref);
                }
            }
        }

        let mut node_set = Self {
            next_node_ref,
            nodes: HashMap::with_capacity(pipeline.len()),
            order: Vec::with_capacity(pipeline.len()),
        };
        for node in pipeline.into_iter() {
            let node_ref = node_id_to_node_ref.get(node.id()).ok_or(
                // This should not fail, as the NodeId must exist from the loop above.
                "internal error: failed to resolve NodeRef for NodeId during initialisation",
            )?;
            node_set.add_node_with_ref(node, *node_ref);
        }
        Ok(node_set)
    }

    fn add_node_with_ref(&mut self, node: pipeline::Node, node_ref: NodeRef) {
        self.nodes.insert(node_ref, node);
        self.order.push(node_ref);
    }

    pub fn nodes_in_order(&self) -> impl Iterator<Item = Option<&pipeline::Node>> {
        self.order.iter().map(|node_ref| self.nodes.get(node_ref))
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    /// Returns a mutable reference to a node, given its ordered index ([[None]] if out of  bounds).
    pub fn get_by_index_mut(&mut self, index: usize) -> Option<(NodeRef, &mut pipeline::Node)> {
        self.order
            .get(index)
            .and_then(|node_ref| self.nodes.get_mut(node_ref).map(|node| (*node_ref, node)))
    }

    /// Returns a mutable reference to a node, given its NodeRef ([[None]] if not exists).
    pub fn get_by_node_ref_mut(&mut self, node_ref: NodeRef) -> Option<&mut pipeline::Node> {
        self.nodes.get_mut(&node_ref)
    }
}
