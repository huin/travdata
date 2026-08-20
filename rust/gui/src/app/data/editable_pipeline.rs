use crate::app::{
    data::{self, NodeRef, node_index},
    ddo,
};

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct EditablePipeline {
    // TODO: Consider using a 3rd party generational arena instead of NodeSet and NodeRef.
    nodes: data::NodeSet,
    node_order: Vec<NodeRef>,
    node_index: data::node_index::NodeIndex,
}

impl EditablePipeline {
    pub fn to_pipeline(&self) -> Result<ddo::PipelineNodes, String> {
        self.node_order
            .iter()
            .map(|&node_ref| {
                self.nodes
                    .get(node_ref)
                    .ok_or_else(|| format!("could not resolve NodeRef {node_ref:?}"))
                    .cloned()
            })
            .collect::<Result<ddo::PipelineNodes, String>>()
    }

    /// Returns the number of nodes in the pipeline.
    pub fn len(&self) -> usize {
        self.node_order.len()
    }

    /// Returns a [NodeContextMut] given its [NodeRef].
    ///
    /// Any modification to the node by the caller should be signaled by a call to
    /// [NodeContextMut::mark_node_changed].
    pub fn get_node_ctx_by_ref_mut<'a>(
        &'a mut self,
        node_ref: NodeRef,
    ) -> Option<NodeContextMut<'a>> {
        self.nodes.get_mut(node_ref).map(|node| NodeContextMut {
            node_ref,
            node,
            node_changed: false,
            node_index: &mut self.node_index,
        })
    }

    /// Returns a [NodeRef] and reference to its [pipeline::Node] given its ordered index.
    pub fn get_node_by_index(&self, index: usize) -> Option<(NodeRef, &pipeline::Node)> {
        self.node_order
            .get(index)
            .and_then(|&node_ref| self.nodes.get(node_ref).map(|node| (node_ref, node)))
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

/// Provides context for editing a [pipeline::Node] that is part of an [EditablePipeline].
pub struct NodeContextMut<'a> {
    pub node_ref: NodeRef,
    pub node: &'a mut pipeline::Node,
    node_changed: bool,
    node_index: &'a mut data::node_index::NodeIndex,
}

impl<'a> NodeContextMut<'a> {
    /// Should be called if a field in [NodeContextMut::node] has been modified.
    pub fn mark_node_changed(&mut self) {
        self.node_changed = true;
    }

    /// Returns the [data::NodeIndex] for the [EditablePipeline].
    pub fn node_index(&self) -> &data::node_index::NodeIndex {
        self.node_index
    }
}

impl<'a> Drop for NodeContextMut<'a> {
    fn drop(&mut self) {
        if self.node_changed {
            self.node_index.index_node(self.node_ref, self.node);
        }
    }
}
