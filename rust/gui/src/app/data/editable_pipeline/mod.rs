#[cfg(test)]
mod tests;

use std::sync::Arc;

use thiserror::Error;

use crate::{
    app::{
        data::{
            self,
            guinode::{self, NodeComponent as _},
            node_index,
        },
        ddo,
    },
    error::{ArcStdError, StdError, StringError},
};

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("internal error: {0}")]
    Internal(#[source] ArcStdError),
    #[error("value error: {0}")]
    Value(#[source] ArcStdError),
}

impl ConversionError {
    pub(crate) fn map_internal<E>() -> impl FnOnce(E) -> Self
    where
        E: StdError + 'static,
    {
        |err| ConversionError::Internal(Arc::new(err))
    }

    pub(crate) fn map_value<E>() -> impl FnOnce(E) -> Self
    where
        E: StdError + 'static,
    {
        |err| ConversionError::Value(Arc::new(err))
    }
}

type GuiNodeSet = slotmap::SlotMap<guinode::NodeRef, guinode::Node>;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct EditablePipeline {
    nodes: GuiNodeSet,
    node_order: Vec<guinode::NodeRef>,
    node_index: data::node_index::NodeIndex,
}

impl EditablePipeline {
    pub fn to_pipeline(&self) -> Result<ddo::PipelineNodes, ConversionError> {
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
                    .and_then(|gui_node| {
                        gui_node.to_pipeline(&|gui_node_id| {
                            to_pipeline_node_id(&self.nodes, gui_node_id)
                        })
                    })
            })
            .collect::<Result<ddo::PipelineNodes, ConversionError>>()
    }

    /// Returns the number of nodes in the pipeline.
    pub fn len(&self) -> usize {
        self.node_order.len()
    }

    pub fn with_node_ctx_by_ref_mut<'a, F, T>(&'a mut self, node_ref: guinode::NodeRef, f: F) -> T
    where
        F: FnOnce(Option<NodeContextMut<'a>>) -> T,
    {
        let node_ctx = self.nodes.get_mut(node_ref).map(|node| NodeContextMut {
            node,
            node_changed: false,
            node_index: &mut self.node_index,
        });
        f(node_ctx)
    }

    /// Returns a [guinode::NodeRef] and reference to its [guinode::Node] given its ordered index.
    pub fn get_node_by_index(&self, index: usize) -> Option<(guinode::NodeRef, &guinode::Node)> {
        self.node_order
            .get(index)
            .and_then(|&node_ref| self.nodes.get(node_ref).map(|node| (node_ref, node)))
    }

    /// Adds a new node, returning its allocated [guinode::NodeRef]. Also sets the
    /// [guinode::NodeRef] on `node.meta.self_ref`.
    pub fn add_node<F>(&mut self, f: F) -> guinode::NodeRef
    where
        F: FnOnce(guinode::NodeRef) -> guinode::Node,
    {
        self.nodes.insert_with_key(|node_ref| {
            let node = f(node_ref);
            self.node_order.push(node_ref);
            self.node_index.index_node(&node);
            node
        })
    }

    /// Removes a node.
    pub fn remove_node(&mut self, node_ref: guinode::NodeRef) {
        self.nodes.remove(node_ref);
        self.node_order.retain(|&item| item != node_ref);
        self.node_index.deindex_node(node_ref);
    }

    #[cfg(test)]
    pub fn node_index(&self) -> &data::NodeIndex {
        &self.node_index
    }

    /// Resolves all [guinode::NodeIdRef::Unresolved]s to [guinode::NodeRef] using the index,
    /// leaving unchanged where missing/ambigious.
    fn resolve_node_ids(&mut self) {
        for gui_node in self.nodes.values_mut() {
            gui_node.resolve_node_ids(&|node_id_str| {
                resolve_node_id_exact(&self.node_index, node_id_str)
            });
        }
    }
}

impl TryFrom<ddo::PipelineNodes> for EditablePipeline {
    type Error = ConversionError;

    fn try_from(pipeline: ddo::PipelineNodes) -> Result<Self, Self::Error> {
        let mut editable_pipeline = Self {
            nodes: GuiNodeSet::with_capacity_and_key(pipeline.len()),
            node_order: Vec::with_capacity(pipeline.len()),
            node_index: node_index::NodeIndex::with_capacity(pipeline.len()),
        };

        for node in pipeline.into_iter() {
            let mut gui_node = guinode::Node::from_pipeline(node)?;
            editable_pipeline.add_node(|node_ref| {
                gui_node.meta.self_ref = node_ref;
                gui_node
            });
        }

        editable_pipeline.resolve_node_ids();

        Ok(editable_pipeline)
    }
}

/// Provides context for editing a [guinode::Node] that is part of an [EditablePipeline].
pub struct NodeContextMut<'a> {
    pub node: &'a mut guinode::Node,
    node_changed: bool,
    pub node_index: &'a mut data::node_index::NodeIndex,
}

impl<'a> NodeContextMut<'a> {
    /// Should be called if a field in [NodeContextMut::node] has been modified.
    pub fn mark_node_changed(&mut self) {
        self.node_changed = true;
    }
}

impl<'a> Drop for NodeContextMut<'a> {
    fn drop(&mut self) {
        if self.node_changed {
            self.node_index.index_node(self.node);
        }
    }
}

fn resolve_node_id_exact(
    node_index: &data::node_index::NodeIndex,
    node_id_str: &str,
) -> Option<guinode::NodeRef> {
    let mut results = node_index.lookup_exact_node_id(node_id_str);
    match (results.next(), results.next()) {
        (None, _) => {
            // No matching node ID.
            None
        }
        (Some(node_entry), None) => {
            // Single exact match.
            Some(*node_entry.node_ref())
        }
        (Some(_), Some(_)) => {
            // Ambigious node ID.
            None
        }
    }
}

fn to_pipeline_node_id(
    nodes: &GuiNodeSet,
    gui_node_id: guinode::NodeIdRef,
) -> Result<pipeline::NodeId, ConversionError> {
    Ok(pipeline::NodeId(match gui_node_id {
        guinode::NodeIdRef::Unresolved(node_id) => node_id,
        guinode::NodeIdRef::Resolved(node_ref) => nodes
            .get(node_ref)
            .ok_or_else(|| {
                StringError(format!(
                    "bug: guinode::NodeIdRef::Resolved({node_ref:?}) referred to unknown node"
                ))
            })
            .map_err(ConversionError::map_internal())?
            .meta
            .id
            .clone(),
    }))
}
