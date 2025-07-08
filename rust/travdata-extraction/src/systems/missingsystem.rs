use anyhow::{Result, anyhow};

use super::System;
use crate::{
    intermediates,
    node::{self, core_type, spec},
    processargs,
};

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
