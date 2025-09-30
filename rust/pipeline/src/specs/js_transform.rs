use serde::{Deserialize, Serialize};

/// Specifies the transformation of data using ECMAScript.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JsTransform {
    /// Node ID of the [super::js_context::JsContext] to use for evaluation.
    pub context: crate::NodeId,
    /// Maps from function parameter name to [crate::NodeId] that the intermediate data is from.
    ///
    /// E.g. `{"param1": "node-1", "param2": "node-2"}`
    pub input_data: hashbrown::HashMap<String, crate::NodeId>,
    /// Body of a JavaScript function that receives each named parameter from `input_data`, and
    /// returns the [crate::Node]'s intermediate data. The named arguments from `input_data` will
    /// be in scope and be provided with values when the code is run.
    ///
    /// E.g.
    ///
    /// ```javascript
    /// return param1[0] + param2.data;
    /// ```
    pub code: String,
}

#[cfg(test)]
impl testutils::DefaultForTest for JsTransform {
    fn default_for_test() -> Self {
        use crate::testutil::node_id;

        Self {
            context: node_id("default-test-js-context"),
            input_data: Default::default(),
            code: "return {}".into(),
        }
    }
}
