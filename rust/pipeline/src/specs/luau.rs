use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
#[cfg(any(test, feature = "testing"))]
use testutils::DefaultForTest;

use crate::NodeId;

/// Defines a Luau context for evaluating Luau within.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LuauContext;

#[cfg(any(test, feature = "testing"))]
impl DefaultForTest for LuauContext {
    fn default_for_test() -> Self {
        Self
    }
}

/// Specifies the transformation of data using Luau.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LuauTransform {
    /// Node ID of the [LuauContext] to use for evaluation.
    pub context: NodeId,
    /// Maps from parameter table field name to [crate::NodeId] that the intermediate data is from.
    ///
    /// E.g.
    /// ```json
    /// "input_data": {
    ///     "param1": "node-1",
    ///     "param2": "node-2"
    /// }
    /// ```
    pub input_data: HashMap<String, NodeId>,
    /// Body of a JavaScript function that receives each named parameter from `input_data`, and
    /// returns the [crate::Node]'s intermediate data. The named arguments from `input_data` will
    /// be in scope and be provided with values when the code is run.
    ///
    /// E.g.
    ///
    /// ```luau
    /// local arg = ...
    /// local result = {}
    /// for _, t in ipairs({arg.param1, arg.param2}) do
    ///     for _, i in ipairs(t) do
    ///         table.insert(result, i)
    ///     end
    /// end
    /// return result
    /// ```
    pub code: String,
}

#[cfg(any(test, feature = "testing"))]
impl DefaultForTest for LuauTransform {
    fn default_for_test() -> Self {
        Self {
            context: NodeId::default_for_test(),
            input_data: Default::default(),
            code: "return {}".into(),
        }
    }
}
