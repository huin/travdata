use anyhow::{Result, bail};

use crate::{intermediates, specs};

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
    ) -> Result<<crate::PipelineTypes as generic_pipeline::PipelineTypes>::IntermediateValue> {
        if !matches!(&node.spec, specs::Spec::JsContext(_)) {
            bail!("node is not of type JsContext");
        }

        let global_context = v8wrapper::try_with_isolate(|tls_isolate| tls_isolate.new_ctx())?;

        Ok(intermediates::JsContext(global_context).into())
    }
}
