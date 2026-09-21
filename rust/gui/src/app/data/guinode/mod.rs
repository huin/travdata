//! [pipeline] node and spec types specific to the GUI.

pub mod spec_types;
mod specs;

use serde::{Deserialize, Serialize};
#[cfg(test)]
use testutils::DefaultForTest;

use crate::app::data::editable_pipeline;

pub use specs::*;

/// Trait for all substantive types within a [Node], including the [Node] itself. Handles converting
/// to/from [pipeline] types and resolving [NodeIdRef::Unresolved] when possible.
pub trait NodeComponent: Sized {
    type PipelineType;

    fn to_pipeline(
        self,
        resolve_id: &dyn Fn(
            NodeIdRef,
        )
            -> Result<pipeline::NodeId, editable_pipeline::ConversionError>,
    ) -> Result<Self::PipelineType, editable_pipeline::ConversionError>;

    fn from_pipeline(value: Self::PipelineType)
    -> Result<Self, editable_pipeline::ConversionError>;

    fn resolve_node_ids(&mut self, resolver: &dyn Fn(&str) -> Option<NodeRef>);
}

slotmap::new_key_type! {
    /// Remains the same for the lifetime of an [editable_pipeline::EditablePipeline]. Isn't
    /// displayed, but used in maintaining consistent references to other nodes regardless of their
    /// [Node::id] changing.
    pub struct NodeRef;
}

trait ToGui<T> {
    type GuiType;

    fn to_gui(self) -> Result<Self::GuiType, editable_pipeline::ConversionError>;
}

impl<F, T> ToGui<T> for F
where
    T: NodeComponent<PipelineType = F>,
{
    type GuiType = T;

    fn to_gui(self) -> Result<Self::GuiType, editable_pipeline::ConversionError> {
        Self::GuiType::from_pipeline(self)
    }
}

impl hashbrown::Equivalent<NodeRef> for &NodeRef {
    fn equivalent(&self, key: &NodeRef) -> bool {
        *self == key
    }
}

/// Reference to a [NodeMeta::id] that may be resolved to an actual [Node] (via its [NodeRef]), or
/// not (then the [String] of the unresolved and maybe non-existing [NodeMeta::id]).
#[derive(Clone, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub enum NodeIdRef {
    /// The node ID has not (yet) been resolved to a specific [NodeRef].
    ///
    /// This may happen temporarily during import of a [pipeline::Node], or for longer if the
    /// [pipeline::NodeId] does not exist or is ambigious (multiple nodes with same ID).
    Unresolved(String),
    /// The [pipeline::NodeId] has been resolved to a specific [NodeRef].
    Resolved(NodeRef),
}

impl NodeIdRef {
    pub fn is_resolved(&self) -> bool {
        matches!(self, Self::Resolved(_))
    }
}

impl NodeComponent for NodeIdRef {
    type PipelineType = pipeline::NodeId;

    fn to_pipeline(
        self,
        resolve_id: &dyn Fn(
            NodeIdRef,
        )
            -> Result<pipeline::NodeId, editable_pipeline::ConversionError>,
    ) -> Result<Self::PipelineType, editable_pipeline::ConversionError> {
        resolve_id(self)
    }

    fn from_pipeline(
        value: Self::PipelineType,
    ) -> Result<Self, editable_pipeline::ConversionError> {
        Ok(Self::Unresolved(value.0))
    }

    fn resolve_node_ids(&mut self, resolver: &dyn Fn(&str) -> Option<NodeRef>) {
        let node_ref = match &*self {
            Self::Resolved(node_ref) => Some(*node_ref),
            Self::Unresolved(node_id_string) => resolver(node_id_string),
        };

        if let Some(node_ref) = node_ref {
            *self = Self::Resolved(node_ref);
        }
    }
}

#[cfg(test)]
impl testutils::DefaultForTest for NodeIdRef {
    fn default_for_test() -> Self {
        use slotmap::Key as _;

        NodeIdRef::Resolved(NodeRef::null())
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Node {
    #[serde(flatten)]
    pub meta: NodeMeta,
    #[serde(flatten)]
    pub spec: specs::Spec,
}

impl NodeComponent for Node {
    type PipelineType = pipeline::Node;

    fn to_pipeline(
        self,
        resolve_id: &dyn Fn(
            NodeIdRef,
        )
            -> Result<pipeline::NodeId, editable_pipeline::ConversionError>,
    ) -> Result<Self::PipelineType, editable_pipeline::ConversionError> {
        Ok(pipeline::Node {
            meta: self.meta.to_pipeline(resolve_id)?,
            spec: self.spec.to_pipeline(resolve_id)?,
        })
    }

    fn from_pipeline(
        value: Self::PipelineType,
    ) -> Result<Self, editable_pipeline::ConversionError> {
        Ok(Self {
            meta: NodeMeta::from_pipeline(value.meta)?,
            spec: Spec::from_pipeline(value.spec)?,
        })
    }

    fn resolve_node_ids(&mut self, resolver: &dyn Fn(&str) -> Option<NodeRef>) {
        self.meta.resolve_node_ids(resolver);
        self.spec.resolve_node_ids(resolver);
    }
}

#[cfg(test)]
impl DefaultForTest for Node {
    fn default_for_test() -> Self {
        Self {
            meta: DefaultForTest::default_for_test(),
            spec: DefaultForTest::default_for_test(),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct NodeMeta {
    /// NodeMeta has a [String] instead of [NodeIdRef] because it is authoritative about what the ID
    /// text is.
    pub id: String,
}

impl NodeComponent for NodeMeta {
    type PipelineType = pipeline::NodeMeta;

    fn to_pipeline(
        self,
        _resolve_id: &dyn Fn(
            NodeIdRef,
        )
            -> Result<pipeline::NodeId, editable_pipeline::ConversionError>,
    ) -> Result<Self::PipelineType, editable_pipeline::ConversionError> {
        Ok(pipeline::NodeMeta { id: self.id.into() })
    }

    fn from_pipeline(
        value: Self::PipelineType,
    ) -> Result<Self, editable_pipeline::ConversionError> {
        Ok(Self { id: value.id.0 })
    }

    fn resolve_node_ids(&mut self, _resolver: &dyn Fn(&str) -> Option<NodeRef>) {}
}

#[cfg(test)]
impl DefaultForTest for NodeMeta {
    fn default_for_test() -> Self {
        Self {
            id: "default-gui-node-id".into(),
        }
    }
}
