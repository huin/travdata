//! Arguments for a [crate::Pipeline].
//!
//! These provide runtime parameters for the [crate::plparams] for the pipeline.

use std::path::PathBuf;

use crate::impl_enum_conversions;

/// Monomorphic form of [generic_pipeline::plargs::GenericArgSet].
pub type ArgSet = generic_pipeline::plargs::GenericArgSet<ArgValue>;

/// Typed value of an argument to a [crate::Node].
#[derive(Debug)]
pub enum ArgValue {
    InputPdf(InputPdf),
    OutputDirectory(OutputDirectory),
}

#[derive(Debug)]
pub struct InputPdf(pub PathBuf);

#[derive(Debug)]
pub struct OutputDirectory(pub PathBuf);

impl_enum_conversions!(ArgValue, InputPdf, "argument value");
impl_enum_conversions!(ArgValue, OutputDirectory, "argument value");
