//! Data types that configure an aspect of extraction processing.

pub mod core_type;
pub mod spec;
pub mod spec_type;
#[cfg(test)]
mod tests;

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// Generic wrapper and properties of an extraction configuration node.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Node {
    pub id: core_type::NodeId,
    #[serde(default)]
    pub tags: HashSet<core_type::Tag>,
    #[serde(default)]
    pub public: bool,
    #[serde(flatten)]
    pub spec: spec::Spec,
}
