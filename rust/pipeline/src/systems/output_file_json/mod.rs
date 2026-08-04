#[cfg(test)]
mod tests;

use generic_pipeline::systems::GenericSystem;
use thiserror_context::Context;

use crate::{
    SystemError, SystemResult, intermediates,
    monomorph::{ArgSet, NodeInputsRegistrator},
    specs,
};

pub struct OutputFileJsonSystem;

impl GenericSystem<crate::PipelineTypes> for OutputFileJsonSystem {
    fn inputs(&self, node: &crate::Node, reg: &mut NodeInputsRegistrator) -> SystemResult<()> {
        let spec: &specs::OutputFileJson = node.spec.downcast()?;
        reg.add_input(&spec.input_data);
        reg.add_input(&spec.directory);
        Ok(())
    }

    fn process(
        &self,
        node: &crate::Node,
        _args: &ArgSet,
        intermediates: &intermediates::IntermediateSet,
    ) -> SystemResult<intermediates::IntermediateValue> {
        let spec: &specs::OutputFileJson = node.spec.downcast()?;
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
