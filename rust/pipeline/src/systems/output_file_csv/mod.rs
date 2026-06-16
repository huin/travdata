#[cfg(test)]
mod tests;

use generic_pipeline::systems::GenericSystem;
use thiserror_context::Context;

use crate::{StringError, SystemError, SystemResult, intermediates, specs};

pub struct OutputFileCsvSystem;

impl GenericSystem<crate::PipelineTypes> for OutputFileCsvSystem {
    fn inputs<'a>(
        &self,
        node: &generic_pipeline::node::GenericNode<
            <crate::PipelineTypes as generic_pipeline::PipelineTypes>::Spec,
        >,
        reg: &'a mut generic_pipeline::plinputs::NodeInputsRegistrator<'a>,
    ) -> SystemResult<()> {
        let spec: &specs::OutputFileCsv = node.spec.downcast_spec()?;
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
        let spec: &specs::OutputFileCsv = node.spec.downcast_spec()?;
        let directory: &intermediates::OutputDirectory =
            intermediates::get_intermediate_input(intermediates, &spec.directory)?;
        let data: &intermediates::JsonData =
            intermediates::get_intermediate_input(intermediates, &spec.input_data)?;

        let data = data
            .0
            .as_array()
            .ok_or(StringError("input data is not a JSON array".into()))
            .map_err(SystemError::map_input(&spec.input_data))?;

        let output_path = directory
            .create_parent_dirs_for_file(&spec.filename)
            .map_err(SystemError::map_execution())
            .context("creating parent directory for output data")?;

        let mut output = csv::WriterBuilder::new()
            .terminator(csv::Terminator::CRLF)
            .flexible(true)
            .from_path(output_path)
            .map_err(SystemError::map_execution())
            .context("opening CSV writer")?;

        let mut row_fields: Vec<&[u8]> = Vec::new();
        for (row_index, row) in data.iter().enumerate() {
            let row = row
                .as_array()
                .ok_or_else(|| {
                    StringError(format!("output data [{row_index}] is not a JSON array"))
                })
                .map_err(SystemError::map_input(&spec.input_data))?;

            row_fields.resize(row.len(), Default::default());

            for (field_index, field) in row.iter().enumerate() {
                // For now, only str supported. Leave open the interpretation of other types for
                // future decisions, for now anything else is an error.
                let field = field
                    .as_str()
                    .ok_or_else(|| {
                        StringError(format!(
                            "output data [{row_index}][{field_index}] is not a JSON string"
                        ))
                    })
                    .map_err(SystemError::map_input(&spec.input_data))?;

                row_fields[field_index] = field.as_bytes();
            }

            output
                .write_record(&row_fields[0..row.len()])
                .map_err(SystemError::map_execution())
                .with_context(|| format!("writing row index {row_index}"))?;
        }

        output
            .flush()
            .map_err(SystemError::map_execution())
            .context("flushing CSV output")?;
        drop(output);

        Ok(intermediates::NoData.into())
    }
}
