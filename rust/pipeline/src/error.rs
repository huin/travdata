use std::sync::Arc;

use generic_pipeline::{intermediates::IntermediateError, plargs::ArgError, plparams::ParamId};
use thiserror::Error;
use thiserror_context::{Context, impl_context};

use crate::NodeId;

pub type SystemResult<T> = std::result::Result<T, SystemError>;

impl_context!(SystemError(SystemErrorKind));

pub(crate) trait StdError: std::error::Error + Send + Sync {}

impl<T> StdError for T where T: std::error::Error + Send + Sync {}

impl Clone for SystemError {
    fn clone(&self) -> Self {
        match self {
            Self::Base(inner) => Self::Base(inner.clone()),
            Self::Context { error, context } => Self::Context {
                error: error.clone(),
                context: context.clone(),
            },
        }
    }
}

/// Error type produced when running a pipeline system.
impl SystemError {
    pub(crate) fn map_execution<E>() -> impl FnOnce(E) -> Self
    where
        E: StdError + 'static,
    {
        |err| SystemErrorKind::Execution(Arc::new(err)).into()
    }

    pub(crate) fn map_input<E>(input_node: &NodeId) -> impl FnOnce(E) -> Self
    where
        E: StdError + 'static,
    {
        |err| {
            SystemErrorKind::Input {
                input_node: input_node.clone(),
                error: Arc::new(err),
            }
            .into()
        }
    }

    pub(crate) fn map_internal<E>() -> impl FnOnce(E) -> Self
    where
        E: StdError + 'static,
    {
        |err| SystemErrorKind::Internal(Arc::new(err)).into()
    }

    pub(crate) fn map_param<E>(param_id: &ParamId) -> impl FnOnce(E) -> Self
    where
        E: StdError + 'static,
    {
        |err| {
            SystemErrorKind::Param {
                param_id: param_id.clone(),
                error: Arc::new(err),
            }
            .into()
        }
    }

    pub(crate) fn map_spec<E>() -> impl FnOnce(E) -> Self
    where
        E: StdError + 'static,
    {
        |err| SystemErrorKind::Spec(Arc::new(err)).into()
    }
}

#[derive(Clone, Debug)]
pub struct StringError(pub(crate) String);

impl StringError {
    pub(crate) fn from_display_error<E>(err: E) -> Self
    where
        E: std::fmt::Display,
    {
        Self(format!("{err}"))
    }
}

impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StringError {}

type ArcStdError = Arc<dyn std::error::Error + Send + Sync>;

/// Specifies the type of system error.
#[derive(Clone, Error, Debug)]
pub enum SystemErrorKind {
    /// Could not use an argument for a node.
    #[error("argument error: {0}")]
    Arg(
        #[from]
        #[source]
        ArgError,
    ),
    /// System execution encountered some general error.
    #[error("system execution error: {0}")]
    Execution(#[source] ArcStdError),
    /// Error while using input data from another node.
    #[error("error processing input data from node {input_node:?}: {error}")]
    Input {
        input_node: NodeId,
        #[source]
        error: ArcStdError,
    },
    /// Could not acquire input data from another node.
    #[error("intermediate data error: {0}")]
    Intermediate(
        #[from]
        #[source]
        IntermediateError,
    ),
    /// Internal system error (likely a bug or error misclassification).
    #[error("internal error: {0}")]
    Internal(#[source] ArcStdError),
    /// Error while using a given parameter's value.
    #[error("error processing parameter {param_id:?}: {error}")]
    Param {
        param_id: ParamId,
        #[source]
        error: ArcStdError,
    },
    /// Error with the node's specification itself.
    #[error("error in spec: {0}")]
    Spec(#[source] ArcStdError),
}
