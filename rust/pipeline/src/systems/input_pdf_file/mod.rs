use generic_pipeline::plparams::ParamId;
use thiserror_context::Context;

use crate::{
    StringError, SystemError, SystemResult,
    intermediates::{self, InputFile},
    monomorph::{ArgSet, NodeParamsRegistrator},
    plargs,
    plparams::{self},
    specs,
};

pub struct InputPdfFileSystem;

const PARAM_PATH: ParamId = ParamId::from_static("path");

impl generic_pipeline::systems::GenericSystem<crate::PipelineTypes> for InputPdfFileSystem {
    fn params(&self, node: &crate::Node, reg: &mut NodeParamsRegistrator) -> SystemResult<()> {
        let spec: &specs::InputPdfFile = node.spec.downcast()?;
        reg.add_param(
            PARAM_PATH,
            plparams::ParamType::InputPdf,
            spec.description.clone(),
        );
        Ok(())
    }

    fn process(
        &self,
        node: &crate::Node,
        args: &ArgSet,
        _intermediates: &intermediates::IntermediateSet,
    ) -> SystemResult<intermediates::IntermediateValue> {
        let input_pdf: &plargs::InputPdf = plargs::get_arg(args, &node.meta.id, &PARAM_PATH)?;

        if !std::fs::exists(&input_pdf.0)
            .map_err(SystemError::map_arg_value(&PARAM_PATH))
            .context("checking for existance of input PDF")?
        {
            return Err(StringError(format!(
                "input PDF does not exist at path {:?}",
                input_pdf.0
            )))
            .map_err(SystemError::map_arg_value(&PARAM_PATH));
        }

        Ok(InputFile(input_pdf.0.clone()).into())
    }
}
