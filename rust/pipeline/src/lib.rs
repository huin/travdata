pub mod intermediates;
pub mod plargs;
pub mod plparams;
pub mod spec_types;
pub mod specs;
pub mod systems;
#[cfg(test)]
mod testutil;

pub type NodeId = generic_pipeline::node::NodeId;

/// Monomorphic form of [generic_pipeline::node::GenericNode] used with real implementations.
pub type Node = generic_pipeline::node::GenericNode<specs::Spec>;

/// Specific [generic_pipeline::pipeline::GenericPipeline] used in actual processing.
pub type Pipeline = generic_pipeline::pipeline::GenericPipeline<specs::Spec>;

/// Specifies all the types required for a [generic_pipeline::processing::GenericProcessor] and
/// [generic_pipeline::systems::GenericSystem].
pub struct PipelineTypes;

impl generic_pipeline::PipelineTypes for PipelineTypes {
    type Spec = specs::Spec;

    type ParamType = plparams::ParamType;

    type ArgValue = plargs::Arg;

    type IntermediateValue = intermediates::Intermediate;
}

/// Monomorphic form of [generic_pipeline::systems::GenericMetaSystem] used with realm
/// implementations.
pub type MetaSystem = generic_pipeline::systems::GenericMetaSystem<PipelineTypes>;
