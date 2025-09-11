#[cfg(test)]
mod tests;

use anyhow::bail;
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
        node: &crate::Node,
        _args: &crate::plargs::ArgSet,
        _intermediates: &crate::intermediates::IntermediateSet,
    ) -> anyhow::Result<crate::intermediates::IntermediateValue> {
        let spec = match &node.spec {
            specs::Spec::EsTransform(spec) => spec,
            _ => {
                bail!("node is not of type EsTransform");
            }
        };

        let mut arg_names: Vec<&str> = spec.input_data.keys().map(String::as_str).collect();
        // Sort the argument names for consistent ordering of arguments, in case any EsTransform
        // nodes rely on ordering.
        arg_names.sort();

        // XXX use _result
        let _result = self.v8_ctx.run(|try_catch| {
            // XXX
            let func_v8 = v8wrapper::new_v8_function(
                try_catch,
                &arg_names,
                &v8wrapper::ESScriptOrigin {
                    resource_name: format!("<node/{}>", node.id.as_ref()),
                    resource_line_offset: 0,
                    resource_column_offset: todo!(),
                    script_id: todo!(),
                },
                &spec.code,
            );
            Ok(())
        })?;
        // TODO: Use `Object.freeze` to freeze any data passed in. This means that any future
        // batching in `process_multiple` that would require it is not a breaking change.
        // todo!()
        bail!("XXX todo")
    }

    // TODO: Optionally implement `process_multiple` as there might be some possible batching
    // optimisations there.
}
