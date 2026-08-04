pub mod intermediates;
mod node;
pub mod pipeline;
pub mod plargs;
pub mod plinputs;
pub mod plparams;
pub mod processing;
pub mod systems;
#[cfg(test)]
mod testutil;

pub use node::PipelineNode;
pub use node::PipelineNodeId;

/// Types associated with a [systems::GenericSystem] implementation.
pub trait PipelineTypes {
    /// [node::PipelineNodeId] implementation expected by the [systems::GenericSystem]
    /// implementation.
    type NodeId: PipelineNodeId;
    /// [node::PipelineNode] implementation expected by the [systems::GenericSystem] implementation.
    type Node: PipelineNode<Id = Self::NodeId>;
    /// Parameter type emitted by the [systems::GenericSystem] implementation.
    type ParamType;
    /// Argument type expected by the [systems::GenericSystem] implementation.
    type ArgValue;
    /// Process output emitted by the [systems::GenericSystem] implementation.
    type IntermediateValue;
    /// Process error emitted by the [systems::GenericSystem] implementation.
    type SystemError: std::fmt::Debug;
}
