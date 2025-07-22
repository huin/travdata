use anyhow::{Result, anyhow};

use super::GenericSystem;
use crate::{intermediates, node, plargs};

/// Used as a fallback when a [System] implementation has not been provided for a [node::Node]'s
/// [spec::Spec] type.
pub struct MissingSystem;

impl<S> GenericSystem<S> for MissingSystem
where
    S: node::SpecTrait,
{
    fn inputs(&self, _node: &node::GenericNode<S>) -> Vec<node::NodeId> {
        vec![]
    }

    fn process(
        &self,
        node: &node::GenericNode<S>,
        _args: &plargs::ArgSet,
        _intermediates: &intermediates::IntermediateSet,
    ) -> Result<intermediates::Intermediate> {
        Err(anyhow!(
            "node {:?} of type {:?} is processed by MissingSystem that will only produce errors, a system has not been installed for nodes of this type",
            node.id,
            node.spec.discriminant(),
        ))
    }
}
