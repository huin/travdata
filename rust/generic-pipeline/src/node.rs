//! Data types that configure an aspect of extraction processing.

use std::hash::Hash;

/// Required trait of a type that identifies a node within a pipeline.
pub trait PipelineNodeId: Clone + std::fmt::Debug + Eq + PartialEq + Hash {}

impl<T> PipelineNodeId for T where T: Clone + std::fmt::Debug + Eq + PartialEq + Hash {}

/// Required trait of a node in a pipeline.
pub trait PipelineNode {
    /// Type for the unique identifier for the node within its pipeline.
    type Id: PipelineNodeId;

    fn id(&self) -> &Self::Id;
}
