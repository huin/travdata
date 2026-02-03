//! Intermediate data types, that are outputs of some [node::GenericNode] and inputs to others
//! during extraction processing.

use crate::node;

#[derive(Debug, thiserror::Error)]
pub enum IntermediateError {
    #[error(
        "required intermediate value from node {node_id:?} not found (bug: missing dependency)"
    )]
    MissingRequired { node_id: node::NodeId },
}

pub struct GenericIntermediateSet<V> {
    intermediates: hashbrown::HashMap<node::NodeId, V>,
}

impl<V> Default for GenericIntermediateSet<V> {
    fn default() -> Self {
        Self {
            intermediates: Default::default(),
        }
    }
}

impl<V> GenericIntermediateSet<V> {
    pub fn new() -> Self {
        Self {
            intermediates: Default::default(),
        }
    }

    pub fn set(&mut self, node_id: node::NodeId, intermediate: V) {
        self.intermediates.insert(node_id, intermediate);
    }

    pub fn get<'a>(&'a self, node_id: &node::NodeId) -> Option<&'a V> {
        self.intermediates.get(node_id)
    }

    pub fn require<'a>(&'a self, node_id: &node::NodeId) -> Result<&'a V, IntermediateError> {
        self.get(node_id)
            .ok_or_else(|| IntermediateError::MissingRequired {
                node_id: node_id.clone(),
            })
    }
}
