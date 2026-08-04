#[cfg(test)]
mod tests;

use generic_pipeline::systems::GenericSystem;
use thiserror_context::Context;

use crate::{
    StringError, SystemError, SystemResult, intermediates,
    monomorph::{ArgSet, NodeInputsRegistrator},
    specs,
};

pub struct OutputFileCsvSystem;

impl GenericSystem<crate::PipelineTypes> for OutputFileCsvSystem {
    fn inputs(&self, node: &crate::Node, reg: &mut NodeInputsRegistrator) -> SystemResult<()> {
        let spec: &specs::OutputFileCsv = node.spec.downcast()?;
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
        let spec: &specs::OutputFileCsv = node.spec.downcast()?;
        let directory: &intermediates::OutputDirectory =
            intermediates::get_intermediate_input(intermediates, &spec.directory)?;
        let data: &intermediates::JsonData =
            intermediates::get_intermediate_input(intermediates, &spec.input_data)?;

        let data = data
            .0
            .as_array()
            .ok_or(StringError("input data is not a JSON array".into()))
            .map_err(SystemError::map_input_value(&spec.input_data))?;

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
                .map_err(SystemError::map_input_value(&spec.input_data))?;

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
                    .map_err(SystemError::map_input_value(&spec.input_data))?;

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
