use serde::{Deserialize, Serialize};
#[cfg(any(test, feature = "testing"))]
use testutils::DefaultForTest;

/// Specifies the transformation of data using ECMAScript.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JsTransform<NodeId> {
    /// Node ID of the [super::js_context::JsContext] to use for evaluation.
    pub context: NodeId,
    /// Maps from function parameter name to [crate::NodeId] that the intermediate data is from.
    ///
    /// E.g. `{"param1": "node-1", "param2": "node-2"}`
    pub input_data: hashbrown::HashMap<String, NodeId>,
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

#[cfg(any(test, feature = "testing"))]
impl<NodeId> DefaultForTest for JsTransform<NodeId>
where
    NodeId: DefaultForTest,
{
    fn default_for_test() -> Self {
        Self {
            context: NodeId::default_for_test(),
            input_data: Default::default(),
            code: "return {}".into(),
        }
    }
}
