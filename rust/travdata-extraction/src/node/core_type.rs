//! Core types used within an extraction configuration [crate::nodes::Node].
//!
//! Many of these have value validation, so their inner value is private.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Unique identifier of an extraction configuration [crate::node::Node] within a bundle.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
// TODO: Validate the ID when deserializing.
pub struct NodeId(String);

impl TryFrom<String> for NodeId {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // TODO: Validate the ID.
        Ok(Self(value))
    }
}

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

/// Tag value that non-uniquely identifies a set of extraction configuration
/// [crate::node::Node](s).
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
// TODO: Validate the ID when deserializing.
pub struct Tag(String);

impl TryFrom<String> for Tag {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // TODO: Validate the ID.
        Ok(Self(value))
    }
}
