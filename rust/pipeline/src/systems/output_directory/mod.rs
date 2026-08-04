use generic_pipeline::plparams::ParamId;
use thiserror_context::Context;

use crate::{SystemError, SystemResult, intermediates, plargs, plparams, specs};

pub struct OutputDirectorySystem;

const PARAM_PATH: ParamId = ParamId::from_static("path");

impl generic_pipeline::systems::GenericSystem<crate::PipelineTypes> for OutputDirectorySystem {
    fn params(
        &self,
        node: &crate::Node,
        reg: &mut plparams::NodeParamsRegistrator,
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
        node: &crate::Node,
        args: &plargs::ArgSet,
        _intermediates: &intermediates::IntermediateSet,
    ) -> SystemResult<intermediates::IntermediateValue> {
        let output_directory_arg: &plargs::OutputDirectory =
            plargs::get_arg(args, &node.meta.id, &PARAM_PATH)?;

        let output_directory = intermediates::OutputDirectory(output_directory_arg.0.clone());

        std::fs::DirBuilder::new()
            .recursive(true)
            .create(&output_directory.0)
            .map_err(SystemError::map_execution())
            .context("creating OutputDirectory")?;

        Ok(intermediates::IntermediateValue::from(output_directory))
    }
}
