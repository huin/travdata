//! Data types that configure an aspect of extraction processing.

mod core_type;
#[cfg(test)]
mod parse_tests;
#[cfg(any(test, feature = "testing"))]
mod test_defaults;

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

pub use core_type::{NodeId, Tag};

/// Generic wrapper and properties of an extraction configuration node.
///
/// `S` is the spec type.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GenericNode<S> {
    pub id: core_type::NodeId,
    // TODO: Use hashbrown::HashSet instead.
    #[serde(default)]
    pub tags: HashSet<core_type::Tag>,
    #[serde(default)]
    pub public: bool,
    #[serde(flatten)]
    pub spec: S,
}
