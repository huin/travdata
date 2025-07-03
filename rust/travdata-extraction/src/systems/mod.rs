//! Data types that act upon [crate::node::Node] to perform individual parts of an extraction process.

use anyhow::{Result, anyhow};

use crate::{
    intermediates,
    node::{self, core_type, spec},
    processargs, processparams,
};

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
    fn process_multiple(
        &self,
        nodes: &[&node::Node],
        args: &processargs::ArgSet,
        intermediates: &intermediates::IntermediateSet,
    ) -> Vec<(core_type::NodeId, Result<intermediates::Intermediate>)> {
        nodes
            .iter()
            .map(|&node| (node.id.clone(), self.process(node, args, intermediates)))
            .collect()
    }
}

/// Used as a fallback when a [System] implementation has not been provided for a [node::Node]'s
/// [spec::Spec] type.
pub struct MissingSystem;

impl System for MissingSystem {
    fn inputs(&self, _node: &node::Node) -> Vec<core_type::NodeId> {
        vec![]
    }

    fn process(
        &self,
        node: &node::Node,
        _args: &processargs::ArgSet,
        _intermediates: &intermediates::IntermediateSet,
    ) -> Result<intermediates::Intermediate> {
        Err(anyhow!(
            "node {:?} of type {:?} is processed by MissingSystem that will only produce errors, a system has not been installed for nodes of this type",
            node.id,
            spec::SpecDiscriminants::from(&node.spec),
        ))
    }
}
