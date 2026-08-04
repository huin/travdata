#[cfg(test)]
mod tests;

use std::rc::Rc;

use super::{GenericSystem, NodeResult};
use crate::{intermediates, plinputs, plparams};

pub trait TypedNode {
    type NodeType: std::fmt::Debug + Eq + std::hash::Hash;

    fn node_type(&self) -> Self::NodeType;
}

pub type MissingSystemErrorFn<D, E> = dyn Fn(D) -> E;

/// A system that delegates to other systems based on the [TypedNode::node_type].
pub struct GenericMetaSystem<P>
where
    P: crate::PipelineTypes,
    P::Node: TypedNode,
{
    systems: hashbrown::HashMap<<P::Node as TypedNode>::NodeType, Rc<dyn GenericSystem<P>>>,
    missing_system_error:
        Box<MissingSystemErrorFn<<P::Node as TypedNode>::NodeType, P::SystemError>>,
}

impl<P> GenericMetaSystem<P>
where
    P: crate::PipelineTypes,
    P::Node: TypedNode,
{
    /// Creates a new [GenericMetaSystem] that delegates to the given systems for the given
    /// [DiscriminatedSpec::discriminant].
    pub fn new(
        systems: hashbrown::HashMap<<P::Node as TypedNode>::NodeType, Rc<dyn GenericSystem<P>>>,
        missing_system_error: Box<
            MissingSystemErrorFn<<P::Node as TypedNode>::NodeType, P::SystemError>,
        >,
    ) -> Self {
        Self {
            systems,
            missing_system_error,
        }
    }

    fn system_for(
        &self,
        spec_type: <P::Node as TypedNode>::NodeType,
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
    P::Node: TypedNode,
{
    fn params<'a>(
        &self,
        node: &P::Node,
        reg: &'a mut plparams::GenericNodeParamsRegistrator<'a, P>,
    ) -> Result<(), P::SystemError> {
        self.system_for(node.node_type())?.params(node, reg)
    }

    fn inputs<'a>(
        &self,
        node: &P::Node,
        reg: &'a mut plinputs::NodeInputsRegistrator<'a, P>,
    ) -> Result<(), P::SystemError> {
        self.system_for(node.node_type())?.inputs(node, reg)
    }

    fn process(
        &self,
        node: &P::Node,
        args: &crate::plargs::GenericArgSet<P>,
        intermediates: &intermediates::GenericIntermediateSet<P>,
    ) -> Result<P::IntermediateValue, P::SystemError> {
        self.system_for(node.node_type())?
            .process(node, args, intermediates)
    }

    fn process_multiple<'a>(
        &self,
        nodes: &'a [&'a P::Node],
        args: &crate::plargs::GenericArgSet<P>,
        intermediates: &intermediates::GenericIntermediateSet<P>,
    ) -> Vec<NodeResult<P>> {
        let mut node_groups =
            hashbrown::HashMap::<<P::Node as TypedNode>::NodeType, Vec<&P::Node>>::new();

        // Group nodes by their discriminant.
        for node in nodes {
            let discrim = node.node_type();
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
