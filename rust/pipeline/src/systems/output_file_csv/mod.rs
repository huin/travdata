#[cfg(test)]
mod tests;

use anyhow::{Context, anyhow};
use generic_pipeline::systems::GenericSystem;

use crate::{intermediates, specs};

pub struct OutputFileCsvSystem;

impl GenericSystem<crate::PipelineTypes> for OutputFileCsvSystem {
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
        let spec = <&specs::OutputFileCsv>::try_from(&node.spec)?;
        let directory: &intermediates::OutputDirectory = intermediates
            .require(&spec.directory)?
            .try_into()
            .context("getting output directory")?;
        let data: &intermediates::JsonData = intermediates
            .require(&spec.input_data)?
            .try_into()
            .context("getting data to output")?;

        let data = data
            .0
            .as_array()
            .ok_or_else(|| anyhow!("output data is not a JSON array"))?;

        let output_path = directory
            .create_parent_dirs_for_file(&spec.filename)
            .context("creating parent directory for output data")?;

        let mut output = csv::WriterBuilder::new()
            .terminator(csv::Terminator::CRLF)
            .flexible(true)
            .from_path(output_path)
            .context("opening CSV writer")?;

        let mut row_fields: Vec<&[u8]> = Vec::new();
        for (row_index, row) in data.iter().enumerate() {
            let row = row
                .as_array()
                .ok_or_else(|| anyhow!("output data [{row_index}] is not a JSON array"))?;

            row_fields.resize(row.len(), Default::default());

            for (field_index, field) in row.iter().enumerate() {
                // For now, only str supported. Leave open the interpretation of other types for
                // future decisions, for now anything else is an error.
                let field = field.as_str().ok_or_else(|| {
                    anyhow!("output data [{row_index}][{field_index}] is not a JSON string")
                })?;

                row_fields[field_index] = field.as_bytes();
            }

            output
                .write_record(&row_fields[0..row.len()])
                .with_context(|| format!("writing row index {row_index}"))?;
        }

        output.flush().context("flushing CSV output")?;
        drop(output);

        Ok(intermediates::NoData.into())
    }
}
