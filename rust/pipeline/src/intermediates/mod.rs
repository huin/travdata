//! Intermediate data types, that are outputs of some [crate::Node] and inputs to others during
//! extraction processing.

use std::path::PathBuf;

pub enum IntermediateValue {
    NoData,
    InputFile(PathBuf),
    JsonData(serde_json::Value),
}

/// Monomorphic form of [generic_pipeline::intermediates::IntermediateSet].
pub type IntermediateSet =
    generic_pipeline::intermediates::GenericIntermediateSet<IntermediateValue>;
