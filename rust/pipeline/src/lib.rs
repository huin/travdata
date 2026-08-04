mod enum_conversion;
pub mod error;
pub mod intermediates;
pub mod monomorph;
mod node;
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

use crate::tabula_wrapper::TabulaExtractor;
use generic_pipeline::systems::GenericSystem;
use hashbrown::HashMap;
use map_macro::hashbrown::hash_map_e;

pub use error::{StringError, SystemError, SystemErrorKind, SystemResult};
pub use monomorph::PipelineTypes;
pub use node::{Node, NodeId, NodeMeta};

/// Create a new [MetaSystem] with the default implementations of all systems.
pub fn new_metasystem(tabula: Box<dyn TabulaExtractor>) -> monomorph::MetaSystem {
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

    monomorph::MetaSystem::new(
        systems,
        Box::new(|discrim| {
            SystemError::map_internal()(StringError(format!(
                "no registered system for spec type {discrim:?}"
            )))
        }),
    )
}
