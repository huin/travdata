use anyhow::{Context, Result, bail};
use generic_pipeline::plparams::ParamId;

use crate::{intermediates, plargs, plparams, specs};

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
    ) -> Result<()> {
        let spec = <&specs::InputPdfFile>::try_from(&node.spec)?;
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
    ) -> anyhow::Result<<crate::PipelineTypes as generic_pipeline::PipelineTypes>::IntermediateValue>
    {
        let input_pdf = args
            .require(&node.id, &PARAM_PATH)?
            .try_into()
            .map(|arg_value: &plargs::InputPdf| intermediates::InputFile(arg_value.0.clone()))?;

        if !std::fs::exists(&input_pdf.0).context("checking for existance of input PDF")? {
            bail!("input PDF does not exist at path {:?}", input_pdf.0);
        }

        Ok(intermediates::IntermediateValue::from(input_pdf))
    }
}
