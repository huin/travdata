//! Parameters for a [crate::Pipeline].

/// Indicates the required semantic type of an argument.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParamType {
    InputPdf,
    OutputDirectory,
}

/// Monomorphic form of [generic_pipeline::plparams::Params].
pub type Params = generic_pipeline::plparams::GenericParams<ParamType>;

/// Monomorphic form of [generic_pipeline::plparams::NodeParams].
pub type NodeParams = generic_pipeline::plparams::GenericNodeParams<ParamType>;
