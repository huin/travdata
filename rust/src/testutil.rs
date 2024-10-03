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
