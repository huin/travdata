#[cfg(test)]
mod tests;

use std::rc::Rc;

use anyhow::Result;
use hashbrown::{HashMap, HashSet};

use crate::{
    intermediates,
    node::{self, spec},
    processparams, systems,
};

/// Immutable set of [node::Node]s, indexed for processing.
pub struct GenericNodeSet<S> {
    id_to_node: HashMap<node::NodeId, node::GenericNode<S>>,
}

impl<S> GenericNodeSet<S> {
    pub fn new(nodes: impl IntoIterator<Item = node::GenericNode<S>>) -> Self {
        let id_to_node = nodes
            .into_iter()
            .map(|node| (node.id.clone(), node))
            .collect();
        Self { id_to_node }
    }

    /// Returns an [Iterator] over all [node::GenericNode]s in the set.
    pub fn nodes(&self) -> impl Iterator<Item = &node::GenericNode<S>> {
        self.id_to_node.values()
    }

    /// Returns the [node::GenericNode] for the given [node::NodeId].
    pub fn get(&self, node_id: &node::NodeId) -> Option<&node::GenericNode<S>> {
        self.id_to_node.get(node_id)
    }
}

/// Specific [GenericNodeSet] used in actual processing.
pub type NodeSet = GenericNodeSet<spec::Spec>;

/// Processes a set of [crate::node::Node]s using the [crate::systems::System]s that it was given.
pub struct GenericProcessor<S>
where
    S: node::SpecTrait,
{
    system: Rc<dyn systems::GenericSystem<S>>,
}

impl<S> GenericProcessor<S>
where
    S: node::SpecTrait,
{
    pub fn new(system: Rc<dyn systems::GenericSystem<S>>) -> Self {
        Self { system }
    }

    pub fn resolve_params(&self, nodes: &GenericNodeSet<S>) -> processparams::NodeParams {
        processparams::NodeParams {
            params: nodes
                .id_to_node
                .iter()
                .flat_map(|(id, node)| {
                    self.system.params(node).params.into_iter().map(|param| {
                        processparams::NodeParam {
                            node_id: id.clone(),
                            param,
                        }
                    })
                })
                .collect(),
        }
    }

    pub fn process(
        &self,
        nodes: &GenericNodeSet<S>,
        args: &crate::processargs::ArgSet,
    ) -> Result<()> {
        log::debug!("Processing {} nodes total.", nodes.nodes().count());

        let mut processable_ids: HashSet<node::NodeId> = HashSet::new();
        // Map from NodeId to the NodeIds that depend on it.
        let mut dep_id_to_dependee_ids: HashMap<node::NodeId, Vec<node::NodeId>> = HashMap::new();
        // Map from NodeId to the NodeIds that it depends on. This is dynamically updated to remove
        // dependent NodeIds that have been successfully processed (when the value is empty, the
        // key can be scheduled for processing).
        let mut unprocessed_id_to_dep_ids: HashMap<node::NodeId, HashSet<node::NodeId>> =
            HashMap::new();
        for node in nodes.nodes() {
            let deps = self.system.inputs(node);
            if deps.is_empty() {
                processable_ids.insert(node.id.clone());
            } else {
                for dep_id in &deps {
                    dep_id_to_dependee_ids
                        .entry_ref(dep_id)
                        .or_default()
                        .push(node.id.clone());
                }

                unprocessed_id_to_dep_ids.insert(node.id.clone(), deps.into_iter().collect());
            }
        }
        // Make dep_id_to_dependee_ids immutable following initialisation.
        let dep_id_to_dependee_ids = dep_id_to_dependee_ids;

        let mut interms = intermediates::IntermediateSet::new();

        while !processable_ids.is_empty() {
            log::debug!("Processing {} nodes in this pass.", processable_ids.len());

            let phase_nodes: Vec<_> = processable_ids
                .iter()
                .filter_map(|node_id| {
                    if let Some(node) = nodes.get(node_id) {
                        Some(node)
                    } else {
                        log::error!("Failed to resolve processable node with ID {:?}.", node_id);
                        None
                    }
                })
                .collect();

            if phase_nodes.is_empty() {
                log::error!(
                    "Found no further processable nodes, but {} unprocessed node(s) remain. Earlier processes may have failed.",
                    unprocessed_id_to_dep_ids.len()
                );
                break;
            }

            let id_intermediates = self.system.process_multiple(&phase_nodes, args, &interms);

            let mut newly_processable_ids = HashSet::new();
            for (processed_node_id, interm_result) in id_intermediates {
                if !processable_ids.remove(&processed_node_id) {
                    log::error!(
                        "Node {:?} was processed, despite not being requested to process. Faulty system?",
                        processed_node_id
                    );
                    continue;
                }

                match interm_result {
                    Ok(interm) => {
                        log::info!("Node {:?} processed successfully.", processed_node_id);

                        // Update unprocessed_id_to_dep_ids and fnd newly processable nodes in the
                        // process.
                        if let Some(dependee_ids) = dep_id_to_dependee_ids.get(&processed_node_id) {
                            for dependee_id in dependee_ids {
                                use hashbrown::hash_map::EntryRef;
                                match unprocessed_id_to_dep_ids.entry_ref(dependee_id) {
                                    EntryRef::Occupied(mut occupied_entry) => {
                                        if !occupied_entry.get_mut().remove(&processed_node_id) {
                                            log::error!(
                                                "Could not remove node {:?} from node {:?}'s unprocessed dependencies. Bug in processor?",
                                                processed_node_id,
                                                dependee_id,
                                            );
                                        }
                                        if occupied_entry.get().is_empty() {
                                            let removed = occupied_entry.remove_entry();
                                            log::debug!("Newly processable node {:?}.", removed.0);
                                            newly_processable_ids.insert(removed.0);
                                        }
                                    }
                                    EntryRef::Vacant(_vacant_entry) => {
                                        log::error!(
                                            "Unexpected vacant entry for dependees of {:?}. Bug in processor?",
                                            processed_node_id
                                        );
                                    }
                                }
                            }
                        }

                        interms.set(processed_node_id, interm);
                    }
                    Err(err) => {
                        log::error!("Error processing node {:?}: {:?}", processed_node_id, err);
                    }
                }
            }

            for node_id in processable_ids.drain() {
                log::error!(
                    "Node {:?} was not processed, despite being requested to process. Faulty system?",
                    node_id
                );
            }

            processable_ids.extend(newly_processable_ids.drain());
        }

        for unprocessed_id in unprocessed_id_to_dep_ids.keys() {
            log::error!("Node {:?} was not processed.", unprocessed_id);
        }

        Ok(())
    }
}

/// Specific [GenericProcessor] used in actual processing.
pub type Processor = GenericProcessor<spec::Spec>;
