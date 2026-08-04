//! Registration of dependencies (inputs) for nodes within a pipeline.

use hashbrown::{HashMap, HashSet};

use crate::PipelineTypes;

/// Registers pipeline inputs for nodes.
#[derive(Default)]
pub struct InputsRegistrator<P: PipelineTypes> {
    deps: HashMap<P::NodeId, HashSet<P::NodeId>>,
}

impl<P> InputsRegistrator<P>
where
    P: PipelineTypes,
{
    /// Creates a new empty [InputsRegistrator].
    pub fn new() -> Self {
        Self {
            deps: HashMap::new(),
        }
    }

    /// Returns an [InputsRegistrator] for registering inputs for the given [crate::node::PipelineNodeId].
    pub fn for_node<'a>(&'a mut self, node_id: &'a P::NodeId) -> NodeInputsRegistrator<'a, P> {
        NodeInputsRegistrator { node_id, reg: self }
    }

    /// Consumes the [InputsRegistrator] and returns the built up inputs.
    pub fn build(self) -> HashMap<P::NodeId, HashSet<P::NodeId>> {
        self.deps
    }
}

/// Registers pipeline inputs for a single node.
pub struct NodeInputsRegistrator<'a, P: PipelineTypes> {
    node_id: &'a P::NodeId,
    reg: &'a mut InputsRegistrator<P>,
}

impl<'a, P> NodeInputsRegistrator<'a, P>
where
    P: PipelineTypes,
{
    /// Registers a single input that the node depends upon. This declares that the node with ID
    /// `dependency_node_id` is required to provide input for the [node::NodeId] given to
    /// [InputsRegistrator::for_node].
    pub fn add_input(&mut self, dependency_node_id: &P::NodeId) {
        self.reg
            .deps
            .entry_ref(self.node_id)
            .or_default()
            .insert(dependency_node_id.clone());
    }
}
