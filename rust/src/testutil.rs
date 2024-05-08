use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use googletest::{
    description::Description,
    matcher::{Matcher, MatcherResult},
};

/// Creates a matcher against an `anyhow::Error` that downcasts to the given
/// type and matches the inner matcher.
pub fn anyhow_downcasts_to<E: Display + Debug + Send + Sync + 'static>(
    inner: impl Matcher<ActualT = E>,
) -> impl Matcher<ActualT = anyhow::Error> {
    AnyhowDowncastTo::<E, _> {
        inner,
        phantom_e: Default::default(),
    }
}

struct AnyhowDowncastTo<E, InnerMatcherT> {
    inner: InnerMatcherT,
    phantom_e: PhantomData<E>,
}

impl<E, InnerMatcherT> AnyhowDowncastTo<E, InnerMatcherT> {
    fn type_name() -> &'static str {
        std::any::type_name::<E>()
    }
}

impl<E: Display + Debug + Send + Sync + 'static, InnerMatcherT: Matcher<ActualT = E>> Matcher
    for AnyhowDowncastTo<E, InnerMatcherT>
{
    type ActualT = anyhow::Error;

    fn matches(&self, actual: &Self::ActualT) -> MatcherResult {
        actual.downcast_ref::<E>()
            .map(|v| self.inner.matches(v))
            .unwrap_or(MatcherResult::NoMatch)
    }

    fn explain_match(&self, actual: &Self::ActualT) -> Description {
        match actual.downcast_ref::<E>() {
            Some(e) => Description::new()
                .text(format!("which is of the expected concrete error type {}", Self::type_name()))
                .text("with value")
                .nested(self.inner.explain_match(e)),
            None => Description::new()
                .text(format!("which is not the expected concrete error type {}", Self::type_name()))
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
