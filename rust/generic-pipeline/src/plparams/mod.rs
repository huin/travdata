//! Parameters for a [crate::pipeline::GenericPipeline].

use std::fmt::Debug;

use hashbrown::HashMap;

use crate::{PipelineTypes, node::PipelineNodeId};

/// ID of a parameter, within the context of the [crate::node::PipelineNode] that it is for.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ParamId(&'static str);

impl ParamId {
    pub const fn from_static(id: &'static str) -> Self {
        Self(id)
    }
}

impl AsRef<str> for ParamId {
    fn as_ref(&self) -> &str {
        self.0
    }
}

/// Describes an input parameter for processing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericParam<ParamType> {
    /// Human-readable description of the parameter.
    pub description: String,
    /// What semenatic type of value of the argument.
    pub param_type: ParamType,
}

/// A [GenericParam] qualified by its [node::NodeId].
#[derive(Debug)]
pub struct GenericNodeParam<NodeId, ParamType> {
    pub node_id: NodeId,
    pub param: GenericParam<ParamType>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ParamKey<NodeId: PipelineNodeId> {
    pub node_id: NodeId,
    pub param_id: ParamId,
}

impl<NodeId> ParamKey<NodeId>
where
    NodeId: PipelineNodeId,
{
    /// Creates a new [ParamKey].
    pub fn new(node_id: NodeId, param_id: ParamId) -> Self {
        Self { node_id, param_id }
    }
}

#[derive(Hash)]
pub(crate) struct BorrowedParamKey<'a, NodeId>
where
    NodeId: PipelineNodeId,
{
    node_id: &'a NodeId,
    param_id: &'a ParamId,
}

impl<'a, NodeId> BorrowedParamKey<'a, NodeId>
where
    NodeId: PipelineNodeId,
{
    pub(crate) fn new(node_id: &'a NodeId, param_id: &'a ParamId) -> Self {
        Self { node_id, param_id }
    }
}

impl<'a, NodeId> hashbrown::Equivalent<ParamKey<NodeId>> for BorrowedParamKey<'a, NodeId>
where
    NodeId: PipelineNodeId,
{
    fn equivalent(&self, key: &ParamKey<NodeId>) -> bool {
        self.node_id == &key.node_id && self.param_id == &key.param_id
    }
}

pub struct GenericParams<P: PipelineTypes> {
    pub params: HashMap<ParamKey<P::NodeId>, GenericParam<P::ParamType>>,
}

impl<P> GenericParams<P>
where
    P: PipelineTypes,
{
    pub fn registrator() -> GenericParamsRegistrator<P> {
        GenericParamsRegistrator {
            params: Default::default(),
        }
    }
}

impl<P> Debug for GenericParams<P>
where
    P: PipelineTypes,
    P::ParamType: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericParams")
            .field("params", &self.params)
            .finish()
    }
}

impl<P> PartialEq for GenericParams<P>
where
    P: PipelineTypes,
    P::ParamType: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.params == other.params
    }
}

/// Registers pipeline parameters for nodes.
pub struct GenericParamsRegistrator<P: PipelineTypes> {
    params: HashMap<ParamKey<P::NodeId>, GenericParam<P::ParamType>>,
}

impl<P> GenericParamsRegistrator<P>
where
    P: PipelineTypes,
{
    /// Returns a [GenericNodeParamsRegistrator] for registering parameters for the given
    /// [node::NodeId].
    pub fn for_node<'a>(
        &'a mut self,
        node_id: &'a P::NodeId,
    ) -> GenericNodeParamsRegistrator<'a, P> {
        GenericNodeParamsRegistrator { node_id, reg: self }
    }

    /// Consumes the [GenericParamsRegistrator] and returns the built up parameters.
    pub fn build(self) -> GenericParams<P> {
        GenericParams {
            params: self.params,
        }
    }
}

/// Registers pipeline parameters for a single node.
pub struct GenericNodeParamsRegistrator<'a, P: PipelineTypes> {
    node_id: &'a P::NodeId,
    reg: &'a mut GenericParamsRegistrator<P>,
}

impl<'builder, P> GenericNodeParamsRegistrator<'builder, P>
where
    P: PipelineTypes,
{
    /// Registers a single parameter for the [node::NodeId].
    pub fn add_param(&mut self, param_id: ParamId, param_type: P::ParamType, description: String) {
        self.reg.params.insert(
            ParamKey::new(self.node_id.clone(), param_id),
            GenericParam {
                param_type,
                description,
            },
        );
    }
}
