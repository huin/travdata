use hashbrown::HashMap;
use pipeline::spec_types::pdf;
use serde::{Deserialize, Serialize};
use validator::Validate;

use strum::{IntoDiscriminant, VariantMetadata};
#[cfg(test)]
use testutils::DefaultForTest;

use crate::app::data::{
    editable_pipeline,
    guinode::{self, ToGui as _},
};

#[derive(Clone, Debug, Deserialize, Serialize, strum_macros::EnumDiscriminants)]
#[strum_discriminants(derive(Hash, strum::VariantNames))]
#[serde(tag = "type", content = "spec")]
pub enum Spec {
    InputPdfFile(InputPdfFile),
    JsContext(JsContext),
    JsTransform(JsTransform),
    OutputDirectory(OutputDirectory),
    OutputFileCsv(OutputFileCsv),
    OutputFileJson(OutputFileJson),
    PdfExtractTable(PdfExtractTable),
}

impl Spec {
    pub fn variant_name(&self) -> &'static str {
        self.discriminant().variant_name()
    }
}

impl guinode::NodeComponent for Spec {
    type PipelineType = pipeline::Spec;

    fn to_pipeline(
        self,
        resolve_id: &dyn Fn(
            guinode::NodeIdRef,
        )
            -> Result<pipeline::NodeId, editable_pipeline::ConversionError>,
    ) -> Result<Self::PipelineType, editable_pipeline::ConversionError> {
        Ok(match self {
            Spec::InputPdfFile(spec) => spec.to_pipeline(resolve_id)?.into(),
            Spec::JsContext(spec) => spec.to_pipeline(resolve_id)?.into(),
            Spec::JsTransform(spec) => spec.to_pipeline(resolve_id)?.into(),
            Spec::OutputDirectory(spec) => spec.to_pipeline(resolve_id)?.into(),
            Spec::OutputFileCsv(spec) => spec.to_pipeline(resolve_id)?.into(),
            Spec::OutputFileJson(spec) => spec.to_pipeline(resolve_id)?.into(),
            Spec::PdfExtractTable(spec) => spec.to_pipeline(resolve_id)?.into(),
        })
    }

    fn from_pipeline(
        value: Self::PipelineType,
    ) -> Result<Self, editable_pipeline::ConversionError> {
        Ok(match value {
            pipeline::Spec::InputPdfFile(spec) => Spec::InputPdfFile(spec.to_gui()?),
            pipeline::Spec::JsContext(spec) => Spec::JsContext(spec.to_gui()?),
            pipeline::Spec::JsTransform(spec) => Spec::JsTransform(spec.to_gui()?),
            pipeline::Spec::OutputDirectory(spec) => Spec::OutputDirectory(spec.to_gui()?),
            pipeline::Spec::OutputFileCsv(spec) => Spec::OutputFileCsv(spec.to_gui()?),
            pipeline::Spec::OutputFileJson(spec) => Spec::OutputFileJson(spec.to_gui()?),
            pipeline::Spec::PdfExtractTable(spec) => Spec::PdfExtractTable(spec.to_gui()?),
        })
    }

    fn resolve_node_ids(&mut self, resolver: &dyn Fn(&str) -> Option<guinode::NodeRef>) {
        match self {
            Spec::InputPdfFile(spec) => spec.resolve_node_ids(resolver),
            Spec::JsContext(spec) => spec.resolve_node_ids(resolver),
            Spec::JsTransform(spec) => spec.resolve_node_ids(resolver),
            Spec::OutputDirectory(spec) => spec.resolve_node_ids(resolver),
            Spec::OutputFileCsv(spec) => spec.resolve_node_ids(resolver),
            Spec::OutputFileJson(spec) => spec.resolve_node_ids(resolver),
            Spec::PdfExtractTable(spec) => spec.resolve_node_ids(resolver),
        }
    }
}

#[cfg(test)]
impl DefaultForTest for Spec {
    fn default_for_test() -> Self {
        Self::InputPdfFile(DefaultForTest::default_for_test())
    }
}

impl strum::VariantMetadata for SpecDiscriminants {
    const VARIANT_COUNT: usize = <SpecDiscriminants as strum::VariantNames>::VARIANTS.len();
    const VARIANT_NAMES: &'static [&'static str] =
        <SpecDiscriminants as strum::VariantNames>::VARIANTS;

    fn variant_name(&self) -> &'static str {
        match self {
            SpecDiscriminants::InputPdfFile => "InputPdfFile",
            SpecDiscriminants::JsContext => "JsContext",
            SpecDiscriminants::JsTransform => "JsTransform",
            SpecDiscriminants::OutputDirectory => "OutputDirectory",
            SpecDiscriminants::OutputFileCsv => "OutputFileCsv",
            SpecDiscriminants::OutputFileJson => "OutputFileJson",
            SpecDiscriminants::PdfExtractTable => "PdfExtractTable",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, validator::Validate)]
pub struct InputPdfFile {
    /// Human readable description of the PDF file to show to the user when prompted to choose an
    /// input PDF.
    #[validate(length(max = 200))]
    pub description: String,
}

impl guinode::NodeComponent for InputPdfFile {
    type PipelineType = pipeline::specs::InputPdfFile;

    fn to_pipeline(
        self,
        _resolve_id: &dyn Fn(
            guinode::NodeIdRef,
        )
            -> Result<pipeline::NodeId, editable_pipeline::ConversionError>,
    ) -> Result<Self::PipelineType, editable_pipeline::ConversionError> {
        Ok(pipeline::specs::InputPdfFile {
            description: self.description,
        })
    }

    fn from_pipeline(
        value: Self::PipelineType,
    ) -> Result<Self, editable_pipeline::ConversionError> {
        Ok(Self {
            description: value.description,
        })
    }

    fn resolve_node_ids(&mut self, _resolver: &dyn Fn(&str) -> Option<guinode::NodeRef>) {}
}

#[cfg(test)]
impl DefaultForTest for InputPdfFile {
    fn default_for_test() -> Self {
        Self {
            description: "Test default InputPdfFile description.".into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsContext;

impl guinode::NodeComponent for JsContext {
    type PipelineType = pipeline::specs::JsContext;

    fn to_pipeline(
        self,
        _resolve_id: &dyn Fn(
            guinode::NodeIdRef,
        )
            -> Result<pipeline::NodeId, editable_pipeline::ConversionError>,
    ) -> Result<Self::PipelineType, editable_pipeline::ConversionError> {
        Ok(pipeline::specs::JsContext)
    }

    fn from_pipeline(
        _value: Self::PipelineType,
    ) -> Result<Self, editable_pipeline::ConversionError> {
        Ok(Self)
    }

    fn resolve_node_ids(&mut self, _resolver: &dyn Fn(&str) -> Option<guinode::NodeRef>) {}
}

#[cfg(test)]
impl DefaultForTest for JsContext {
    fn default_for_test() -> Self {
        Self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, validator::Validate)]
pub struct JsTransform {
    #[validate(nested)]
    pub context: guinode::NodeIdRef,
    #[validate(nested)]
    pub input_data: Vec<JsTransformParam>,
    pub code: String,
}

impl guinode::NodeComponent for JsTransform {
    type PipelineType = pipeline::specs::JsTransform;

    fn to_pipeline(
        self,
        resolve_id: &dyn Fn(
            guinode::NodeIdRef,
        )
            -> Result<pipeline::NodeId, editable_pipeline::ConversionError>,
    ) -> Result<Self::PipelineType, editable_pipeline::ConversionError> {
        Ok(
            pipeline::specs::JsTransform {
                context: self.context.to_pipeline(resolve_id)?,
                input_data:
                    self.input_data
                        .into_iter()
                        .map(|trn_param| {
                            Ok((trn_param.name, trn_param.input.to_pipeline(resolve_id)?))
                        })
                        .collect::<Result<
                            HashMap<String, pipeline::NodeId>,
                            editable_pipeline::ConversionError,
                        >>()?,
                code: self.code,
            },
        )
    }

    fn from_pipeline(
        value: Self::PipelineType,
    ) -> Result<Self, editable_pipeline::ConversionError> {
        Ok(Self {
            context: value.context.to_gui()?,
            input_data: value
                .input_data
                .into_iter()
                .map(|(name, node_id)| {
                    Ok(JsTransformParam {
                        name,
                        input: node_id.to_gui()?,
                    })
                })
                .collect::<Result<Vec<JsTransformParam>, editable_pipeline::ConversionError>>()?,
            code: value.code,
        })
    }

    fn resolve_node_ids(&mut self, resolver: &dyn Fn(&str) -> Option<guinode::NodeRef>) {
        self.context.resolve_node_ids(resolver);
        for trn_param in &mut self.input_data {
            trn_param.input.resolve_node_ids(resolver);
        }
    }
}

#[cfg(test)]
impl DefaultForTest for JsTransform {
    fn default_for_test() -> Self {
        Self {
            context: DefaultForTest::default_for_test(),
            input_data: vec![JsTransformParam::default_for_test()],
            code: "return test_param_name;".into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Hash, Eq, PartialEq, Serialize, validator::Validate)]
pub struct JsTransformParam {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(nested)]
    pub input: guinode::NodeIdRef,
}

#[cfg(test)]
impl DefaultForTest for JsTransformParam {
    fn default_for_test() -> Self {
        Self {
            name: "test_param_name".into(),
            input: DefaultForTest::default_for_test(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, validator::Validate)]
pub struct OutputDirectory {
    #[validate(length(max = 200))]
    pub description: String,
}

impl guinode::NodeComponent for OutputDirectory {
    type PipelineType = pipeline::specs::OutputDirectory;

    fn to_pipeline(
        self,
        _resolve_id: &dyn Fn(
            guinode::NodeIdRef,
        )
            -> Result<pipeline::NodeId, editable_pipeline::ConversionError>,
    ) -> Result<Self::PipelineType, editable_pipeline::ConversionError> {
        Ok(pipeline::specs::OutputDirectory {
            description: self.description,
        })
    }

    fn from_pipeline(
        value: Self::PipelineType,
    ) -> Result<Self, editable_pipeline::ConversionError> {
        Ok(Self {
            description: value.description,
        })
    }

    fn resolve_node_ids(&mut self, _resolver: &dyn Fn(&str) -> Option<guinode::NodeRef>) {}
}

#[cfg(test)]
impl DefaultForTest for OutputDirectory {
    fn default_for_test() -> Self {
        Self {
            description: "Test default OutputDirectory description.".into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, validator::Validate)]
pub struct OutputFileCsv {
    #[validate(nested)]
    pub input_data: guinode::NodeIdRef,
    #[validate(nested)]
    pub directory: guinode::NodeIdRef,
    #[validate(nested)]
    pub filename: guinode::spec_types::OutputPath,
}

impl guinode::NodeComponent for OutputFileCsv {
    type PipelineType = pipeline::specs::OutputFileCsv;

    fn to_pipeline(
        self,
        resolve_id: &dyn Fn(
            guinode::NodeIdRef,
        )
            -> Result<pipeline::NodeId, editable_pipeline::ConversionError>,
    ) -> Result<Self::PipelineType, editable_pipeline::ConversionError> {
        Ok(pipeline::specs::OutputFileCsv {
            input_data: self.input_data.to_pipeline(resolve_id)?,
            directory: self.directory.to_pipeline(resolve_id)?,
            filename: self.filename.to_pipeline(resolve_id)?,
        })
    }

    fn from_pipeline(
        value: Self::PipelineType,
    ) -> Result<Self, editable_pipeline::ConversionError> {
        Ok(Self {
            input_data: value.input_data.to_gui()?,
            directory: value.directory.to_gui()?,
            filename: value.filename.to_gui()?,
        })
    }

    fn resolve_node_ids(&mut self, resolver: &dyn Fn(&str) -> Option<guinode::NodeRef>) {
        self.input_data.resolve_node_ids(resolver);
        self.directory.resolve_node_ids(resolver);
        self.filename.resolve_node_ids(resolver);
    }
}

#[cfg(test)]
impl DefaultForTest for OutputFileCsv {
    fn default_for_test() -> Self {
        Self {
            input_data: DefaultForTest::default_for_test(),
            directory: DefaultForTest::default_for_test(),
            filename: "default-for-test.csv".into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, validator::Validate)]
pub struct OutputFileJson {
    #[validate(nested)]
    pub input_data: guinode::NodeIdRef,
    #[validate(nested)]
    pub directory: guinode::NodeIdRef,
    #[validate(nested)]
    pub filename: guinode::spec_types::OutputPath,
}

impl guinode::NodeComponent for OutputFileJson {
    type PipelineType = pipeline::specs::OutputFileJson;

    fn to_pipeline(
        self,
        resolve_id: &dyn Fn(
            guinode::NodeIdRef,
        )
            -> Result<pipeline::NodeId, editable_pipeline::ConversionError>,
    ) -> Result<Self::PipelineType, editable_pipeline::ConversionError> {
        Ok(pipeline::specs::OutputFileJson {
            input_data: self.input_data.to_pipeline(resolve_id)?,
            directory: self.directory.to_pipeline(resolve_id)?,
            filename: self.filename.to_pipeline(resolve_id)?,
        })
    }

    fn from_pipeline(
        value: Self::PipelineType,
    ) -> Result<Self, editable_pipeline::ConversionError> {
        Ok(Self {
            input_data: value.input_data.to_gui()?,
            directory: value.directory.to_gui()?,
            filename: value.filename.to_gui()?,
        })
    }

    fn resolve_node_ids(&mut self, resolver: &dyn Fn(&str) -> Option<guinode::NodeRef>) {
        self.input_data.resolve_node_ids(resolver);
        self.directory.resolve_node_ids(resolver);
        self.filename.resolve_node_ids(resolver);
    }
}

#[cfg(test)]
impl DefaultForTest for OutputFileJson {
    fn default_for_test() -> Self {
        Self {
            input_data: DefaultForTest::default_for_test(),
            directory: DefaultForTest::default_for_test(),
            filename: "default-for-test.json".into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, validator::Validate)]
pub struct PdfExtractTable {
    #[validate(nested)]
    pub pdf: guinode::NodeIdRef,
    #[validate(range(min = 1))]
    pub page: i32,
    pub method: pdf::TabulaExtractionMethod,
    // TODO: Validate for this. Either custom function or implemented in pipeline crate.
    pub rect: pdf::TabulaPdfRect,
}

impl guinode::NodeComponent for PdfExtractTable {
    type PipelineType = pipeline::specs::PdfExtractTable;

    fn to_pipeline(
        self,
        resolve_id: &dyn Fn(
            guinode::NodeIdRef,
        )
            -> Result<pipeline::NodeId, editable_pipeline::ConversionError>,
    ) -> Result<Self::PipelineType, editable_pipeline::ConversionError> {
        Ok(pipeline::specs::PdfExtractTable {
            pdf: self.pdf.to_pipeline(resolve_id)?,
            page: self.page,
            method: self.method,
            rect: self.rect,
        })
    }

    fn from_pipeline(
        value: Self::PipelineType,
    ) -> Result<Self, editable_pipeline::ConversionError> {
        Ok(Self {
            pdf: guinode::NodeIdRef::from_pipeline(value.pdf)?,
            page: value.page,
            method: value.method,
            rect: value.rect,
        })
    }

    fn resolve_node_ids(&mut self, resolver: &dyn Fn(&str) -> Option<guinode::NodeRef>) {
        self.pdf.resolve_node_ids(resolver);
    }
}

#[cfg(test)]
impl DefaultForTest for PdfExtractTable {
    fn default_for_test() -> Self {
        Self {
            pdf: DefaultForTest::default_for_test(),
            page: 1,
            method: pdf::TabulaExtractionMethod::Lattice,
            rect: pdf::TabulaPdfRect::default_for_test(),
        }
    }
}
