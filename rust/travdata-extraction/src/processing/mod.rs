// TODO: Remove this allowance.
#![allow(dead_code)]

use crate::{node::spec, systems};

pub struct Processor {
    systems: hashbrown::HashMap<spec::SpecDiscriminants, Box<dyn systems::System>>,
    default_system: systems::MissingSystem,
}

impl Processor {
    pub fn new(
        systems: hashbrown::HashMap<spec::SpecDiscriminants, Box<dyn systems::System>>,
    ) -> Self {
        Self {
            systems,
            default_system: systems::MissingSystem,
        }
    }

    fn system_for(&self, spec_type: spec::SpecDiscriminants) -> &dyn systems::System {
        self.systems
            .get(&spec_type)
            .map(Box::as_ref)
            .unwrap_or(&self.default_system)
    }
}

impl Default for Processor {
    fn default() -> Self {
        let systems = hashbrown::HashMap::new();
        Self::new(systems)
    }
}
