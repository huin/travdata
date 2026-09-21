use pipeline::spec_types;

use crate::{
    app::data::{editable_pipeline, guinode},
    error,
};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct OutputPath(pub String);

impl guinode::NodeComponent for OutputPath {
    type PipelineType = spec_types::OutputPathBuf;

    fn to_pipeline(
        self,
        _resolve_id: &dyn Fn(
            guinode::NodeIdRef,
        ) -> Result<
            pipeline::NodeId,
            crate::app::data::editable_pipeline::ConversionError,
        >,
    ) -> Result<Self::PipelineType, crate::app::data::editable_pipeline::ConversionError> {
        spec_types::OutputPathBuf::new(self.0)
            .map_err(editable_pipeline::ConversionError::map_value())
    }

    fn from_pipeline(
        value: Self::PipelineType,
    ) -> Result<Self, editable_pipeline::ConversionError> {
        Ok(Self(
            value
                .into_inner()
                .into_os_string()
                .into_string()
                .map_err(|os_string| {
                    error::StringError(format!(
                        "path {os_string:?} could not be converted to UTF-8 string"
                    ))
                })
                .map_err(editable_pipeline::ConversionError::map_value())?,
        ))
    }

    fn resolve_node_ids(&mut self, _resolver: &dyn Fn(&str) -> Option<guinode::NodeRef>) {}
}

impl From<String> for OutputPath {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for OutputPath {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}
