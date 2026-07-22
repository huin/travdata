use crate::app::{
    data::{self, NodeRef, node_index},
    ddo,
};

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct EditablePipeline {
    // TODO: Consider using a 3rd party generational arena instead of NodeSet.
    // TODO: Consider making these fields more private so they can be kept in sync with each other.
    pub nodes: data::NodeSet,
    pub node_order: Vec<NodeRef>,
    pub node_index: data::node_index::NodeIndex,
}

impl EditablePipeline {
    pub fn nodes_in_order(&self) -> impl Iterator<Item = Option<&pipeline::Node>> {
        self.node_order
            .iter()
            .map(|&node_ref| self.nodes.get(node_ref))
    }
}

impl TryFrom<ddo::PipelineNodes> for EditablePipeline {
    type Error = String;

    fn try_from(pipeline: ddo::PipelineNodes) -> Result<Self, Self::Error> {
        let mut nodes = data::NodeSet::with_capacity(pipeline.len());
        let mut node_order: Vec<NodeRef> = Vec::with_capacity(pipeline.len());
        let mut node_index = node_index::NodeIndex::with_capacity(pipeline.len());

        for node in pipeline.into_iter() {
            let node_ref = nodes.add_node(node)?;
            let node = match nodes.get(node_ref) {
                Some(node) => node,
                None => {
                    return Err(
                        "bug: internal error, just inserted Node is absent from NodeSet"
                            .to_string(),
                    );
                }
            };
            node_order.push(node_ref);
            node_index.index_node(node_ref, node);
        }

        Ok(Self {
            nodes,
            node_order,
            node_index,
        })
    }
}
