#[cfg(test)]
mod tests;

use generic_pipeline::systems::GenericSystem;
use thiserror_context::Context;

use crate::{SystemError, SystemResult, intermediates, specs};

pub struct OutputFileJsonSystem;

impl GenericSystem<crate::PipelineTypes> for OutputFileJsonSystem {
    fn inputs<'a>(
        &self,
        node: &generic_pipeline::node::GenericNode<
            <crate::PipelineTypes as generic_pipeline::PipelineTypes>::Spec,
        >,
        reg: &'a mut generic_pipeline::plinputs::NodeInputsRegistrator<'a>,
    ) -> SystemResult<()> {
        let spec: &specs::OutputFileJson = node.spec.downcast_spec()?;
        reg.add_input(&spec.input_data);
        reg.add_input(&spec.directory);
        Ok(())
    }

    fn process(
        &self,
        node: &generic_pipeline::node::GenericNode<
            <crate::PipelineTypes as generic_pipeline::PipelineTypes>::Spec,
        >,
        _args: &generic_pipeline::plargs::GenericArgSet<
            <crate::PipelineTypes as generic_pipeline::PipelineTypes>::ArgValue,
        >,
        intermediates: &generic_pipeline::intermediates::GenericIntermediateSet<
            <crate::PipelineTypes as generic_pipeline::PipelineTypes>::IntermediateValue,
        >,
    ) -> SystemResult<intermediates::IntermediateValue> {
        let spec: &specs::OutputFileJson = node.spec.downcast_spec()?;
        let directory: &intermediates::OutputDirectory =
            intermediates::get_intermediate_input(intermediates, &spec.directory)?;
        let data: &intermediates::JsonData =
            intermediates::get_intermediate_input(intermediates, &spec.input_data)?;

        let output_path = directory.create_parent_dirs_for_file(&spec.filename)?;

        let mut output_file = std::fs::File::create(&output_path)
            .map_err(SystemError::map_execution())
            .with_context(|| format!("opening JSON output file {output_path:?}"))?;
        serde_json::to_writer(&mut output_file, &data.0)
            .map_err(SystemError::map_execution())
            .context("writing JSON output")?;
        output_file
            .sync_all()
            .map_err(SystemError::map_execution())
            .context("flushing JSON output")?;

        Ok(intermediates::NoData.into())
    }
}
