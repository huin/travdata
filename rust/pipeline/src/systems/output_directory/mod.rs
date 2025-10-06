use anyhow::{Result, anyhow};
use generic_pipeline::plparams::ParamId;

use crate::{
    intermediates::IntermediateValue,
    plargs::ArgValue,
    plparams,
    specs::{OutputDirectory, TryCastSpec},
};

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
        let spec: &OutputDirectory = node.spec.try_cast_spec()?;
        reg.add_param(
            PARAM_PATH,
            plparams::ParamType::OutputDirectory,
            spec.description.clone(),
        );
        Ok(())
    }

    fn process(
        &self,
        _node: &generic_pipeline::node::GenericNode<
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
        args.get(&_node.id, &PARAM_PATH)
            .ok_or_else(|| anyhow!("argument {:?} not set", PARAM_PATH))
            .and_then(|arg_value| match arg_value {
                ArgValue::OutputDirectory(path) => Ok(path.clone()),
                _ => Err(anyhow!(
                    "argument {:?} should be of type OutputDirectory, but got {:?}",
                    PARAM_PATH,
                    arg_value,
                )),
            })
            .map(IntermediateValue::OutputDirectory)
    }
}
