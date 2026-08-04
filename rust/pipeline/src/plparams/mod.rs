//! Parameters for a [crate::Pipeline].

/// Indicates the required semantic type of an argument.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParamType {
    InputPdf,
    OutputDirectory,
}
