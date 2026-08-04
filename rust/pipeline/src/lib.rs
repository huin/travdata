mod enum_conversion;
pub mod error;
pub mod intermediates;
mod node;
pub mod plargs;
pub mod plinputs;
pub mod plparams;
pub mod spec_types;
pub mod specs;
pub mod systems;
pub mod tabula_wrapper;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod testutil;

use std::rc::Rc;

use crate::tabula_wrapper::TabulaExtractor;
use generic_pipeline::systems::GenericSystem;
use hashbrown::HashMap;
use map_macro::hashbrown::hash_map_e;

pub use error::{StringError, SystemError, SystemErrorKind, SystemResult};
pub use node::{Node, NodeId, NodeMeta};

/// Monomorphic form of [generic_pipeline::systems::NodeResult].
pub type NodeResult = generic_pipeline::systems::NodeResult<PipelineTypes>;

/// Monomorphic form of [generic_pipeline::pipeline::GenericPipeline].
pub type Pipeline = generic_pipeline::pipeline::GenericPipeline<PipelineTypes>;

/// Monomorphic form of [generic_pipeline::plargs::ArgError].
pub type ArgError = generic_pipeline::plargs::ArgError<PipelineTypes>;

/// Monomorphic form of [generic_pipeline::intermediates::IntermediateError].
pub type IntermediateError = generic_pipeline::intermediates::IntermediateError<PipelineTypes>;

/// Monomorphic form of [generic_pipeline::plparams::ParamKey<NodeId>].
pub type ParamKey = generic_pipeline::plparams::ParamKey<NodeId>;

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

/// Create a new [MetaSystem] with the default implementations of all systems.
pub fn new_metasystem(tabula: Box<dyn TabulaExtractor>) -> MetaSystem {
    use crate::specs::SpecDiscriminants::*;

    let systems: HashMap<crate::specs::SpecDiscriminants, Rc<dyn GenericSystem<PipelineTypes>>> = hash_map_e! {
        InputPdfFile => Rc::new(systems::InputPdfFileSystem),
        JsContext => Rc::new(systems::JsContextSystem),
        JsTransform => Rc::new(systems::JsTransformSystem),
        OutputDirectory => Rc::new(systems::OutputDirectorySystem),
        OutputFileCsv => Rc::new(systems::OutputFileCsvSystem),
        OutputFileJson => Rc::new(systems::OutputFileJsonSystem),
        PdfExtractTable => Rc::new(systems::TabulaPdfExtractTableSystem::new(tabula)),
    };

    MetaSystem::new(
        systems,
        Box::new(|discrim| {
            SystemError::map_internal()(StringError(format!(
                "no registered system for spec type {discrim:?}"
            )))
        }),
    )
}
