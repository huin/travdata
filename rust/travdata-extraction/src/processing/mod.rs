// TODO: Remove this allowance.
#![allow(dead_code)]

use crate::{
    node::{self, spec},
    systems,
};

/// Processes a set of [crate::node::Node]s using the [crate::systems::System]s that it was given.
pub struct GenericProcessor<S>
where
    S: node::SpecTrait,
{
    system: systems::GenericMetaSystem<S>,
}

impl<S> GenericProcessor<S>
where
    S: node::SpecTrait,
{
    pub fn new(system: systems::GenericMetaSystem<S>) -> Self {
        Self { system }
    }
}

/// Specific [GenericProcessor] used in actual processing.
pub type Processor = GenericProcessor<spec::Spec>;
