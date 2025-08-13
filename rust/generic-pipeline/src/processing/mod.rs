//! Processing of a [pipeline::GenericPipeline].

#[cfg(test)]
mod tests;

use std::rc::Rc;

use anyhow::anyhow;
use hashbrown::{HashMap, HashSet};

use crate::{intermediates, node, pipeline, plinputs, plparams, systems};

/// Describes the outcome of an entire processing attempt. It does not attempt to contain the
/// processed data itself, but rather information about the processing.
#[derive(Debug, PartialEq)]
pub struct PipelineOutcome {
    pub node_outcomes: HashMap<node::NodeId, NodeOutcome>,
}

/// Describes the outcome of a single node.
#[derive(Debug)]
pub enum NodeOutcome {
    /// Node processed successfully.
    Success,
    /// Node processed, but unexpectedly. Dependent nodes not processed.
    Unexpected,
    /// Attempted to process the node, but resulted in an error.
    ProcessErrored(anyhow::Error),
    /// No attempt was made to process the node.
    Unprocessed(NodeUnprocessedReason),
    /// Processing encountered an internal error, likely a bug.
    InternalError(anyhow::Error),
}

/// NOTE: the equality comparison does not check any form of equality for underlying errors in the
/// case of [NodeOutcome::ProcessErrored] or [NodeOutcome::InternalError], instead regarding them
/// as equal on the basis of variant selection equality.
impl PartialEq for NodeOutcome {
    fn eq(&self, other: &Self) -> bool {
        use NodeOutcome::*;
        match (self, other) {
            (Success, Success) => true,
            (ProcessErrored(_), ProcessErrored(_)) => true,
            (Unprocessed(unproc_self), Unprocessed(unproc_other)) => unproc_self == unproc_other,
            (InternalError(_), InternalError(_)) => true,
            _ => false,
        }
    }
}

/// Describes the reasons for a single [node::GenericNode] being unprocessed.
#[derive(Debug, PartialEq)]
pub struct NodeUnprocessedReason {
    pub unprocessed_dependencies: HashMap<node::NodeId, UnprocessedDependencyReason>,
}

#[derive(Debug, PartialEq)]
pub enum UnprocessedDependencyReason {
    /// Could not process the node due to failing to process nodes that it depends on. This could
    /// be because a dependency errored during processing, or that there was a dependency cycle.
    Unprocessed,
    /// Dependency was unknown.
    Unknown,
}

/// Processes a [pipeline::GenericPipeline] using the [crate::systems::GenericSystem]s that it was
/// given to process the nodes within.
pub struct GenericProcessor<P> {
    system: Rc<dyn systems::GenericSystem<P>>,
}

impl<P> GenericProcessor<P>
where
    P: crate::PipelineTypes,
{
    pub fn new(system: Rc<dyn systems::GenericSystem<P>>) -> Self {
        Self { system }
    }

    pub fn resolve_params(
        &self,
        nodes: &pipeline::GenericPipeline<P::Spec>,
        _params: &mut plparams::GenericParamsRegistrator<P::ParamType>,
    ) -> plparams::GenericParams<P::ParamType> {
        let mut reg = plparams::GenericParams::<P::ParamType>::registrator();
        for node in nodes.nodes() {
            self.system.params(node, &mut reg.for_node(&node.id));
        }
        reg.build()
    }

    pub fn process(
        &self,
        nodes: &pipeline::GenericPipeline<P::Spec>,
        args: &crate::plargs::GenericArgSet<P::ArgValue>,
    ) -> PipelineOutcome {
        let state = GenericProcessingState::new(nodes, args, self.system.clone());
        state.process()
    }
}

struct GenericProcessingState<'a, P>
where
    P: crate::PipelineTypes,
{
    nodes: &'a pipeline::GenericPipeline<P::Spec>,
    args: &'a crate::plargs::GenericArgSet<P::ArgValue>,

    system: Rc<dyn systems::GenericSystem<P>>,

    // Map from NodeId to the NodeIds that depend on it.
    dep_id_to_dependee_ids: HashMap<node::NodeId, Vec<node::NodeId>>,

    outcome: PipelineOutcome,
    interms: intermediates::GenericIntermediateSet<P::IntermediateValue>,
    processable_ids: HashSet<node::NodeId>,
    // Map from NodeId to the NodeIds that it depends on. This is dynamically updated to remove
    // dependent NodeIds that have been successfully processed (when the value is empty, the
    // key can be scheduled for processing).
    unprocessed_id_to_dep_ids: HashMap<node::NodeId, HashSet<node::NodeId>>,
}

impl<'a, P> GenericProcessingState<'a, P>
where
    P: crate::PipelineTypes,
{
    fn new(
        nodes: &'a pipeline::GenericPipeline<P::Spec>,
        args: &'a crate::plargs::GenericArgSet<P::ArgValue>,
        system: Rc<dyn systems::GenericSystem<P>>,
    ) -> Self {
        log::debug!("Processing {} nodes total.", nodes.nodes().count());

        let mut inputs_reg = plinputs::InputsRegistrator::new();
        for node in nodes.nodes() {
            system.inputs(node, &mut inputs_reg.for_node(&node.id));
        }
        // Map from NodeId to the NodeIds that it depends on.
        let unprocessed_id_to_dep_ids = inputs_reg.build();

        let mut processable_ids: HashSet<node::NodeId> = HashSet::new();
        // Map from NodeId to the NodeIds that depend on it.
        let mut dep_id_to_dependee_ids = HashMap::<node::NodeId, Vec<node::NodeId>>::new();

        for node in nodes.nodes() {
            match unprocessed_id_to_dep_ids.get(&node.id) {
                Some(dependency_ids) if !dependency_ids.is_empty() => {
                    for dep_id in dependency_ids {
                        dep_id_to_dependee_ids
                            .entry_ref(dep_id)
                            .or_default()
                            .push(node.id.clone());
                    }
                }
                _ => {
                    // The node has no dependencies, so it is immediately processable.
                    processable_ids.insert(node.id.clone());
                }
            }
        }

        Self {
            nodes,
            args,

            system,

            dep_id_to_dependee_ids,

            outcome: PipelineOutcome {
                node_outcomes: HashMap::with_capacity(nodes.len()),
            },
            interms: intermediates::GenericIntermediateSet::new(),
            processable_ids,
            unprocessed_id_to_dep_ids,
        }
    }

    fn process(mut self) -> PipelineOutcome {
        while !self.processable_ids.is_empty() {
            log::debug!(
                "Processing {} nodes in this pass.",
                self.processable_ids.len()
            );

            let phase_nodes: Vec<_> = self.gather_phase_nodes();

            if phase_nodes.is_empty() {
                log::error!(
                    "Found no further processable nodes, but {} unprocessed node(s) remain. Earlier processes may have failed.",
                    self.unprocessed_id_to_dep_ids.len()
                );
                break;
            }

            let id_intermediates =
                self.system
                    .process_multiple(&phase_nodes, self.args, &self.interms);

            let mut newly_processable_ids = HashSet::new();
            for node_result in id_intermediates {
                if !self.processable_ids.remove(&node_result.id) {
                    log::error!(
                        "Node {:?} was processed, despite not being requested to process. Faulty system?",
                        node_result.id,
                    );
                    self.outcome
                        .node_outcomes
                        .insert(node_result.id, NodeOutcome::Unexpected);
                    continue;
                }

                self.process_result(node_result, &mut newly_processable_ids);
            }

            for node_id in self.processable_ids.drain() {
                let err = anyhow!(
                    "Node {node_id:?} was not processed, despite being requested to process. Faulty system?",
                );
                log::error!("{err:?}");
                self.outcome
                    .node_outcomes
                    .insert(node_id, NodeOutcome::ProcessErrored(err));
            }

            self.processable_ids.extend(newly_processable_ids.drain());
        }

        for (unprocessed_id, mut dep_ids) in self.unprocessed_id_to_dep_ids.drain() {
            log::error!("Node {unprocessed_id:?} was not processed.");
            self.outcome.node_outcomes.insert(
                unprocessed_id,
                NodeOutcome::Unprocessed(NodeUnprocessedReason {
                    unprocessed_dependencies: dep_ids
                        .drain()
                        .map(|dep_id| {
                            let reason = if self.nodes.get(&dep_id).is_some() {
                                UnprocessedDependencyReason::Unprocessed
                            } else {
                                UnprocessedDependencyReason::Unknown
                            };
                            (dep_id, reason)
                        })
                        .collect(),
                }),
            );
        }

        self.outcome
    }

    fn gather_phase_nodes(&self) -> Vec<&'a node::GenericNode<P::Spec>> {
        self.processable_ids
            .iter()
            .filter_map(|node_id| {
                if let Some(node) = self.nodes.get(node_id) {
                    Some(node)
                } else {
                    log::error!("Failed to resolve processable node with ID {node_id:?}.");
                    None
                }
            })
            .collect()
    }

    fn process_result(
        &mut self,
        node_result: systems::NodeResult<P::IntermediateValue>,
        newly_processable_ids: &mut HashSet<node::NodeId>,
    ) {
        match node_result.value {
            Ok(interm) => {
                log::info!("Node {:?} processed successfully.", node_result.id);

                self.mark_dependent_nodes_processable(&node_result.id, newly_processable_ids);

                self.outcome
                    .node_outcomes
                    .insert(node_result.id.clone(), NodeOutcome::Success);
                self.interms.set(node_result.id, interm);
            }
            Err(err) => {
                log::error!("Error processing node {:?}: {err:?}", node_result.id);
                self.outcome
                    .node_outcomes
                    .insert(node_result.id.clone(), NodeOutcome::ProcessErrored(err));
            }
        }
    }

    /// Updates unprocessed_id_to_dep_ids and find newly processable nodes in the process.
    fn mark_dependent_nodes_processable(
        &mut self,
        processed_node_id: &node::NodeId,
        newly_processable_ids: &mut HashSet<node::NodeId>,
    ) {
        let dependee_ids = match self.dep_id_to_dependee_ids.get(processed_node_id) {
            Some(dependee_ids) => dependee_ids,
            None => return,
        };

        for dependee_id in dependee_ids {
            use hashbrown::hash_map::EntryRef;
            match self.unprocessed_id_to_dep_ids.entry_ref(dependee_id) {
                EntryRef::Occupied(mut occupied_entry) => {
                    if !occupied_entry.get_mut().remove(processed_node_id) {
                        let err = anyhow!(
                            "Could not remove node {processed_node_id:?} from node {dependee_id:?}'s unprocessed dependencies. Bug in processor?"
                        );
                        log::error!("{err:?}");
                        self.outcome
                            .node_outcomes
                            .insert(dependee_id.clone(), NodeOutcome::InternalError(err));
                    }
                    if occupied_entry.get().is_empty() {
                        let removed = occupied_entry.remove_entry();
                        log::debug!("Newly processable node {:?}.", removed.0);
                        newly_processable_ids.insert(removed.0);
                    }
                }
                EntryRef::Vacant(_vacant_entry) => {
                    let err = anyhow!(
                        "Unexpected vacant entry for dependees of {processed_node_id:?}. Bug in processor?",
                    );
                    log::error!("{err:?}");
                    self.outcome
                        .node_outcomes
                        .insert(dependee_id.clone(), NodeOutcome::InternalError(err));
                }
            }
        }
    }
}
