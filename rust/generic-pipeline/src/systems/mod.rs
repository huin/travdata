//! Data types that act upon a [node::GenericNode] to perform individual parts of an pipeline.

mod metasystem;

use crate::{PipelineTypes, intermediates, node, plargs, plinputs, plparams};

pub use metasystem::{DiscriminatedSpec, GenericMetaSystem};

/// Result of processing a node.
pub struct NodeResult<P>
where
    P: PipelineTypes,
{
    pub id: node::NodeId,
    pub value: Result<P::IntermediateValue, P::SystemError>,
}

impl<P> std::fmt::Debug for NodeResult<P>
where
    P: PipelineTypes,
    P::IntermediateValue: std::fmt::Debug,
    P::SystemError: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeResult")
            .field("id", &self.id)
            .field("value", &self.value)
            .finish()
    }
}

/// Required trait for types that perform processing of a [crate::node::GenericNode].
/// Implementations are expected to be stateless with regards to nodes, their arguments, outputs,
/// etc.
pub trait GenericSystem<P>
where
    P: crate::PipelineTypes,
{
    /// Generates the parameters for the node, if any.
    fn params<'a>(
        &self,
        _node: &node::GenericNode<P::Spec>,
        _reg: &'a mut plparams::GenericNodeParamsRegistrator<'a, P::ParamType>,
    ) -> Result<(), P::SystemError> {
        Ok(())
    }

    /// Registers the set of node IDs that the given node depends on as inputs.
    fn inputs<'a>(
        &self,
        _node: &node::GenericNode<P::Spec>,
        _reg: &'a mut plinputs::NodeInputsRegistrator<'a>,
    ) -> Result<(), P::SystemError> {
        Ok(())
    }

    /// Performs processing of the given [node::GenericNode], returning its intermediate value.
    fn process(
        &self,
        node: &node::GenericNode<P::Spec>,
        args: &plargs::GenericArgSet<P::ArgValue>,
        intermediates: &intermediates::GenericIntermediateSet<P::IntermediateValue>,
    ) -> Result<P::IntermediateValue, P::SystemError>;

    /// Performs processing of the given [node::GenericNode]s, returning their
    /// intermediate value(s).
    ///
    /// The default implementation processes in serial. Specific implementations may choose to
    /// optimise this.
    fn process_multiple<'a>(
        &self,
        nodes: &'a [&'a node::GenericNode<P::Spec>],
        args: &plargs::GenericArgSet<P::ArgValue>,
        intermediates: &intermediates::GenericIntermediateSet<P::IntermediateValue>,
        // TODO: maybe this should return `impl Iterator` instead of an allocated vector
    ) -> Vec<NodeResult<P>> {
        nodes
            .iter()
            .map(|&node| NodeResult::<P> {
                id: node.id.clone(),
                value: self.process(node, args, intermediates),
            })
            .collect()
    }
}
