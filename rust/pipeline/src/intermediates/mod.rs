//! Intermediate data types, that are outputs of some [crate::node::Node] and inputs to others
//! during extraction processing.

use std::path::PathBuf;

use crate::node;

pub mod es_transform;

pub enum Intermediate {
    NoData,
    EsTransform(es_transform::EsTransform),
    InputFile(PathBuf),
    JsonData(serde_json::Value),
}

#[derive(Default)]
pub struct IntermediateSet {
    intermediates: hashbrown::HashMap<node::NodeId, Intermediate>,
}

impl IntermediateSet {
    pub fn new() -> Self {
        Self {
            intermediates: Default::default(),
        }
    }

    pub fn set(&mut self, node_id: node::NodeId, intermediate: Intermediate) {
        self.intermediates.insert(node_id, intermediate);
    }

    pub fn get<'a>(&'a self, node_id: &node::NodeId) -> Option<&'a Intermediate> {
        self.intermediates.get(node_id)
    }
}
