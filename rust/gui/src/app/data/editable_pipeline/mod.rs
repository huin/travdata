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
            node_index::{self, NodeIndex, NodeIndexEntry, NodeIndexGeneration},
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

type NodeSet = slotmap::SlotMap<NodeRef, GuiNodeWithId>;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct EditablePipeline {
    nodes: NodeSet,
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
                    .remove(node_ref)
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

    pub fn with_node_ctx_by_ref_mut<'a, F, T>(&'a mut self, node_ref: NodeRef, f: F) -> T
    where
        F: FnOnce(Option<NodeContextMut<'a>>) -> T,
    {
        let node_ctx = self.nodes.get_mut(node_ref).map(|node| NodeContextMut {
            node_ref,
            node,
            node_changed: false,
            node_index: &mut self.node_index,
        });
        f(node_ctx)
    }

    /// Returns a [NodeRef] and reference to its [GuiNodeWithId] given its ordered index.
    pub fn get_node_by_index(&self, index: usize) -> Option<(NodeRef, &GuiNodeWithId)> {
        self.node_order
            .get(index)
            .and_then(|&node_ref| self.nodes.get(node_ref).map(|node| (node_ref, node)))
    }

    /// Adds a new node, returning its allocated [NodeRef].
    pub fn add_node(&mut self, mut node: GuiNodeWithId) -> NodeRef {
        self.nodes.insert_with_key(|node_ref| {
            node.node.meta.id = GuiNodeId::Resolved(node_ref);
            self.node_order.push(node_ref);
            self.node_index.index_node(node_ref, &node);
            node
        })
    }

    /// Removes a node.
    pub fn remove_node(&mut self, node_ref: NodeRef) {
        self.nodes.remove(node_ref);
        self.node_order.retain(|&item| item != node_ref);
        self.node_index.deindex_node(node_ref);
    }

    pub fn node_index(&self) -> &NodeIndex {
        &self.node_index
    }

    /// Resolves all [GuiNodeId::Unresolved]s to [NodeRef] using the index, leaving unchanged where
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
        let mut editable_pipeline = Self {
            nodes: NodeSet::with_capacity_and_key(pipeline.len()),
            node_order: Vec::with_capacity(pipeline.len()),
            node_index: node_index::NodeIndex::with_capacity(pipeline.len()),
        };

        let node_id_trn = NodeIdToGuiNodeId;
        for node in pipeline.into_iter() {
            let node_id = node.meta.id.0.clone();
            let Ok(gui_node) = data::GuiNode::transform_node_ids(node, &node_id_trn);
            editable_pipeline.add_node(GuiNodeWithId {
                node_id,
                node: gui_node,
            });
        }

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

struct GuiNodeIdToNodeId<'a>(&'a NodeSet);
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

struct GuiNodeIdResolver<'a>(&'a data::node_index::NodeIndex);
impl<'a> generic::node::NodeIdTransformer for GuiNodeIdResolver<'a> {
    type FromNodeId = GuiNodeId;
    type ToNodeId = GuiNodeId;
    type Error = NeverError;

    fn transform_node_id(&self, node_id: GuiNodeId) -> Result<GuiNodeId, NeverError> {
        match &node_id {
            GuiNodeId::Unresolved(node_id) => {
                let mut results = self.0.lookup_exact_node_id(node_id);
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
