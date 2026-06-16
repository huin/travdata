use thiserror_context::Context;

use crate::{SystemError, SystemResult, intermediates, specs};

pub struct JsContextSystem;

impl generic_pipeline::systems::GenericSystem<crate::PipelineTypes> for JsContextSystem {
    fn process(
        &self,
        node: &generic_pipeline::node::GenericNode<
            <crate::PipelineTypes as generic_pipeline::PipelineTypes>::Spec,
        >,
        _args: &generic_pipeline::plargs::GenericArgSet<
            <crate::PipelineTypes as generic_pipeline::PipelineTypes>::ArgValue,
        >,
        _intermediates: &generic_pipeline::intermediates::GenericIntermediateSet<
            <crate::PipelineTypes as generic_pipeline::PipelineTypes>::IntermediateValue,
        >,
    ) -> SystemResult<<crate::PipelineTypes as generic_pipeline::PipelineTypes>::IntermediateValue>
    {
        let _: &specs::JsContext = node.spec.downcast_spec()?;

        let global_context = v8wrapper::try_with_isolate(|tls_isolate| tls_isolate.new_ctx())
            .map_err(SystemError::map_execution())
            .context("accessing JS context")?;

        Ok(intermediates::JsContext(global_context).into())
    }
}
