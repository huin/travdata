//! Intermediate data types, that are outputs of some [crate::systems::GenericSystem] and inputs to
//! others during extraction processing.

use std::fmt::Debug;

use crate::PipelineTypes;

#[derive(thiserror::Error)]
pub enum IntermediateError<P: PipelineTypes> {
    #[error(
        "required intermediate value from node {node_id:?} not found (bug: missing dependency)"
    )]
    MissingRequired { node_id: P::NodeId },
}

impl<P> Clone for IntermediateError<P>
where
    P: PipelineTypes,
{
    fn clone(&self) -> Self {
        match self {
            Self::MissingRequired { node_id } => Self::MissingRequired {
                node_id: node_id.clone(),
            },
        }
    }
}

impl<P> Debug for IntermediateError<P>
where
    P: PipelineTypes,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingRequired { node_id } => f
                .debug_struct("MissingRequired")
                .field("node_id", node_id)
                .finish(),
        }
    }
}

impl<P> Eq for IntermediateError<P> where P: PipelineTypes {}

impl<P> PartialEq for IntermediateError<P>
where
    P: PipelineTypes,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::MissingRequired { node_id: l_node_id },
                Self::MissingRequired { node_id: r_node_id },
            ) => l_node_id == r_node_id,
        }
    }
}

pub struct GenericIntermediateSet<P: PipelineTypes> {
    intermediates: hashbrown::HashMap<P::NodeId, P::IntermediateValue>,
}

impl<P> Default for GenericIntermediateSet<P>
where
    P: PipelineTypes,
{
    fn default() -> Self {
        Self {
            intermediates: Default::default(),
        }
    }
}

impl<P> GenericIntermediateSet<P>
where
    P: PipelineTypes,
{
    pub fn new() -> Self {
        Self {
            intermediates: Default::default(),
        }
    }

    pub fn set(&mut self, node_id: P::NodeId, intermediate: P::IntermediateValue) {
        self.intermediates.insert(node_id, intermediate);
    }

    pub fn get<'a>(&'a self, node_id: &P::NodeId) -> Option<&'a P::IntermediateValue> {
        self.intermediates.get(node_id)
    }

    pub fn require<'a>(
        &'a self,
        node_id: &P::NodeId,
    ) -> Result<&'a P::IntermediateValue, IntermediateError<P>> {
        self.get(node_id)
            .ok_or_else(|| IntermediateError::MissingRequired {
                node_id: node_id.clone(),
            })
    }
}
