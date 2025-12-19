use std::path::PathBuf;

pub type PipelineNodes = Vec<pipeline::Node>;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct PathSelection {
    pub path: PathBuf,
    pub as_string: String,
}
