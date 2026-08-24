//! Types used within extraction configuration specification types.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
#[cfg(any(test, feature = "testing"))]
use testutils::DefaultForTest;

use crate::SystemResult;

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
    pub fn new<P>(value: P) -> SystemResult<Self>
    where
        P: Into<PathBuf> + AsRef<Path>,
    {
        // TODO: Validate the path.
        Ok(Self(value.into()))
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn new_for_test<P>(value: P) -> crate::spec_types::OutputPathBuf
    where
        P: Into<PathBuf> + AsRef<Path>,
    {
        crate::spec_types::OutputPathBuf::new(value).expect("expected valid OutputPathBufValue")
    }
}

#[cfg(any(test, feature = "testing"))]
impl DefaultForTest for OutputPathBuf {
    fn default_for_test() -> Self {
        Self(PathBuf::from("fake-output-path.txt"))
    }
}
