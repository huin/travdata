//! Parameters for a [crate::pipeline::GenericPipeline].

use crate::node;

/// ID of a parameter, within the namespace of the [node::GenericNode] that it is for.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ParamId(pub &'static str);

/// Describes an input parameter for processing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericParam<P> {
    /// ID of the parameter.
    pub param_id: ParamId,
    /// Human-readable description of the parameter.
    pub description: String,
    /// What semenatic type of value of the argument.
    pub param_type: P,
}

/// [Param]s for a single [node::GenericNode].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericParams<P> {
    pub params: Vec<GenericParam<P>>,
}

/// A [Param] qualified by its [node::NodeId].
#[derive(Debug)]
pub struct GenericNodeParam<P> {
    pub node_id: node::NodeId,
    pub param: GenericParam<P>,
}

/// [NodeParam]s for a collection of [node::GenericNode]s.
#[derive(Debug)]
pub struct GenericNodeParams<P> {
    pub params: Vec<GenericNodeParam<P>>,
}
