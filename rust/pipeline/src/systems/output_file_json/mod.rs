#[cfg(test)]
mod tests;

use anyhow::Context;
use generic_pipeline::systems::GenericSystem;

use crate::{intermediates, specs};

pub struct OutputFileJsonSystem;

impl GenericSystem<crate::PipelineTypes> for OutputFileJsonSystem {
    fn inputs<'a>(
        &self,
        node: &generic_pipeline::node::GenericNode<
            <crate::PipelineTypes as generic_pipeline::PipelineTypes>::Spec,
        >,
        reg: &'a mut generic_pipeline::plinputs::NodeInputsRegistrator<'a>,
    ) -> anyhow::Result<()> {
        let spec = <&specs::OutputFileCsv>::try_from(&node.spec)?;
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
    ) -> anyhow::Result<intermediates::IntermediateValue> {
        let spec = <&specs::OutputFileJson>::try_from(&node.spec)?;
        let directory = intermediates
            .require(&spec.directory)
            .and_then(<&intermediates::OutputDirectory>::try_from)
            .context("getting output directory")?;
        let data = intermediates
            .require(&spec.input_data)
            .and_then(<&intermediates::JsonData>::try_from)
            .context("getting data to output")?;

        let output_path = directory
            .create_parent_dirs_for_file(&spec.filename)
            .context("creating parent directory for output data")?;

        let mut output_file = std::fs::File::create(output_path)?;
        serde_json::to_writer(&mut output_file, &data.0).context("writing JSON output")?;
        output_file.sync_all().context("flushing JSON output")?;

        Ok(intermediates::NoData.into())
    }
}
