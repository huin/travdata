//! Parameters for a [crate::Pipeline].

/// Indicates the required semantic type of an argument.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParamType {
    InputPdf,
    OutputDirectory,
}

/// Monomorphic form of [generic_pipeline::plparams::GenericParams].
pub type Params = generic_pipeline::plparams::GenericParams<ParamType>;

/// Monomorphic form of [generic_pipeline::plparams::GenericParamsRegistrator].
pub type ParamsRegistrator = generic_pipeline::plparams::GenericParamsRegistrator<ParamType>;

/// Monomorphic form of [generic_pipeline::plparams::GenericNodeParamsRegistrator].
pub type NodeParamsRegistrator<'a> =
    generic_pipeline::plparams::GenericNodeParamsRegistrator<'a, ParamType>;
