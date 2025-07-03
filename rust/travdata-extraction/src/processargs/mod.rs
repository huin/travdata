use std::path::PathBuf;

use crate::{node::core_type, processparams};

/// Typed value of an argument to a [crate::node::Node].
pub enum Arg {
    InputPdf(PathBuf),
    OutputDirectory(PathBuf),
}

#[derive(Default)]
pub struct ArgSet {
    args: hashbrown::HashMap<ParamKey, Arg>,
}

impl ArgSet {
    pub fn set(&mut self, node_id: core_type::NodeId, param_id: processparams::ParamId, arg: Arg) {
        self.args.insert(ParamKey { node_id, param_id }, arg);
    }

    pub fn get<'a>(
        &'a self,
        node_id: &core_type::NodeId,
        param_id: &processparams::ParamId,
    ) -> Option<&'a Arg> {
        self.args.get(&BorrowedParamKey { node_id, param_id })
    }
}

#[derive(Eq, Hash, PartialEq)]
struct ParamKey {
    node_id: core_type::NodeId,
    param_id: processparams::ParamId,
}

#[derive(Hash)]
struct BorrowedParamKey<'a> {
    node_id: &'a core_type::NodeId,
    param_id: &'a processparams::ParamId,
}

impl<'a> hashbrown::Equivalent<ParamKey> for BorrowedParamKey<'a> {
    fn equivalent(&self, key: &ParamKey) -> bool {
        self.node_id == &key.node_id && self.param_id == &key.param_id
    }
}
