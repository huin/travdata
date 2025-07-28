//! Core types used within an extraction configuration [crate::node::GenericNode].
//!
//! Many of these have value validation, so their inner value is private.

use serde::{Deserialize, Serialize};

/// Unique identifier of an extraction configuration [crate::node::GenericNode] within a
/// [crate::pipeline::GenericPipeline].
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
// TODO: Validate the ID when deserializing.
pub struct NodeId(String);

impl From<&NodeId> for NodeId {
    fn from(value: &NodeId) -> Self {
        value.clone()
    }
}

impl TryFrom<String> for NodeId {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // TODO: Validate the ID.
        Ok(Self(value))
    }
}

/// Tag value that non-uniquely identifies a set of extraction configuration
/// [crate::node::GenericNode]s.
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
