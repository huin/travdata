use serde::{Deserialize, Serialize};

/// Specifies the transformation of data using ECMAScript.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EsTransform {
    // TODO: Consider making this a map from ECMAScript parameter name to NodeId.
    pub input_data: crate::NodeId,
    pub code: String,
}
