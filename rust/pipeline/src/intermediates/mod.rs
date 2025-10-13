//! Intermediate data types, that are outputs of some [crate::Node] and inputs to others during
//! extraction processing.

use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};

use crate::{impl_enum_conversions, spec_types::OutputPathBuf};

/// Monomorphic form of [generic_pipeline::intermediates::GenericIntermediateSet].
pub type IntermediateSet =
    generic_pipeline::intermediates::GenericIntermediateSet<IntermediateValue>;

#[derive(Debug, Eq, PartialEq)]
pub enum IntermediateValue {
    NoData(NoData),
    InputFile(InputFile),
    JsContext(JsContext),
    JsonData(JsonData),
    OutputDirectory(OutputDirectory),
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
    pub fn create_parent_dirs_for_file(&self, file_path: &OutputPathBuf) -> Result<PathBuf> {
        let qualified_file_path = self.0.join(&file_path);
        std::fs::DirBuilder::new()
            .recursive(true)
            .create(qualified_file_path.parent().ok_or_else(|| {
                anyhow!("{qualified_file_path:?} does not have a parent directory")
            })?)
            .context("creating parent directory")?;
        Ok(qualified_file_path)
    }
}

impl_enum_conversions!(IntermediateValue, NoData, "intermediate value");
impl_enum_conversions!(IntermediateValue, InputFile, "intermediate value");
impl_enum_conversions!(IntermediateValue, JsContext, "intermediate value");
impl_enum_conversions!(IntermediateValue, JsonData, "intermediate value");
impl_enum_conversions!(IntermediateValue, OutputDirectory, "intermediate value");
