#[cfg(test)]
mod tests;

use std::{borrow::Cow, collections::BTreeSet};

use hashbrown::{HashMap, hash_map::Entry};

use crate::app::data::{self, node::GuiNodeWithId};

/// Provides an index of nodes. The data is stale and updated by calls to [NodeIndex::index_node].
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct NodeIndex {
    heap: HashMap<data::NodeRef, NodeIndexEntry>,

    /// `node_id_idx` is effectively an ordered multimap from [pipeline::NodeId] to zero or more
    /// [data::NodeRef]s.
    node_id_idx: BTreeSet<NodeIdNodeRef<'static>>,

    node_ids: HashMap<data::NodeRef, String>,

    generation: NodeIndexGeneration,
}

impl NodeIndex {
    /// Creates an empty [NodeIndex].
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            heap: HashMap::with_capacity(capacity),
            node_id_idx: BTreeSet::new(),
            node_ids: HashMap::with_capacity(capacity),
            generation: NodeIndexGeneration::default(),
        }
    }

    /// Returns the current generation of the [NodeIndex].
    ///
    /// If results are read from the index and cached, then a subsequent comparison to this value
    /// will indicate if the cached data could be stale.
    pub fn generation(&self) -> NodeIndexGeneration {
        self.generation
    }

    /// Adds or updates a [data::GuiNodeWithId] in the index.
    ///
    /// Assumes that a [data::NodeRef] value is a stable identifier for a Node throughout the
    /// lifetime of `self`.
    pub fn index_node(&mut self, node_ref: data::NodeRef, node: &GuiNodeWithId) {
        match self.heap.entry(node_ref) {
            Entry::Occupied(mut occupied_entry) => {
                let existing_entry = occupied_entry.get_mut();

                if existing_entry.node_id != node.node_id {
                    // Update `NodeEntry::node_id`, capture the old value.
                    let old_node_id = {
                        let mut node_id = node.node_id.clone();
                        std::mem::swap(&mut node_id, &mut existing_entry.node_id);
                        node_id
                    };

                    // Remove stale entry from node_id_idx.
                    self.node_id_idx
                        .remove(&NodeIdNodeRef(Cow::Owned(old_node_id), node_ref));
                    // Insert new entry into node_id_idx.
                    self.node_id_idx
                        .insert(NodeIdNodeRef(Cow::Owned(node.node_id.clone()), node_ref));

                    self.generation.increment();
                }
            }

            Entry::Vacant(vacant_entry) => {
                // Add to heap.
                vacant_entry.insert_entry(NodeIndexEntry {
                    node_ref,
                    node_id: node.node_id.clone(),
                });
                // Insert new entry into node_id_idx.
                self.node_id_idx
                    .insert(NodeIdNodeRef(Cow::Owned(node.node_id.clone()), node_ref));

                self.generation.increment();
            }
        };
    }

    /// Removes the node with the given [data::NodeRef] from the index.
    pub fn deindex_node(&mut self, node_ref: data::NodeRef) {
        let entry = self.heap.remove(&node_ref);

        let entry = if let Some(entry) = entry {
            entry
        } else {
            return;
        };

        self.node_id_idx
            .remove(&NodeIdNodeRef(Cow::Owned(entry.node_id), node_ref));

        self.generation.increment();
    }

    pub fn by_node_id<'idx, 'id>(
        &'idx self,
        id: &'id str,
    ) -> impl Iterator<Item = &'idx NodeIndexEntry> + 'id
    where
        'idx: 'id,
    {
        self.node_id_idx
            .range(
                NodeIdNodeRef(Cow::Borrowed(id), data::NodeRef::MIN)
                    ..=NodeIdNodeRef(Cow::Borrowed(id), data::NodeRef::MAX),
            )
            .filter_map(move |idx_entry| match self.heap.get(&idx_entry.1) {
                Some(node_entry) => Some(node_entry),
                None => {
                    log::warn!("bug: dangling entry in node_id_idx: {idx_entry:?}");
                    None
                }
            })
    }
}

/// Indicator of changes to a [NodeIndex], meaning that any data previously read at an older
/// generation than current is stale.
///
/// If the [NodeIndexGeneration] is equal to the last read, then there have been no changes to the
/// index at all. If they are unequal, then any data from the last read may be stale.
#[derive(Copy, Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct NodeIndexGeneration(usize);

impl NodeIndexGeneration {
    /// Compare if `self` and `new_value` are of the same generation (returned directly) and update
    /// `self` to `new_value`.
    pub fn is_same_and_update(&mut self, new_value: Self) -> bool {
        let result = self.is_same(new_value);
        *self = new_value;
        result
    }

    /// Compare if `self` and `new_value` are of the same generation.
    pub fn is_same(&mut self, other: Self) -> bool {
        self.0 == other.0
    }

    /// In-place increments to the next generation.
    fn increment(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

/// Indexed data about a node.
///
/// NOTE: This data will be stale between an update to the node and the update of the [NodeIndex].
#[derive(serde::Deserialize, serde::Serialize)]
pub struct NodeIndexEntry {
    node_ref: data::NodeRef,
    node_id: String,
}

impl NodeIndexEntry {
    /// Returns the [data::NodeRef] uniquely identifying the node.
    pub fn node_ref(&self) -> &data::NodeRef {
        &self.node_ref
    }

    /// Returns the [pipeline::NodeId] identifying the node. This is not required to be unique by
    /// the [NodeIndex], to support editing where node IDs may temporarily be equal.
    pub fn node_id(&self) -> &str {
        &self.node_id
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize)]
struct NodeIdNodeRef<'a>(Cow<'a, str>, data::NodeRef);
