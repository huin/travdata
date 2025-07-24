//! Intermediate data types, that are outputs of some [crate::node::Node] and inputs to others
//! during extraction processing.

use crate::node;

pub struct IntermediateSet<V> {
    intermediates: hashbrown::HashMap<node::NodeId, V>,
}

impl<V> Default for IntermediateSet<V> {
    fn default() -> Self {
        Self {
            intermediates: Default::default(),
        }
    }
}

impl<V> IntermediateSet<V> {
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
