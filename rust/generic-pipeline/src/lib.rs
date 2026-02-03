pub mod intermediates;
pub mod node;
pub mod pipeline;
pub mod plargs;
pub mod plinputs;
pub mod plparams;
pub mod processing;
pub mod systems;
#[cfg(test)]
mod testutil;

/// Types associated with a [systems::GenericSystem] implementation.
pub trait PipelineTypes {
    /// Type of the [node::GenericNode::spec] field expected by the [systems::GenericSystem]
    /// implementation.
    type Spec: node::SpecTrait;
    /// Parameter type emitted by the [systems::GenericSystem] implementation.
    type ParamType;
    /// Argument type expected by the [systems::GenericSystem] implementation.
    type ArgValue;
    /// Process output emitted by the [systems::GenericSystem] implementation.
    type IntermediateValue;
    /// Process error emitted by the [systems::GenericSystem] implementation.
    type SystemError: std::fmt::Debug;
}
