use thiserror_context::Context;

use crate::{SystemError, SystemResult, intermediates, plargs, specs};

pub struct JsContextSystem;

impl generic_pipeline::systems::GenericSystem<crate::PipelineTypes> for JsContextSystem {
    fn process(
        &self,
        node: &crate::Node,
        _args: &plargs::ArgSet,
        _intermediates: &intermediates::IntermediateSet,
    ) -> SystemResult<intermediates::IntermediateValue> {
        let _: &specs::JsContext = node.spec.downcast()?;

        let global_context = v8wrapper::try_with_isolate(|tls_isolate| tls_isolate.new_ctx())
            .map_err(SystemError::map_execution())
            .context("accessing JS context")?;

        Ok(intermediates::JsContext(global_context).into())
    }
}
