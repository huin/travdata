use serde::{Deserialize, Serialize};

/// Defines a JavaScript context for evaluating JavaScript within.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JsContext;

#[cfg(test)]
impl testutils::DefaultForTest for JsContext {
    fn default_for_test() -> Self {
        Self
    }
}
