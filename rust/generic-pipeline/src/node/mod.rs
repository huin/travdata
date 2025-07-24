//! Data types that configure an aspect of extraction processing.

mod core_type;
#[cfg(test)]
mod parse_tests;

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

pub use core_type::{NodeId, Tag};

pub trait SpecTrait:
    std::fmt::Debug + for<'a> Deserialize<'a> + Eq + PartialEq + Serialize
{
    type Discrim: std::fmt::Debug + Eq + std::hash::Hash;

    fn discriminant(&self) -> Self::Discrim;
}

/// Generic wrapper and properties of an extraction configuration node.
///
/// `S` is the spec type.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GenericNode<S> {
    pub id: core_type::NodeId,
    #[serde(default)]
    pub tags: HashSet<core_type::Tag>,
    #[serde(default)]
    pub public: bool,
    #[serde(flatten)]
    pub spec: S,
}
