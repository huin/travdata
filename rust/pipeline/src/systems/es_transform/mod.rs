#[cfg(test)]
mod tests;

/// Provides processing support for [EsTransform].
pub struct EsTransformSystem {
    v8_ctx: v8wrapper::ContextClient,
}

impl EsTransformSystem {
    pub fn new(v8_ctx: v8wrapper::ContextClient) -> Self {
        Self { v8_ctx }
    }
}

impl generic_pipeline::systems::GenericSystem<crate::PipelineTypes> for EsTransformSystem {
    fn params(&self, _node: &crate::Node) -> crate::plparams::Params {
        crate::plparams::Params::default()
    }

    fn inputs(&self, _node: &crate::Node) -> Vec<crate::NodeId> {
        todo!()
    }

    fn process(
        &self,
        _node: &crate::Node,
        _args: &crate::plargs::ArgSet,
        _intermediates: &crate::intermediates::IntermediateSet,
    ) -> anyhow::Result<crate::intermediates::IntermediateValue> {
        // TODO: Use `Object.freeze` to freeze any data passed in. This means that any future
        // batching in `process_multiple` that would require it is not a breaking change.
        todo!()
    }

    // TODO: Optionally implement `process_multiple` as there might be some possible batching
    // optimisations there.
}
