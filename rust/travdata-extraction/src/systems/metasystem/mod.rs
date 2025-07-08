// TODO: Remove this allowance.
#![allow(dead_code)]

#[cfg(test)]
mod tests;

use std::rc::Rc;

use strum::IntoDiscriminant;

use super::{MissingSystem, System};
use crate::{
    intermediates,
    node::{self, spec},
};

/// A system that delegates to other systems based on the [spec::SpecDiscriminants] of any given
/// [node::Node].
pub struct MetaSystem {
    systems: hashbrown::HashMap<spec::SpecDiscriminants, Rc<dyn System>>,
    default_system: MissingSystem,
}

impl MetaSystem {
    /// Creates a new [MetaSystem] that delegates to the given systems for the given
    /// [spec::SpecDiscriminants].
    pub fn new(systems: hashbrown::HashMap<spec::SpecDiscriminants, Rc<dyn System>>) -> Self {
        Self {
            systems,
            default_system: MissingSystem,
        }
    }

    fn system_for(&self, spec_type: spec::SpecDiscriminants) -> &dyn System {
        self.systems
            .get(&spec_type)
            .map(Rc::as_ref)
            .unwrap_or(&self.default_system)
    }
}

impl System for MetaSystem {
    fn params(&self, node: &node::Node) -> Option<crate::processparams::NodeParams> {
        self.system_for(node.spec.discriminant()).params(node)
    }

    fn inputs(&self, _node: &node::Node) -> Vec<node::core_type::NodeId> {
        todo!()
    }

    fn process(
        &self,
        _node: &node::Node,
        _args: &crate::processargs::ArgSet,
        _intermediates: &intermediates::IntermediateSet,
    ) -> anyhow::Result<intermediates::Intermediate> {
        todo!()
    }

    fn process_multiple<'a>(
        &self,
        _nodes: &'a [&'a node::Node],
        _args: &crate::processargs::ArgSet,
        _intermediates: &intermediates::IntermediateSet,
    ) -> Vec<(
        node::core_type::NodeId,
        anyhow::Result<intermediates::Intermediate>,
    )> {
        todo!()
    }
}
