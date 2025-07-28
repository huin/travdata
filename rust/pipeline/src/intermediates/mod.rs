//! Intermediate data types, that are outputs of some [crate::Node] and inputs to others during
//! extraction processing.

use std::path::PathBuf;

pub mod es_transform;

pub enum Intermediate {
    NoData,
    EsTransform(es_transform::EsTransform),
    InputFile(PathBuf),
    JsonData(serde_json::Value),
}
