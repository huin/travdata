//! Arguments for a [crate::Pipeline].
//!
//! These provide runtime parameters for the [crate::plparams] for the pipeline.

use std::path::PathBuf;

use generic_pipeline::plparams::ParamId;

use crate::{NodeId, StringError, SystemError, SystemResult, impl_enum_conversions};

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

pub fn get_arg<'a, T>(args: &'a ArgSet, node_id: &NodeId, param_id: &ParamId) -> SystemResult<&'a T>
where
    &'a T: TryFrom<&'a ArgValue, Error = StringError>,
{
    args.require(node_id, param_id)
        .map_err(SystemError::from)?
        .try_into()
        .map_err(SystemError::map_param(param_id))
}
