//! Arguments for a [crate::pipeline::GenericPipeline].
//!
//! These provide runtime parameters for the [crate::plparams] for the pipeline.

use crate::{
    PipelineTypes,
    plparams::{self, BorrowedParamKey, ParamKey},
};

#[derive(thiserror::Error)]
pub enum ArgError<P>
where
    P: PipelineTypes,
{
    #[error(
        "required argument value for node {node_id:?} with parameter ID {param_id:?} not found (bug: missing parameter or argument)"
    )]
    NotFound {
        node_id: P::NodeId,
        param_id: plparams::ParamId,
    },
}

impl<P> Clone for ArgError<P>
where
    P: PipelineTypes,
{
    fn clone(&self) -> Self {
        match self {
            Self::NotFound { node_id, param_id } => Self::NotFound {
                node_id: node_id.clone(),
                param_id: param_id.clone(),
            },
        }
    }
}

impl<P> std::fmt::Debug for ArgError<P>
where
    P: PipelineTypes,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { node_id, param_id } => f
                .debug_struct("NotFound")
                .field("node_id", node_id)
                .field("param_id", param_id)
                .finish(),
        }
    }
}

impl<P> Eq for ArgError<P> where P: PipelineTypes {}

impl<P> PartialEq for ArgError<P>
where
    P: PipelineTypes,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::NotFound {
                    node_id: l_node_id,
                    param_id: l_param_id,
                },
                Self::NotFound {
                    node_id: r_node_id,
                    param_id: r_param_id,
                },
            ) => l_node_id == r_node_id && l_param_id == r_param_id,
        }
    }
}

pub struct GenericArgSet<P>
where
    P: PipelineTypes,
{
    args: hashbrown::HashMap<ParamKey<P::NodeId>, P::ArgValue>,
}

impl<P> Default for GenericArgSet<P>
where
    P: PipelineTypes,
{
    fn default() -> Self {
        Self {
            args: Default::default(),
        }
    }
}

impl<P> GenericArgSet<P>
where
    P: PipelineTypes,
{
    pub fn set(&mut self, node_id: P::NodeId, param_id: plparams::ParamId, arg: P::ArgValue) {
        self.args.insert(ParamKey { node_id, param_id }, arg);
    }

    pub fn set_key(&mut self, key: ParamKey<P::NodeId>, arg: P::ArgValue) {
        self.args.insert(key, arg);
    }

    pub fn get<'a>(
        &'a self,
        node_id: &P::NodeId,
        param_id: &plparams::ParamId,
    ) -> Option<&'a P::ArgValue> {
        self.args.get(&BorrowedParamKey::new(node_id, param_id))
    }

    pub fn require<'a>(
        &'a self,
        node_id: &P::NodeId,
        param_id: &plparams::ParamId,
    ) -> Result<&'a P::ArgValue, ArgError<P>> {
        self.get(node_id, param_id)
            .ok_or_else(|| ArgError::NotFound {
                node_id: node_id.clone(),
                param_id: param_id.clone(),
            })
    }
}
