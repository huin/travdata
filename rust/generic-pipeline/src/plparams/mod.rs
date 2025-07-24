//! Parameters for a [crate::pipeline::GenericPipeline].

use crate::node;

/// ID of a parameter, within the namespace of the [node::Node] that it is for.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ParamId(pub &'static str);

/// Describes an input parameter for processing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Param {
    /// ID of the parameter.
    pub param_id: ParamId,
    /// Human-readable description of the parameter.
    pub description: String,
    /// What semenatic type of value of the argument.
    pub param_type: ParamType,
}

/// Indicates the required semantic type of an argument.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParamType {
    InputPdf,
    OutputDirectory,
}

/// [Param]s for a single [node::Node].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Params {
    pub params: Vec<Param>,
}

/// A [Param] qualified by its [node::NodeId].
#[derive(Debug)]
pub struct NodeParam {
    pub node_id: node::NodeId,
    pub param: Param,
}

/// [NodeParam]s for a collection of [node::Node]s.
#[derive(Debug)]
pub struct NodeParams {
    pub params: Vec<NodeParam>,
}
