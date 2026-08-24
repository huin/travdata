#[cfg(test)]
mod tests;

use std::sync::Arc;

use pipeline::generic::{self, node::TranslateFromNodeId};
use thiserror::Error;

use crate::{
    app::{
        data::{
            self, GuiNode, GuiNodeId, GuiSpec, NodeRef,
            node::{GuiNodeMeta, GuiNodeWithId},
            node_index,
        },
        ddo,
    },
    error::{ArcStdError, StdError, StringError},
};

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("internal error:: {0}")]
    Internal(#[source] ArcStdError),
}

impl ConversionError {
    pub(crate) fn map_internal<E>() -> impl FnOnce(E) -> Self
    where
        E: StdError + 'static,
    {
        |err| ConversionError::Internal(Arc::new(err))
    }
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct EditablePipeline {
    // TODO: Consider using a 3rd party generational arena instead of NodeSet and NodeRef.
    nodes: data::NodeSet,
    node_order: Vec<NodeRef>,
    node_index: data::node_index::NodeIndex,
}

impl EditablePipeline {
    pub fn to_pipeline(&self) -> Result<ddo::PipelineNodes, ConversionError> {
        let trn = GuiNodeIdToNodeId(&self.nodes);
        let mut nodes = self.nodes.clone();
        self.node_order
            .iter()
            .map(|&node_ref| {
                nodes
                    .take(node_ref)
                    .ok_or_else(|| {
                        StringError(format!("bug: could not resolve NodeRef {node_ref:?}"))
                    })
                    .map_err(ConversionError::map_internal())
                    .and_then(|gui_node_with_id| {
                        let mut node =
                            pipeline::Node::transform_node_ids(gui_node_with_id.node, &trn)?;
                        node.meta.id = pipeline::NodeId(gui_node_with_id.node_id);
                        Ok(node)
                    })
            })
            .collect::<Result<ddo::PipelineNodes, ConversionError>>()
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

    /// Returns a [NodeRef] and reference to its [GuiNodeWithId] given its ordered index.
    pub fn get_node_by_index(&self, index: usize) -> Option<(NodeRef, &GuiNodeWithId)> {
        self.node_order
            .get(index)
            .and_then(|&node_ref| self.nodes.get(node_ref).map(|node| (node_ref, node)))
    }

    /// Resolves all [GuiNodeId]s to [NodeRef] using the index, leaving unresolved where
    /// missing/ambigious.
    fn resolve_node_ids(&mut self) {
        let mut tmp_node = GuiNode {
            meta: GuiNodeMeta {
                id: GuiNodeId::Unresolved("".into()),
            },
            spec: GuiSpec::InputPdfFile(pipeline::generic::specs::InputPdfFile {
                description: "".into(),
            }),
        };
        let trn = GuiNodeIdResolver(&self.node_index);
        for gui_node_with_id in self.nodes.values_mut() {
            std::mem::swap(&mut tmp_node, &mut gui_node_with_id.node);
            let Ok(tmp_node_updated) = GuiNode::transform_node_ids(tmp_node, &trn);
            tmp_node = tmp_node_updated;
            std::mem::swap(&mut tmp_node, &mut gui_node_with_id.node);
        }
    }
}

impl TryFrom<ddo::PipelineNodes> for EditablePipeline {
    type Error = ConversionError;

    fn try_from(pipeline: ddo::PipelineNodes) -> Result<Self, Self::Error> {
        let mut nodes = data::NodeSet::with_capacity(pipeline.len());
        let mut node_order: Vec<NodeRef> = Vec::with_capacity(pipeline.len());
        let mut node_index = node_index::NodeIndex::with_capacity(pipeline.len());

        {
            let node_id_trn = NodeIdToGuiNodeId;
            for node in pipeline.into_iter() {
                let node_id = node.meta.id.0.clone();
                let Ok(gui_node) = data::GuiNode::transform_node_ids(node, &node_id_trn);
                let gui_node_with_id = GuiNodeWithId {
                    node_id,
                    node: gui_node,
                };
                let node_ref = nodes
                    .add_node(gui_node_with_id)
                    .map_err(StringError)
                    .map_err(ConversionError::map_internal())?;
                let node = match nodes.get(node_ref) {
                    Some(node) => node,
                    None => {
                        return Err(ConversionError::map_internal()(StringError(
                            "bug: internal error, just inserted Node is absent from NodeSet"
                                .to_string(),
                        )));
                    }
                };
                node_order.push(node_ref);
                node_index.index_node(node_ref, node);
            }
        }

        let mut editable_pipeline = Self {
            nodes,
            node_order,
            node_index,
        };

        // Resolve all GuiNodeIds in `nodes` to `NodeRef`s using `node_index`, leaving unresolved
        // where missing/ambigious.
        editable_pipeline.resolve_node_ids();

        Ok(editable_pipeline)
    }
}

/// Provides context for editing a [GuiNodeWithId] that is part of an [EditablePipeline].
pub struct NodeContextMut<'a> {
    pub node_ref: NodeRef,
    pub node: &'a mut GuiNodeWithId,
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

enum NeverError {}

struct NodeIdToGuiNodeId;
impl pipeline::generic::node::NodeIdTransformer for NodeIdToGuiNodeId {
    type FromNodeId = pipeline::NodeId;
    type ToNodeId = GuiNodeId;
    type Error = NeverError;

    fn transform_node_id(&self, node_id: pipeline::NodeId) -> Result<GuiNodeId, NeverError> {
        Ok(data::GuiNodeId::Unresolved(node_id.0))
    }
}

struct GuiNodeIdToNodeId<'a>(&'a data::NodeSet);
impl<'a> generic::node::NodeIdTransformer for GuiNodeIdToNodeId<'a> {
    type FromNodeId = GuiNodeId;
    type ToNodeId = pipeline::NodeId;
    type Error = ConversionError;

    fn transform_node_id(&self, node_id: GuiNodeId) -> Result<pipeline::NodeId, ConversionError> {
        Ok(pipeline::NodeId(match node_id {
            GuiNodeId::Unresolved(node_id) => node_id,
            GuiNodeId::Resolved(node_ref) => self
                .0
                .get(node_ref)
                .ok_or_else(|| {
                    StringError(format!(
                        "bug: GuiNodeId::Resolved({node_ref:?}) referred to unknown node"
                    ))
                })
                .map_err(ConversionError::map_internal())?
                .node_id
                .clone(),
        }))
    }
}

struct GuiNodeIdResolver<'a>(&'a data::NodeIndex);
impl<'a> generic::node::NodeIdTransformer for GuiNodeIdResolver<'a> {
    type FromNodeId = GuiNodeId;
    type ToNodeId = GuiNodeId;
    type Error = NeverError;

    fn transform_node_id(&self, node_id: GuiNodeId) -> Result<GuiNodeId, NeverError> {
        match &node_id {
            GuiNodeId::Unresolved(node_id) => {
                let mut results = self.0.by_node_id(node_id);
                match (results.next(), results.next()) {
                    (None, _) => {
                        // No matching node ID.
                    }
                    (Some(node_entry), None) => {
                        return Ok(GuiNodeId::Resolved(*node_entry.node_ref()));
                    }
                    (Some(_), Some(_)) => {
                        // Ambigious node ID.
                    }
                }
            }
            GuiNodeId::Resolved(_) => {
                // Already resolved.
            }
        }
        Ok(node_id)
    }
}
