/// ID of a parameter, within the namespace of the [crate::node::Node] that it is for.
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

/// [Param]s for a single [crate::node::Node].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeParams {
    pub params: Vec<Param>,
}
