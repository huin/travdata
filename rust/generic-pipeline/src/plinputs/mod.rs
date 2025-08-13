//! Registration of dependencies (inputs) for nodes within a pipeline.

use hashbrown::{HashMap, HashSet};

use crate::node;

/// Registers pipeline inputs for nodes.
#[derive(Default)]
pub struct InputsRegistrator {
    deps: HashMap<node::NodeId, HashSet<node::NodeId>>,
}

impl InputsRegistrator {
    /// Creates a new empty [InputsRegistrator].
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns an [InputsRegistrator] for registering inputs for the given [node::NodeId].
    pub fn for_node<'a>(&'a mut self, node_id: &'a node::NodeId) -> NodeInputsRegistrator<'a> {
        NodeInputsRegistrator { node_id, reg: self }
    }

    /// Consumes the [InputsRegistrator] and returns the built up inputs.
    pub fn build(self) -> HashMap<node::NodeId, HashSet<node::NodeId>> {
        self.deps
    }
}

/// Registers pipeline inputs for a single node.
pub struct NodeInputsRegistrator<'a> {
    node_id: &'a node::NodeId,
    reg: &'a mut InputsRegistrator,
}

impl<'a> NodeInputsRegistrator<'a> {
    /// Registers a single input that the node depends upon. This declares that the node with ID
    /// `dependency_node_id` is required to provide input for the [node::NodeId] given to
    /// [InputsRegistrator::for_node].
    pub fn add_input(&mut self, dependency_node_id: &node::NodeId) {
        self.reg
            .deps
            .entry_ref(self.node_id)
            .or_default()
            .insert(dependency_node_id.clone());
    }
}
