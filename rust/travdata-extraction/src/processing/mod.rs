// TODO: Remove this allowance.
#![allow(dead_code)]

use crate::systems;

/// Processes a set of [crate::node::Node]s using the [crate::systems::System]s that it was given.
pub struct Processor {
    system: systems::MetaSystem,
}

impl Processor {
    pub fn new(system: systems::MetaSystem) -> Self {
        Self { system }
    }
}
