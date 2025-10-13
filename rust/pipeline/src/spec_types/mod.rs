//! Types used within extraction configuration specification types.

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod pdf;

/// Relative path to an output file within a runtime-specified directory.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
// TODO: Validate the path when deserializing. Should be a relative-and-subdir-only value.
pub struct OutputPathBuf(PathBuf);

impl AsRef<Path> for OutputPathBuf {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl OutputPathBuf {
    // TODO: Ideally this would be a TryFrom, but there's a conflicting blanket impl in the stdlib,
    // and I haven't debugged how to avoid that.
    pub fn new<P>(value: P) -> Result<Self>
    where
        P: Into<PathBuf> + AsRef<Path>,
    {
        // TODO: Validate the path.
        Ok(Self(value.into()))
    }
}
