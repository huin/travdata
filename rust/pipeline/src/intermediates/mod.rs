//! Intermediate data types, that are outputs of some [crate::Node] and inputs to others during
//! extraction processing.

use std::path::PathBuf;

use thiserror_context::Context;

use crate::{
    NodeId, StringError, SystemError, SystemResult, impl_enum_conversions,
    spec_types::OutputPathBuf,
};

/// Monomorphic form of [generic_pipeline::intermediates::GenericIntermediateSet].
pub type IntermediateSet =
    generic_pipeline::intermediates::GenericIntermediateSet<crate::PipelineTypes>;

#[derive(Debug, Eq, PartialEq)]
pub enum IntermediateValue {
    NoData(NoData),
    InputFile(InputFile),
    JsContext(JsContext),
    JsonData(JsonData),
    OutputDirectory(OutputDirectory),
}

impl PartialEq<IntermediateValue> for &IntermediateValue {
    fn eq(&self, other: &IntermediateValue) -> bool {
        <IntermediateValue as PartialEq>::eq(self, other)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct NoData;

#[derive(Debug, Eq, PartialEq)]
pub struct InputFile(pub PathBuf);

#[derive(Debug, Eq, PartialEq)]
pub struct JsContext(pub v8::Global<v8::Context>);

#[derive(Debug, Eq, PartialEq)]
pub struct JsonData(pub serde_json::Value);

#[derive(Debug, Eq, PartialEq)]
pub struct OutputDirectory(pub PathBuf);

impl OutputDirectory {
    /// Creates parent directories for the given file (relative to the output directory), and
    /// returns the path to the file.
    pub fn create_parent_dirs_for_file(&self, file_path: &OutputPathBuf) -> SystemResult<PathBuf> {
        let qualified_file_path = self.0.join(file_path);
        let qualified_dir_path = qualified_file_path
            .parent()
            .ok_or_else(|| {
                StringError(format!(
                    "{qualified_file_path:?} does not have a parent directory"
                ))
            })
            // Internal error as there should always be a parent directory for
            // qualified_file_path (self.0 or a subdir thereof).
            .map_err(SystemError::map_internal())?;
        std::fs::DirBuilder::new()
            .recursive(true)
            .create(qualified_dir_path)
            .map_err(SystemError::map_execution())
            .with_context(|| "creating parent directory for output file {file_path:?}")?;
        Ok(qualified_file_path)
    }
}

impl_enum_conversions!(IntermediateValue, NoData, "intermediate value");
impl_enum_conversions!(IntermediateValue, InputFile, "intermediate value");
impl_enum_conversions!(IntermediateValue, JsContext, "intermediate value");
impl_enum_conversions!(IntermediateValue, JsonData, "intermediate value");
impl_enum_conversions!(IntermediateValue, OutputDirectory, "intermediate value");

pub fn get_intermediate_input<'i, T>(
    interms: &'i IntermediateSet,
    input_node_id: &NodeId,
) -> SystemResult<&'i T>
where
    &'i T: TryFrom<&'i IntermediateValue, Error = StringError>,
{
    interms
        .require(input_node_id)
        .map_err(SystemError::from)?
        .try_into()
        .map_err(SystemError::map_input_value(input_node_id))
}
