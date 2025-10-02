//! Intermediate data types, that are outputs of some [crate::Node] and inputs to others during
//! extraction processing.

use std::path::PathBuf;

#[derive(Debug, Eq, PartialEq)]
pub enum IntermediateValue {
    NoData,
    InputFile(PathBuf),
    JsContext(v8::Global<v8::Context>),
    JsonData(serde_json::Value),
    OutputDirectory(PathBuf),
}

/// Monomorphic form of [generic_pipeline::intermediates::GenericIntermediateSet].
pub type IntermediateSet =
    generic_pipeline::intermediates::GenericIntermediateSet<IntermediateValue>;
