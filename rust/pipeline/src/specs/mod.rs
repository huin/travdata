//! Concrete specialisations of [generic_pipeline::Node]s.

#[cfg(test)]
mod tests;

use crate::{NodeId, StringError, SystemError, SystemResult, generic, impl_enum_conversions};

pub type Spec = generic::specs::Spec<NodeId>;

pub type SpecDiscriminants = generic::specs::SpecDiscriminants;

pub type InputPdfFile = generic::specs::InputPdfFile;
pub type JsContext = generic::specs::JsContext;
pub type JsTransform = generic::specs::JsTransform<NodeId>;
pub type OutputDirectory = generic::specs::OutputDirectory;
pub type OutputFileCsv = generic::specs::OutputFileCsv<NodeId>;
pub type OutputFileJson = generic::specs::OutputFileJson<NodeId>;
pub type PdfExtractTable = generic::specs::PdfExtractTable<NodeId>;

impl generic::specs::Spec<NodeId> {
    pub fn downcast<'s, S>(&'s self) -> SystemResult<&'s S>
    where
        &'s S: TryFrom<&'s Spec, Error = StringError>,
    {
        self.try_into().map_err(SystemError::map_spec())
    }
}

impl generic_pipeline::systems::TypedNode for generic::specs::Spec<NodeId> {
    type NodeType = generic::specs::SpecDiscriminants;

    fn node_type(&self) -> Self::NodeType {
        self.into()
    }
}

impl_enum_conversions!(Spec, InputPdfFile, "node");
impl_enum_conversions!(Spec, JsContext, "node");
impl_enum_conversions!(Spec, JsTransform, "node");
impl_enum_conversions!(Spec, OutputDirectory, "node");
impl_enum_conversions!(Spec, OutputFileCsv, "node");
impl_enum_conversions!(Spec, OutputFileJson, "node");
impl_enum_conversions!(Spec, PdfExtractTable, "node");
