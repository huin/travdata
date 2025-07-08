// TODO: Remove this allowance.
#![allow(dead_code)]

#[cfg(test)]
mod tests;

use std::rc::Rc;

use super::{GenericSystem, MissingSystem};
use crate::{
    intermediates,
    node::{self, spec},
};

/// A system that delegates to other systems based on the [spec::SpecDiscriminants] of any given
/// [node::Node].
pub struct GenericMetaSystem<S>
where
    S: node::SpecTrait,
{
    systems: hashbrown::HashMap<S::Discrim, Rc<dyn GenericSystem<S>>>,
    default_system: MissingSystem,
}

impl<S> GenericMetaSystem<S>
where
    S: node::SpecTrait,
{
    /// Creates a new [MetaSystem] that delegates to the given systems for the given
    /// [spec::SpecDiscriminants].
    pub fn new(systems: hashbrown::HashMap<S::Discrim, Rc<dyn GenericSystem<S>>>) -> Self {
        Self {
            systems,
            default_system: MissingSystem,
        }
    }

    fn system_for(&self, spec_type: S::Discrim) -> &dyn GenericSystem<S> {
        self.systems
            .get(&spec_type)
            .map(Rc::as_ref)
            .unwrap_or(&self.default_system)
    }
}

impl<S> GenericSystem<S> for GenericMetaSystem<S>
where
    S: node::SpecTrait,
{
    fn params(&self, node: &node::GenericNode<S>) -> Option<crate::processparams::NodeParams> {
        self.system_for(node.spec.discriminant()).params(node)
    }

    fn inputs(&self, _node: &node::GenericNode<S>) -> Vec<node::core_type::NodeId> {
        todo!()
    }

    fn process(
        &self,
        _node: &node::GenericNode<S>,
        _args: &crate::processargs::ArgSet,
        _intermediates: &intermediates::IntermediateSet,
    ) -> anyhow::Result<intermediates::Intermediate> {
        todo!()
    }

    fn process_multiple<'a>(
        &self,
        _nodes: &'a [&'a node::GenericNode<S>],
        _args: &crate::processargs::ArgSet,
        _intermediates: &intermediates::IntermediateSet,
    ) -> Vec<(
        node::core_type::NodeId,
        anyhow::Result<intermediates::Intermediate>,
    )> {
        todo!()
    }
}

/// Specific [GenericMetaSystem] used in actual processing.
pub type MetaSystem = GenericMetaSystem<spec::Spec>;
