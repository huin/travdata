//! Arguments for a [crate::pipeline::GenericPipeline].
//!
//! These provide runtime parameters for the [crate::plparams] for the pipeline.

use crate::{node, plparams};

pub struct GenericArgSet<A> {
    args: hashbrown::HashMap<ParamKey, A>,
}

impl<A> Default for GenericArgSet<A> {
    fn default() -> Self {
        Self {
            args: Default::default(),
        }
    }
}

impl<A> GenericArgSet<A> {
    pub fn set(&mut self, node_id: node::NodeId, param_id: plparams::ParamId, arg: A) {
        self.args.insert(ParamKey { node_id, param_id }, arg);
    }

    pub fn get<'a>(
        &'a self,
        node_id: &node::NodeId,
        param_id: &plparams::ParamId,
    ) -> Option<&'a A> {
        self.args.get(&BorrowedParamKey { node_id, param_id })
    }
}

#[derive(Eq, Hash, PartialEq)]
struct ParamKey {
    node_id: node::NodeId,
    param_id: plparams::ParamId,
}

#[derive(Hash)]
struct BorrowedParamKey<'a> {
    node_id: &'a node::NodeId,
    param_id: &'a plparams::ParamId,
}

impl<'a> hashbrown::Equivalent<ParamKey> for BorrowedParamKey<'a> {
    fn equivalent(&self, key: &ParamKey) -> bool {
        self.node_id == &key.node_id && self.param_id == &key.param_id
    }
}
