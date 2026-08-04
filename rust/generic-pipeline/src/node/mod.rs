//! Data types that configure an aspect of extraction processing.

mod core_type;
#[cfg(test)]
mod parse_tests;
#[cfg(any(test, feature = "testing"))]
mod test_defaults;

use hashbrown::HashSet;
use serde::{Deserialize, Serialize};

pub use core_type::{NodeId, Tag};

/// Generic wrapper and properties of an extraction configuration node.
///
/// `S` is the spec type.
///
/// TODO: Consider moving except `id` and `spec` into a generic type for metadata, as
/// `generic_pipeline` does not consume those fields. Or even just delete them and allow for the
/// caller to wrap the node with any metadata type.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GenericNode<S> {
    pub id: core_type::NodeId,
    #[serde(default)]
    pub tags: HashSet<core_type::Tag>,
    #[serde(default)]
    pub public: bool,
    #[serde(flatten)]
    pub spec: S,
}
