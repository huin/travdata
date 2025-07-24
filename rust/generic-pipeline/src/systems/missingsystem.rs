use anyhow::{Result, anyhow};

use super::GenericSystem;
use crate::{
    intermediates,
    node::{self, SpecTrait},
    plargs,
};

/// Used as a fallback when a [crate::systems::GenericSystem] implementation has not been provided
/// for a [node::Node]'s [crate::node::spec::Spec] type.
pub struct MissingSystem;

impl<P> GenericSystem<P> for MissingSystem
where
    P: crate::PipelineTypes,
{
    fn inputs(&self, _node: &node::GenericNode<P::Spec>) -> Vec<node::NodeId> {
        vec![]
    }

    fn process(
        &self,
        node: &node::GenericNode<P::Spec>,
        _args: &plargs::GenericArgSet<P::ArgValue>,
        _intermediates: &intermediates::IntermediateSet<P::IntermediateValue>,
    ) -> Result<P::IntermediateValue> {
        Err(anyhow!(
            "node {:?} of type {:?} is processed by MissingSystem that will only produce errors, a system has not been installed for nodes of this type",
            node.id,
            node.spec.discriminant(),
        ))
    }
}
