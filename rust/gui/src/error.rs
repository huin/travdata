use std::sync::Arc;

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

pub(crate) trait StdError: std::error::Error + Send + Sync {}

impl<T> StdError for T where T: std::error::Error + Send + Sync {}

pub type ArcStdError = Arc<dyn std::error::Error + Send + Sync>;
