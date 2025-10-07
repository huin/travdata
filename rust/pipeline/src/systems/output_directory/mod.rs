use anyhow::Result;
use generic_pipeline::plparams::ParamId;

use crate::{intermediates, plargs, plparams, specs};

pub struct OutputDirectorySystem;

const PARAM_PATH: ParamId = ParamId::from_static("path");

impl generic_pipeline::systems::GenericSystem<crate::PipelineTypes> for OutputDirectorySystem {
    fn params<'a>(
        &self,
        node: &generic_pipeline::node::GenericNode<
            <crate::PipelineTypes as generic_pipeline::PipelineTypes>::Spec,
        >,
        reg: &'a mut generic_pipeline::plparams::GenericNodeParamsRegistrator<
            'a,
            <crate::PipelineTypes as generic_pipeline::PipelineTypes>::ParamType,
        >,
    ) -> Result<()> {
        let spec = <&specs::OutputDirectory>::try_from(&node.spec)?;
        reg.add_param(
            PARAM_PATH,
            plparams::ParamType::OutputDirectory,
            spec.description.clone(),
        );
        Ok(())
    }

    fn process(
        &self,
        node: &generic_pipeline::node::GenericNode<
            <crate::PipelineTypes as generic_pipeline::PipelineTypes>::Spec,
        >,
        args: &generic_pipeline::plargs::GenericArgSet<
            <crate::PipelineTypes as generic_pipeline::PipelineTypes>::ArgValue,
        >,
        _intermediates: &generic_pipeline::intermediates::GenericIntermediateSet<
            <crate::PipelineTypes as generic_pipeline::PipelineTypes>::IntermediateValue,
        >,
    ) -> anyhow::Result<<crate::PipelineTypes as generic_pipeline::PipelineTypes>::IntermediateValue>
    {
        args.require(&node.id, &PARAM_PATH)
            .and_then(<&plargs::OutputDirectory>::try_from)
            .map(|arg_value| intermediates::OutputDirectory(arg_value.0.clone()))
            .map(intermediates::IntermediateValue::from)
    }
}
