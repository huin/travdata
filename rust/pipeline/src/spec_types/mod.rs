//! Types used within extraction configuration specification types.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub mod pdf;

/// Relative path to an output file within a runtime-specified directory.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
// TODO: Validate the path when deserializing. Should be a relative-and-subdir-only value.
pub struct OutputPathBuf(PathBuf);

impl TryFrom<PathBuf> for OutputPathBuf {
    type Error = anyhow::Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        // TODO: Validate the path.
        Ok(Self(value))
    }
}
