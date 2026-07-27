use generic_pipeline::plparams::ParamId;
use thiserror_context::Context;

use crate::{SystemError, SystemResult, intermediates, plargs, plparams, specs};

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
    ) -> SystemResult<()> {
        let spec: &specs::OutputDirectory = node.spec.downcast()?;
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
    ) -> SystemResult<<crate::PipelineTypes as generic_pipeline::PipelineTypes>::IntermediateValue>
    {
        let output_directory_arg: &plargs::OutputDirectory =
            plargs::get_arg(args, &node.id, &PARAM_PATH)?;

        let output_directory = intermediates::OutputDirectory(output_directory_arg.0.clone());

        std::fs::DirBuilder::new()
            .recursive(true)
            .create(&output_directory.0)
            .map_err(SystemError::map_execution())
            .context("creating OutputDirectory")?;

        Ok(intermediates::IntermediateValue::from(output_directory))
    }
}
