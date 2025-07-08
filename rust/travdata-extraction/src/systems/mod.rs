//! Data types that act upon [crate::node::Node] to perform individual parts of an extraction process.

mod metasystem;
mod missingsystem;

use anyhow::Result;

use crate::{
    intermediates,
    node::{self, core_type},
    processargs, processparams,
};

pub use metasystem::MetaSystem;
pub use missingsystem::MissingSystem;

/// Required trait for types that perform processing of a [node::Node]. Implementations are
/// expected to be stateless with regards to nodes, their arguments, outputs, etc.
pub trait System {
    /// Returns the parameters for the node, if any.
    fn params(&self, _node: &node::Node) -> Option<processparams::NodeParams> {
        None
    }

    /// Returns the set of node IDs that the given node depends on as inputs.
    fn inputs(&self, node: &node::Node) -> Vec<core_type::NodeId>;

    /// Performs processing of the given [node::Node], returning its [intermediates::Intermediate].
    fn process(
        &self,
        node: &node::Node,
        args: &processargs::ArgSet,
        intermediates: &intermediates::IntermediateSet,
    ) -> Result<intermediates::Intermediate>;

    /// Performs processing of the given [node::Node]s, returning their
    /// [intermediates::Intermediate]s.
    ///
    /// The default implementation processes in serial. Specific implementations may choose to
    /// optimise this.
    fn process_multiple<'a>(
        &self,
        nodes: &'a [&'a node::Node],
        args: &processargs::ArgSet,
        intermediates: &intermediates::IntermediateSet,
    ) -> Vec<(core_type::NodeId, Result<intermediates::Intermediate>)> {
        nodes
            .iter()
            .map(|&node| (node.id.clone(), self.process(node, args, intermediates)))
            .collect()
    }
}
