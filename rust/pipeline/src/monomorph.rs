use crate::{
    Node, NodeId, error, intermediates, plargs,
    plparams::{self, ParamType},
};

/// Monomorphic form of [generic_pipeline::plparams::GenericParam].
pub type Param = generic_pipeline::plparams::GenericParam<ParamType>;

/// Monomorphic form of [generic_pipeline::plparams::GenericParams].
pub type Params = generic_pipeline::plparams::GenericParams<crate::PipelineTypes>;

/// Monomorphic form of [generic_pipeline::plparams::GenericParamsRegistrator].
pub type ParamsRegistrator =
    generic_pipeline::plparams::GenericParamsRegistrator<crate::PipelineTypes>;

/// Monomorphic form of [generic_pipeline::plparams::GenericNodeParamsRegistrator].
pub type NodeParamsRegistrator<'a> =
    generic_pipeline::plparams::GenericNodeParamsRegistrator<'a, crate::PipelineTypes>;

/// Monomorphic form of [generic_pipeline::systems::NodeResult].
pub type NodeResult = generic_pipeline::systems::NodeResult<PipelineTypes>;

/// Monomorphic form of [generic_pipeline::pipeline::GenericPipeline].
pub type Pipeline = generic_pipeline::pipeline::GenericPipeline<PipelineTypes>;

/// Monomorphic form of [generic_pipeline::plargs::GenericArgSet].
pub type ArgSet = generic_pipeline::plargs::GenericArgSet<crate::PipelineTypes>;

/// Monomorphic form of [generic_pipeline::plargs::ArgError].
pub type ArgError = generic_pipeline::plargs::ArgError<PipelineTypes>;

/// Monomorphic form of [generic_pipeline::intermediates::IntermediateError].
pub type IntermediateError = generic_pipeline::intermediates::IntermediateError<PipelineTypes>;

/// Monomorphic form of [generic_pipeline::plparams::ParamKey<NodeId>].
pub type ParamKey = generic_pipeline::plparams::ParamKey<NodeId>;

pub type InputsRegistrator = generic_pipeline::plinputs::InputsRegistrator<crate::PipelineTypes>;
pub type NodeInputsRegistrator<'a> =
    generic_pipeline::plinputs::NodeInputsRegistrator<'a, crate::PipelineTypes>;

/// Specifies all the types required for a [generic_pipeline::processing::GenericProcessor] and
/// [generic_pipeline::systems::GenericSystem].
pub struct PipelineTypes;

impl generic_pipeline::PipelineTypes for PipelineTypes {
    type NodeId = NodeId;

    type Node = Node;

    type ParamType = plparams::ParamType;

    type ArgValue = plargs::ArgValue;

    type IntermediateValue = intermediates::IntermediateValue;

    type SystemError = error::SystemError;
}

/// Monomorphic form of [generic_pipeline::systems::GenericMetaSystem].
pub type MetaSystem = generic_pipeline::systems::GenericMetaSystem<PipelineTypes>;
