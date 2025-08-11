//! Utilities used in tests in multiple crates within the workspace.

use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use googletest::{
    description::Description,
    matcher::{Matcher, MatcherBase, MatcherResult},
};

/// Creates a matcher against an `anyhow::Error` that downcasts to the given
/// type and matches the inner matcher.
pub fn anyhow_downcasts_to<E, M>(inner: M) -> AnyhowDowncastTo<E, M> {
    AnyhowDowncastTo::<E, M> {
        inner,
        phantom_e: Default::default(),
    }
}

pub struct AnyhowDowncastTo<E, M> {
    inner: M,
    phantom_e: PhantomData<E>,
}

impl<E, M> AnyhowDowncastTo<E, M> {
    fn type_name() -> &'static str {
        std::any::type_name::<E>()
    }
}

impl<E, M> MatcherBase for AnyhowDowncastTo<E, M> {}

impl<E, M> Matcher<&anyhow::Error> for AnyhowDowncastTo<E, M>
where
    E: Copy + Display + Debug + Send + Sync + 'static,
    M: Matcher<E>,
{
    fn matches(&self, actual: &anyhow::Error) -> MatcherResult {
        actual
            .downcast_ref::<E>()
            .map(|v| self.inner.matches(*v))
            .unwrap_or(MatcherResult::NoMatch)
    }

    fn explain_match(&self, actual: &anyhow::Error) -> Description {
        match actual.downcast_ref::<E>() {
            Some(e) => Description::new()
                .text(format!(
                    "which is of the expected concrete error type {}",
                    Self::type_name()
                ))
                .text("with value")
                .nested(self.inner.explain_match(*e)),
            None => Description::new().text(format!(
                "which is not the expected concrete error type {}",
                Self::type_name()
            )),
        }
    }

    fn describe(&self, matcher_result: MatcherResult) -> Description {
        match matcher_result {
            MatcherResult::Match => format!(
                "is of concrete error type {} with value which {}",
                Self::type_name(),
                self.inner.describe(MatcherResult::Match)
            )
            .into(),
            MatcherResult::NoMatch => format!(
                "is or is not a concrete error type {} with value which {}",
                Self::type_name(),
                self.inner.describe(MatcherResult::NoMatch)
            )
            .into(),
        }
    }
}

/// Adapts [anyhow::Error] to [std::error::Error] to make it compatible with [googletest] tests
/// that use fixtures.
#[derive(Debug)]
pub struct WrappedError(anyhow::Error);

pub trait WrapError<T> {
    fn wrap_error(self) -> std::result::Result<T, WrappedError>;
}

/// Trait to convert an [anyhow::Result] to a [std::result::Result<T, WrappedError>].
impl<T> WrapError<T> for anyhow::Result<T> {
    fn wrap_error(self) -> std::result::Result<T, WrappedError> {
        self.map_err(WrappedError::from)
    }
}

impl std::fmt::Display for WrappedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl std::error::Error for WrappedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<anyhow::Error> for WrappedError {
    fn from(value: anyhow::Error) -> Self {
        Self(value)
    }
}

pub trait DefaultForTest {
    fn default_for_test() -> Self;
}
