#[cfg(test)]
mod tests;

use std::rc::Rc;

use super::{GenericSystem, NodeResult};
use crate::{
    intermediates,
    node::{self, SpecTrait},
    plinputs, plparams,
};

pub type MissingSystemErrorFn<D, E> = dyn Fn(D) -> E;

/// A system that delegates to other systems based on the [SpecTrait::discriminant] of any given
/// [node::GenericNode]'s `spec`.
pub struct GenericMetaSystem<P>
where
    P: crate::PipelineTypes,
{
    systems: hashbrown::HashMap<<P::Spec as node::SpecTrait>::Discrim, Rc<dyn GenericSystem<P>>>,
    missing_system_error:
        Box<MissingSystemErrorFn<<P::Spec as node::SpecTrait>::Discrim, P::SystemError>>,
}

impl<P> GenericMetaSystem<P>
where
    P: crate::PipelineTypes,
{
    /// Creates a new [GenericMetaSystem] that delegates to the given systems for the given
    /// [SpecTrait::discriminant].
    pub fn new(
        systems: hashbrown::HashMap<
            <P::Spec as node::SpecTrait>::Discrim,
            Rc<dyn GenericSystem<P>>,
        >,
        missing_system_error: Box<
            MissingSystemErrorFn<<P::Spec as node::SpecTrait>::Discrim, P::SystemError>,
        >,
    ) -> Self {
        Self {
            systems,
            missing_system_error,
        }
    }

    fn system_for(
        &self,
        spec_type: <P::Spec as node::SpecTrait>::Discrim,
    ) -> Result<&dyn GenericSystem<P>, P::SystemError> {
        self.systems
            .get(&spec_type)
            .map(Rc::as_ref)
            .ok_or_else(|| (self.missing_system_error)(spec_type))
    }
}

impl<P> GenericSystem<P> for GenericMetaSystem<P>
where
    P: crate::PipelineTypes,
{
    fn params<'a>(
        &self,
        node: &node::GenericNode<P::Spec>,
        reg: &'a mut plparams::GenericNodeParamsRegistrator<'a, P::ParamType>,
    ) -> Result<(), P::SystemError> {
        self.system_for(node.spec.discriminant())?.params(node, reg)
    }

    fn inputs<'a>(
        &self,
        node: &node::GenericNode<P::Spec>,
        reg: &'a mut plinputs::NodeInputsRegistrator<'a>,
    ) -> Result<(), P::SystemError> {
        self.system_for(node.spec.discriminant())?.inputs(node, reg)
    }

    fn process(
        &self,
        node: &node::GenericNode<P::Spec>,
        args: &crate::plargs::GenericArgSet<P::ArgValue>,
        intermediates: &intermediates::GenericIntermediateSet<P::IntermediateValue>,
    ) -> Result<P::IntermediateValue, P::SystemError> {
        self.system_for(node.spec.discriminant())?
            .process(node, args, intermediates)
    }

    fn process_multiple<'a>(
        &self,
        nodes: &'a [&'a node::GenericNode<P::Spec>],
        args: &crate::plargs::GenericArgSet<P::ArgValue>,
        intermediates: &intermediates::GenericIntermediateSet<P::IntermediateValue>,
    ) -> Vec<NodeResult<P>> {
        let mut node_groups = hashbrown::HashMap::<
            <P::Spec as SpecTrait>::Discrim,
            Vec<&node::GenericNode<P::Spec>>,
        >::new();

        // Group nodes by their discriminant.
        for node in nodes {
            let discrim = node.spec.discriminant();
            node_groups.entry(discrim).or_default().push(node);
        }

        // Delegate by each group.
        let mut results = Vec::<NodeResult<P>>::with_capacity(nodes.len());
        for (discrim, node_group) in node_groups.drain() {
            results.extend(
                self.system_for(discrim)
                    .map(|system| system.process_multiple(&node_group, args, intermediates))
                    .into_iter()
                    .flatten(),
            );
        }

        results
    }
}
