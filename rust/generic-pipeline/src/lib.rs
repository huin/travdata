pub mod intermediates;
pub mod node;
pub mod pipeline;
pub mod plargs;
pub mod plparams;
pub mod processing;
pub mod systems;
#[cfg(test)]
mod testutil;

pub trait PipelineTypes {
    type Spec: node::SpecTrait;
    type ParamType;
    type ArgValue;
    type IntermediateValue;
}
