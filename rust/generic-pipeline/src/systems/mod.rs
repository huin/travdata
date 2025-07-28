//! Data types that act upon a [node::GenericNode] to perform individual parts of an pipeline.

mod metasystem;
mod missingsystem;

use anyhow::Result;

use crate::{intermediates, node, plargs, plparams};

pub use metasystem::GenericMetaSystem;
pub use missingsystem::MissingSystem;

/// Required trait for types that perform processing of a [crate::node::GenericNode].
/// Implementations are expected to be stateless with regards to nodes, their arguments, outputs,
/// etc.
pub trait GenericSystem<P>
where
    P: crate::PipelineTypes,
{
    /// Returns the parameters for the node, if any.
    fn params(&self, _node: &node::GenericNode<P::Spec>) -> plparams::Params<P::ParamType> {
        plparams::Params {
            params: Vec::default(),
        }
    }

    /// Returns the set of node IDs that the given node depends on as inputs.
    fn inputs(&self, node: &node::GenericNode<P::Spec>) -> Vec<node::NodeId>;

    /// Performs processing of the given [node::GenericNode], returning its intermediate value.
    fn process(
        &self,
        node: &node::GenericNode<P::Spec>,
        args: &plargs::GenericArgSet<P::ArgValue>,
        intermediates: &intermediates::IntermediateSet<P::IntermediateValue>,
    ) -> Result<P::IntermediateValue>;

    /// Performs processing of the given [node::GenericNode]s, returning their
    /// intermediate value(s).
    ///
    /// The default implementation processes in serial. Specific implementations may choose to
    /// optimise this.
    fn process_multiple<'a>(
        &self,
        nodes: &'a [&'a node::GenericNode<P::Spec>],
        args: &plargs::GenericArgSet<P::ArgValue>,
        intermediates: &intermediates::IntermediateSet<P::IntermediateValue>,
    ) -> Vec<(node::NodeId, Result<P::IntermediateValue>)> {
        nodes
            .iter()
            .map(|&node| (node.id.clone(), self.process(node, args, intermediates)))
            .collect()
    }
}
