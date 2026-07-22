use serde::{Deserialize, Serialize};

// TODO: Consider reserving the name `default-js-context`, which is lazily created implicitly at run
// time, and dependencies on a JsContext default to `default-js-context`. Maybe similar for output
// directory?

/// Defines a JavaScript context for evaluating JavaScript within.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JsContext;

#[cfg(test)]
impl testutils::DefaultForTest for JsContext {
    fn default_for_test() -> Self {
        Self
    }
}
