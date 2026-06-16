use generic_pipeline::plparams::ParamId;
use thiserror_context::Context;

use crate::{
    StringError, SystemError, SystemResult, intermediates::InputFile, plargs, plparams, specs,
};

pub struct InputPdfFileSystem;

const PARAM_PATH: ParamId = ParamId::from_static("path");

impl generic_pipeline::systems::GenericSystem<crate::PipelineTypes> for InputPdfFileSystem {
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
        let spec: &specs::InputPdfFile = node.spec.downcast_spec()?;
        reg.add_param(
            PARAM_PATH,
            plparams::ParamType::InputPdf,
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
        let input_pdf: &plargs::InputPdf = plargs::get_arg(args, &node.id, &PARAM_PATH)?;

        if !std::fs::exists(&input_pdf.0)
            .map_err(SystemError::map_param(&PARAM_PATH))
            .context("checking for existance of input PDF")?
        {
            return Err(StringError(format!(
                "input PDF does not exist at path {:?}",
                input_pdf.0
            )))
            .map_err(SystemError::map_param(&PARAM_PATH));
        }

        Ok(InputFile(input_pdf.0.clone()).into())
    }
}
