//! Arguments for a [crate::Pipeline].
//!
//! These provide runtime parameters for the [crate::plparams] for the pipeline.

use std::path::PathBuf;

/// Typed value of an argument to a [crate::Node].
pub enum Arg {
    InputPdf(PathBuf),
    OutputDirectory(PathBuf),
}
