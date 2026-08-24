use serde::{Deserialize, Serialize};
#[cfg(any(test, feature = "testing"))]
use testutils::DefaultForTest;

use crate::generic::node::{NodeIdTransformer, TranslateFromNodeId};

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

impl<FromNodeId, NodeId> TranslateFromNodeId<FromNodeId> for JsTransform<NodeId> {
    type FromType = JsTransform<FromNodeId>;
    type NodeId = NodeId;

    fn transform_node_ids<
        Transformer: NodeIdTransformer<FromNodeId = FromNodeId, ToNodeId = Self::NodeId>,
    >(
        from: Self::FromType,
        trn: &Transformer,
    ) -> Result<Self, Transformer::Error> {
        Ok(Self {
            context: trn.transform_node_id(from.context)?,
            input_data: from
                .input_data
                .into_iter()
                .map(|(name, node_id)| Ok((name, trn.transform_node_id(node_id)?)))
                .collect::<Result<hashbrown::HashMap<String, NodeId>, Transformer::Error>>()?,
            code: from.code.clone(),
        })
    }
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
