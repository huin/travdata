//! Parameters for a [crate::pipeline::GenericPipeline].

use hashbrown::HashMap;

use crate::node;

/// ID of a parameter, within the namespace of the [node::GenericNode] that it is for.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ParamId(&'static str);

impl ParamId {
    pub fn from_static(id: &'static str) -> Self {
        Self(id)
    }
}

impl AsRef<str> for ParamId {
    fn as_ref(&self) -> &str {
        self.0
    }
}

/// Describes an input parameter for processing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericParam<P> {
    /// Human-readable description of the parameter.
    pub description: String,
    /// What semenatic type of value of the argument.
    pub param_type: P,
}

/// A [GenericParam] qualified by its [node::NodeId].
#[derive(Debug)]
pub struct GenericNodeParam<P> {
    pub node_id: node::NodeId,
    pub param: GenericParam<P>,
}

/// Key for a parameter within [GenericParams::params].
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct ParamKey {
    node_id: node::NodeId,
    param_id: ParamId,
}

impl ParamKey {
    /// Creates a new [ParamKey].
    pub fn new(node_id: node::NodeId, param_id: ParamId) -> Self {
        Self { node_id, param_id }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct GenericParams<P> {
    pub params: HashMap<ParamKey, GenericParam<P>>,
}

impl<P> GenericParams<P> {
    pub fn registrator() -> GenericParamsRegistrator<P> {
        GenericParamsRegistrator {
            params: Default::default(),
        }
    }
}

/// Registers pipeline parameters for nodes.
pub struct GenericParamsRegistrator<P> {
    params: HashMap<ParamKey, GenericParam<P>>,
}

impl<P> GenericParamsRegistrator<P> {
    /// Returns a [GenericNodeParamsRegistrator] for registering parameters for the given
    /// [node::NodeId].
    pub fn for_node<'a>(
        &'a mut self,
        node_id: &'a node::NodeId,
    ) -> GenericNodeParamsRegistrator<'a, P> {
        GenericNodeParamsRegistrator { node_id, reg: self }
    }

    /// Consumes the [GenericParamsRegistrator] and returns the built up parameters.
    pub fn build(self) -> GenericParams<P> {
        GenericParams {
            params: self.params,
        }
    }
}

/// Registers pipeline parameters for a single node.
pub struct GenericNodeParamsRegistrator<'a, P> {
    node_id: &'a node::NodeId,
    reg: &'a mut GenericParamsRegistrator<P>,
}

impl<'builder, P> GenericNodeParamsRegistrator<'builder, P> {
    /// Registers a single parameter for the [node::NodeId].
    pub fn add_param(&mut self, param_id: ParamId, param_type: P, description: String) {
        self.reg.params.insert(
            ParamKey::new(self.node_id.clone(), param_id),
            GenericParam {
                param_type,
                description,
            },
        );
    }
}
