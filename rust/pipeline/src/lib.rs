mod enum_conversion;
pub mod error;
pub mod intermediates;
pub mod plargs;
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

pub use error::{StringError, SystemError, SystemErrorKind, SystemResult};
use generic_pipeline::systems::GenericSystem;
use hashbrown::HashMap;
use map_macro::hashbrown::hash_map_e;

use crate::tabula_wrapper::TabulaExtractor;

pub type NodeId = generic_pipeline::node::NodeId;

/// Monomorphic form of [generic_pipeline::node::GenericNode] used with real implementations.
pub type Node = generic_pipeline::node::GenericNode<specs::Spec>;

/// Monomorphic form of [generic_pipeline::systems::NodeResult] used with real implementations.
pub type NodeResult = generic_pipeline::systems::NodeResult<PipelineTypes>;

/// Specific [generic_pipeline::pipeline::GenericPipeline] used in actual processing.
pub type Pipeline = generic_pipeline::pipeline::GenericPipeline<specs::Spec>;

/// Specifies all the types required for a [generic_pipeline::processing::GenericProcessor] and
/// [generic_pipeline::systems::GenericSystem].
pub struct PipelineTypes;

impl generic_pipeline::PipelineTypes for PipelineTypes {
    type Spec = specs::Spec;

    type ParamType = plparams::ParamType;

    type ArgValue = plargs::ArgValue;

    type IntermediateValue = intermediates::IntermediateValue;

    type SystemError = error::SystemError;
}

/// Monomorphic form of [generic_pipeline::systems::GenericMetaSystem] used with realm
/// implementations.
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
