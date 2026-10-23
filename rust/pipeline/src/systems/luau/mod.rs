use crate::{
    Node, NodeId, SystemError, SystemErrorKind, SystemResult, intermediates,
    monomorph::{ArgSet, NodeInputsRegistrator},
    specs::{self, LuauContext, LuauTransform},
};

/// Provides processing support for [crate::specs::Spec::LuauContext] and
/// [crate::specs::Spec::LuauTransform].
#[derive(Default)]
pub struct LuauSystem;

impl generic_pipeline::systems::GenericSystem<crate::PipelineTypes> for LuauSystem {
    fn process(
        &self,
        node: &Node,
        _args: &ArgSet,
        intermediates: &crate::intermediates::IntermediateSet,
    ) -> Result<
        <crate::PipelineTypes as generic_pipeline::PipelineTypes>::IntermediateValue,
        <crate::PipelineTypes as generic_pipeline::PipelineTypes>::SystemError,
    > {
        match node.spec {
            specs::Spec::LuauContext(ctx) => {}
            specs::Spec::LuauTransform(trn) => {}
            _ => Err(SystemErrorKind::Internal(())),
        }
        // let spec: &JsTransform = node.spec.downcast()?;
        //
        // reg.add_input(&spec.context);
        // for node_id in spec.input_data.values() {
        //     reg.add_input(node_id);
        // }
    }
}
