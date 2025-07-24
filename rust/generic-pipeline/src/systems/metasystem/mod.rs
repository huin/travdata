// TODO: Remove this allowance.
#![allow(dead_code)]

#[cfg(test)]
mod tests;

use std::rc::Rc;

use super::{GenericSystem, MissingSystem};
use crate::{
    intermediates,
    node::{self, SpecTrait},
};

/// A system that delegates to other systems based on the [spec::SpecDiscriminants] of any given
/// [node::Node].
pub struct GenericMetaSystem<P>
where
    P: crate::PipelineTypes,
{
    systems: hashbrown::HashMap<<P::Spec as node::SpecTrait>::Discrim, Rc<dyn GenericSystem<P>>>,
    default_system: MissingSystem,
}

impl<P> GenericMetaSystem<P>
where
    P: crate::PipelineTypes,
{
    /// Creates a new [MetaSystem] that delegates to the given systems for the given
    /// [spec::SpecDiscriminants].
    pub fn new(
        systems: hashbrown::HashMap<
            <P::Spec as node::SpecTrait>::Discrim,
            Rc<dyn GenericSystem<P>>,
        >,
    ) -> Self {
        Self {
            systems,
            default_system: MissingSystem,
        }
    }

    fn system_for(
        &self,
        spec_type: <P::Spec as node::SpecTrait>::Discrim,
    ) -> &dyn GenericSystem<P> {
        self.systems
            .get(&spec_type)
            .map(Rc::as_ref)
            .unwrap_or(&self.default_system)
    }
}

impl<P> GenericSystem<P> for GenericMetaSystem<P>
where
    P: crate::PipelineTypes,
{
    fn params(&self, node: &node::GenericNode<P::Spec>) -> crate::plparams::Params<P::ParamType> {
        self.system_for(node.spec.discriminant()).params(node)
    }

    fn inputs(&self, _node: &node::GenericNode<P::Spec>) -> Vec<node::NodeId> {
        todo!()
    }

    fn process(
        &self,
        _node: &node::GenericNode<P::Spec>,
        _args: &crate::plargs::GenericArgSet<P::ArgValue>,
        _intermediates: &intermediates::IntermediateSet<P::IntermediateValue>,
    ) -> anyhow::Result<P::IntermediateValue> {
        todo!()
    }

    fn process_multiple<'a>(
        &self,
        _nodes: &'a [&'a node::GenericNode<P::Spec>],
        _args: &crate::plargs::GenericArgSet<P::ArgValue>,
        _intermediates: &intermediates::IntermediateSet<P::IntermediateValue>,
    ) -> Vec<(node::NodeId, anyhow::Result<P::IntermediateValue>)> {
        todo!()
    }
}
