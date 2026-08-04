#[cfg(test)]
mod tests;

use thiserror_context::Context;
use v8wrapper::CatchToResult;

use crate::{
    Node, NodeId, SystemError, SystemResult, intermediates,
    monomorph::{ArgSet, NodeInputsRegistrator},
    specs::JsTransform,
};

/// Provides processing support for [crate::specs::Spec::JsTransform].
#[derive(Default)]
pub struct JsTransformSystem;

impl generic_pipeline::systems::GenericSystem<crate::PipelineTypes> for JsTransformSystem {
    fn inputs<'a>(
        &self,
        node: &crate::Node,
        reg: &'a mut NodeInputsRegistrator<'a>,
    ) -> SystemResult<()> {
        let spec: &JsTransform = node.spec.downcast()?;

        reg.add_input(&spec.context);
        for dep_id in spec.input_data.values() {
            reg.add_input(dep_id);
        }

        Ok(())
    }

    fn process(
        &self,
        node: &Node,
        _args: &ArgSet,
        intermediates: &crate::intermediates::IntermediateSet,
    ) -> SystemResult<crate::intermediates::IntermediateValue> {
        let spec: &JsTransform = node.spec.downcast()?;

        let global_context: &intermediates::JsContext =
            intermediates::get_intermediate_input(intermediates, &spec.context)?;

        let mut arg_refs: Vec<(&str, &NodeId)> = spec
            .input_data
            .iter()
            .map(|(arg_name, node_id)| (arg_name.as_str(), node_id))
            .collect();
        // Sort the argument names for consistent ordering of arguments, in case any JsTransform
        // nodes rely on ordering.
        arg_refs.sort_by_key(|(arg_name, _)| *arg_name);

        let result = v8wrapper::try_with_isolate(
            |tls_isolate| -> SystemResult<serde_json::Value> {
                v8::scope!(let scope, tls_isolate.isolate());
                let ctx = v8::Local::new(scope, &global_context.0);
                v8::scope_with_context!(let scope, scope, ctx);

                let arg_names: Vec<&str> = arg_refs.iter().map(|(arg_name, _)| *arg_name).collect();

                v8::tc_scope!(let try_catch, scope);

                // Create the transformation function.
                let func = v8wrapper::new_v8_function(
                    try_catch,
                    &arg_names,
                    &v8wrapper::ESScriptOrigin {
                        resource_name: format!("nodes[{:?}].spec.code", node.meta.id),
                        is_module: false,
                        ..Default::default()
                    },
                    &spec.code,
                )
                .map_err(SystemError::map_spec())
                .context("creating transformation function")?;

                // Collect the arguments to call it with.
                let args: Vec<v8::Local<v8::Value>> = arg_refs
                    .iter()
                    .map(
                        |(arg_name, node_id)| -> SystemResult<v8::Local<v8::Value>> {
                            let arg_json_value: &intermediates::JsonData = intermediates::get_intermediate_input(
                            intermediates,
                                node_id
                            )?;

                            serde_v8::to_v8(try_catch, &arg_json_value.0)
                                .map_err(SystemError::map_input_value(node_id))
                                // TODO: Use `Object.freeze` to freeze any data passed in. This means
                                // that any future batching in `process_multiple` that would require it
                                // is not a breaking change. todo!()
                                .with_context(|| {
                                    format!("converting JsonData to v8::Value for argument {arg_name:?} from node {node_id:?}")
                                })
                        },
                    )
                    .collect::<SystemResult<Vec<_>>>()?;

                // Call the transformation function.
                let global = ctx.global(try_catch);
                let result_v8 = func
                    .call(try_catch, global.cast(), &args)
                    .to_exception_result(try_catch)
                    .map_err(SystemError::map_spec())
                    .context("calling transformation function")?;

                // Transform the result back to JsonData.
                let result: serde_json::Value = serde_v8::from_v8(try_catch, result_v8).map_err(
                    SystemError::map_spec(),
                ).context("converting result v8::Value to JsonData")?;

                Ok(result)
            },
        ).map_err(SystemError::map_internal())??;

        Ok(intermediates::JsonData(result).into())
    }

    // TODO: Optionally implement `process_multiple` as there might be some possible batching
    // optimisations there.
}
