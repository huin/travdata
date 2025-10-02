use anyhow::{Result, anyhow};

use super::GenericSystem;
use crate::{
    intermediates,
    node::{self, SpecTrait},
    plargs, plinputs,
};

/// Used as a fallback when a [crate::systems::GenericSystem] implementation has not been provided
/// for a [node::GenericNode]'s [node::SpecTrait] type.
pub struct MissingSystem;

impl MissingSystem {
    fn error<P>(node: &node::GenericNode<<P as crate::PipelineTypes>::Spec>) -> anyhow::Error
    where
        P: crate::PipelineTypes,
    {
        anyhow!(
            "node {:?} of type {:?} is processed by MissingSystem that will only produce errors, a system has not been installed for nodes of this type",
            node.id,
            node.spec.discriminant(),
        )
    }
}

impl<P> GenericSystem<P> for MissingSystem
where
    P: crate::PipelineTypes,
{
    fn params<'a>(
        &self,
        node: &node::GenericNode<<P as crate::PipelineTypes>::Spec>,
        _reg: &'a mut crate::plparams::GenericNodeParamsRegistrator<
            'a,
            <P as crate::PipelineTypes>::ParamType,
        >,
    ) -> Result<()> {
        Err(Self::error::<P>(node))
    }

    fn inputs<'a>(
        &self,
        node: &node::GenericNode<<P as crate::PipelineTypes>::Spec>,
        _reg: &'a mut plinputs::NodeInputsRegistrator<'a>,
    ) -> Result<()> {
        Err(Self::error::<P>(node))
    }

    fn process(
        &self,
        node: &node::GenericNode<P::Spec>,
        _args: &plargs::GenericArgSet<P::ArgValue>,
        _intermediates: &intermediates::GenericIntermediateSet<P::IntermediateValue>,
    ) -> Result<P::IntermediateValue> {
        Err(Self::error::<P>(node))
    }
}
