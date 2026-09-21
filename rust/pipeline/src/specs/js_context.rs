use serde::{Deserialize, Serialize};
#[cfg(any(test, feature = "testing"))]
use testutils::DefaultForTest;

// TODO: Consider reserving the name `default-js-context`, which is lazily created implicitly at run
// time, and dependencies on a JsContext default to `default-js-context`. Maybe similar for output
// directory?

/// Defines a JavaScript context for evaluating JavaScript within.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JsContext;

#[cfg(any(test, feature = "testing"))]
impl DefaultForTest for JsContext {
    fn default_for_test() -> Self {
        Self
    }
}
