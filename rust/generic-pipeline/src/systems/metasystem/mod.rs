#[cfg(test)]
mod tests;

use std::rc::Rc;

use super::{GenericSystem, MissingSystem, NodeResult};
use crate::{
    intermediates,
    node::{self, SpecTrait},
    plinputs, plparams,
};

/// A system that delegates to other systems based on the [SpecTrait::discriminant] of any given
/// [node::GenericNode]'s `spec`.
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
    /// Creates a new [GenericMetaSystem] that delegates to the given systems for the given
    /// [SpecTrait::discriminant].
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
    fn params<'a>(
        &self,
        node: &node::GenericNode<P::Spec>,
        reg: &'a mut plparams::GenericNodeParamsRegistrator<'a, P::ParamType>,
    ) {
        self.system_for(node.spec.discriminant()).params(node, reg)
    }

    fn inputs<'a>(
        &self,
        node: &node::GenericNode<P::Spec>,
        reg: &'a mut plinputs::NodeInputsRegistrator<'a>,
    ) {
        self.system_for(node.spec.discriminant()).inputs(node, reg)
    }

    fn process(
        &self,
        node: &node::GenericNode<P::Spec>,
        args: &crate::plargs::GenericArgSet<P::ArgValue>,
        intermediates: &intermediates::GenericIntermediateSet<P::IntermediateValue>,
    ) -> anyhow::Result<P::IntermediateValue> {
        self.system_for(node.spec.discriminant())
            .process(node, args, intermediates)
    }

    fn process_multiple<'a>(
        &self,
        nodes: &'a [&'a node::GenericNode<P::Spec>],
        args: &crate::plargs::GenericArgSet<P::ArgValue>,
        intermediates: &intermediates::GenericIntermediateSet<P::IntermediateValue>,
    ) -> Vec<NodeResult<P::IntermediateValue>> {
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
        let mut results = Vec::<NodeResult<P::IntermediateValue>>::with_capacity(nodes.len());
        for (discrim, node_group) in node_groups.drain() {
            results.extend(self.system_for(discrim).process_multiple(
                &node_group,
                args,
                intermediates,
            ));
        }

        results
    }
}
