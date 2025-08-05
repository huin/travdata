//! Intermediate data types, that are outputs of some [node::GenericNode] and inputs to others
//! during extraction processing.

use crate::node;

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
}
