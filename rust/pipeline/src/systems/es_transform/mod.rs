#[cfg(test)]
mod tests;

use generic_pipeline::plinputs;

use crate::{plparams, specs};

/// Provides processing support for [EsTransform].
pub struct EsTransformSystem {
    // TODO: Use v8_ctx.
    #[allow(dead_code)]
    v8_ctx: v8wrapper::ContextClient,
}

impl EsTransformSystem {
    pub fn new(v8_ctx: v8wrapper::ContextClient) -> Self {
        Self { v8_ctx }
    }
}

impl generic_pipeline::systems::GenericSystem<crate::PipelineTypes> for EsTransformSystem {
    fn params<'a>(&self, _node: &crate::Node, _reg: &'a mut plparams::NodeParamsRegistrator<'a>) {}

    fn inputs<'a>(&self, node: &crate::Node, reg: &'a mut plinputs::NodeInputsRegistrator<'a>) {
        let spec = match &node.spec {
            specs::Spec::EsTransform(spec) => spec,
            _ => {
                return;
            }
        };

        for dep_id in spec.input_data.values() {
            reg.add_input(dep_id);
        }
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
