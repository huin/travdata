use std::{
    borrow::Cow,
    collections::BTreeSet,
    hash::Hash,
    ops::{Bound, RangeBounds},
};

use hashbrown::{HashMap, hash_map::Entry};
use itertools::Itertools;
use slotmap::{Key as _, KeyData};

use crate::app::data::guinode;

/// Provides an index of nodes. The data is stale and updated by calls to [NodeIndex::index_node].
#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct NodeIndex {
    heap: HashMap<guinode::NodeRef, NodeIndexEntry>,

    /// `node_id_idx` is effectively an ordered multimap from [pipeline::NodeId] to zero or more
    /// [guinode::NodeRef]s.
    node_id_idx: BTreeSet<NodeIdNodeRef<'static>>,

    node_ids: HashMap<guinode::NodeRef, String>,

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

    /// Adds or updates a [guinode::Node] in the index.
    ///
    /// Assumes that a [guinode::NodeRef] value is a stable identifier for a Node throughout the
    /// lifetime of `self`.
    pub fn index_node(&mut self, node_ref: guinode::NodeRef, node: &guinode::Node) {
        match self.heap.entry(node_ref) {
            Entry::Occupied(mut occupied_entry) => {
                let existing_entry = occupied_entry.get_mut();

                if existing_entry.node_id != node.meta.id {
                    // Update `NodeEntry::node_id`, capture the old value.
                    let old_node_id = {
                        let mut node_id = node.meta.id.clone();
                        std::mem::swap(&mut node_id, &mut existing_entry.node_id);
                        node_id
                    };

                    // Remove stale entry from node_id_idx.
                    self.node_id_idx
                        .remove(&NodeIdNodeRef(Cow::Owned(old_node_id), node_ref));
                    // Insert new entry into node_id_idx.
                    self.node_id_idx
                        .insert(NodeIdNodeRef(Cow::Owned(node.meta.id.clone()), node_ref));

                    self.generation.increment();
                }
            }

            Entry::Vacant(vacant_entry) => {
                // Add to heap.
                vacant_entry.insert_entry(NodeIndexEntry {
                    node_ref,
                    node_id: node.meta.id.clone(),
                });
                // Insert new entry into node_id_idx.
                self.node_id_idx
                    .insert(NodeIdNodeRef(Cow::Owned(node.meta.id.clone()), node_ref));

                self.generation.increment();
            }
        };
    }

    /// Removes the node with the given [guinode::NodeRef] from the index.
    pub fn deindex_node(&mut self, node_ref: guinode::NodeRef) {
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

    /// Looks up a node by its [guinode::NodeRef].
    pub fn lookup_node_ref(&self, node_ref: guinode::NodeRef) -> Option<&NodeIndexEntry> {
        self.heap.get(&node_ref)
    }

    /// Looks up a node by its ID from the index. This may match zero to many.
    pub fn lookup_exact_node_id<'idx, 'id>(
        &'idx self,
        id: &'id str,
    ) -> impl Iterator<Item = &'idx NodeIndexEntry> + 'id
    where
        'idx: 'id,
    {
        self.node_id_idx
            .range(
                NodeIdNodeRef::first_for_node_id(Cow::Borrowed(id))
                    ..=NodeIdNodeRef::last_for_node_id(Cow::Borrowed(id)),
            )
            .filter_map(move |idx_entry| self.index_entry(idx_entry))
    }

    /// Scan for nodes by IDs sharing the given prefix.
    pub fn scan_node_id_prefix<'idx, 'id>(
        &'idx self,
        id: &'id str,
    ) -> impl Iterator<Item = &'idx NodeIndexEntry> + 'id
    where
        'idx: 'id,
    {
        let range_bound = if id.is_empty() {
            // Everything matches the empty prefix.
            NodeIdNodeRefScanRangeBounds {
                start: Bound::Unbounded,
                end: Bound::Unbounded,
            }
        } else {
            let last_matching_id: String = successive_prefix(id);
            NodeIdNodeRefScanRangeBounds {
                start: Bound::Included(NodeIdNodeRef::first_for_node_id(Cow::Borrowed(id))),
                end: Bound::Excluded(NodeIdNodeRef::first_for_node_id(Cow::Owned(
                    last_matching_id,
                ))),
            }
        };

        self.node_id_idx
            .range(range_bound)
            .filter_map(move |idx_entry| self.index_entry(idx_entry))
    }

    fn index_entry<'idx>(&'idx self, idx_entry: &NodeIdNodeRef) -> Option<&'idx NodeIndexEntry> {
        match self.heap.get(&idx_entry.1) {
            Some(node_entry) => Some(node_entry),
            None => {
                log::warn!("bug: dangling entry in node_id_idx: {idx_entry:?}");
                None
            }
        }
    }
}

/// Indicator of changes to a [NodeIndex], meaning that any data previously read at an older
/// generation than current is stale.
///
/// If the [NodeIndexGeneration] is equal to the last read, then there have been no changes to the
/// index at all. If they are unequal, then any data from the last read may be stale.
#[derive(Copy, Clone, Debug, Default, Hash, serde::Deserialize, serde::Serialize)]
pub struct NodeIndexGeneration(usize);

impl NodeIndexGeneration {
    /// Compare if `self` and `new_value` are of the same generation (returned directly) and update
    /// `self` to `new_value`.
    #[cfg(test)]
    pub fn is_same_and_update(&mut self, new_value: Self) -> bool {
        let result = self.is_same(new_value);
        *self = new_value;
        result
    }

    /// Compare if `self` and `new_value` are of the same generation.
    #[cfg(test)]
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
#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct NodeIndexEntry {
    node_ref: guinode::NodeRef,
    node_id: String,
}

impl NodeIndexEntry {
    /// Returns the [guinode::NodeRef] uniquely identifying the node.
    pub fn node_ref(&self) -> &guinode::NodeRef {
        &self.node_ref
    }

    /// Returns the [pipeline::NodeId] identifying the node. This is not required to be unique by
    /// the [NodeIndex], to support editing where node IDs may temporarily be equal.
    pub fn node_id(&self) -> &str {
        &self.node_id
    }
}

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
struct NodeIdNodeRef<'a>(Cow<'a, str>, guinode::NodeRef);

impl<'a> NodeIdNodeRef<'a> {
    fn first_for_node_id(node_id: Cow<'a, str>) -> Self {
        Self(node_id, guinode::NodeRef::from(KeyData::from_ffi(u64::MIN)))
    }
    fn last_for_node_id(node_id: Cow<'a, str>) -> Self {
        Self(node_id, guinode::NodeRef::from(KeyData::from_ffi(u64::MAX)))
    }
}

impl<'a> PartialOrd for NodeIdNodeRef<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(Self::cmp(self, other))
    }
}

impl<'a> Ord for NodeIdNodeRef<'a> {
    // Implement PartialOrd and Ord for NodeIdNodeRef because slotmap::KeyData does not define an
    // ordering, but we need to support NodeIdNodeRef::first_for_node_id and
    // NodeIdNodeRef::last_for_node_id.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .cmp(&other.0)
            .then_with(|| self.1.data().as_ffi().cmp(&other.1.data().as_ffi()))
    }
}

#[derive(Debug)]
struct NodeIdNodeRefScanRangeBounds<'a> {
    start: Bound<NodeIdNodeRef<'a>>,
    end: Bound<NodeIdNodeRef<'a>>,
}

impl<'a> RangeBounds<NodeIdNodeRef<'a>> for NodeIdNodeRefScanRangeBounds<'a> {
    fn start_bound(&self) -> Bound<&NodeIdNodeRef<'a>> {
        self.start.as_ref()
    }

    fn end_bound(&self) -> Bound<&NodeIdNodeRef<'a>> {
        self.end.as_ref()
    }
}

/// Generates the [String] that is the successive prefix to `s`.
fn successive_prefix(s: &str) -> String {
    let mut do_append = false;
    let mut next_str: String = s
        .chars()
        .enumerate()
        .circular_array_windows()
        .map(|[(_, cur_char), (next_index, _)]| {
            if next_index > 0 {
                cur_char
            } else {
                // In the rare case that the last char in `s` is char::MAX, leave it as it is, and
                // we append char::MAX below.
                (cur_char..=char::MAX).nth(1).unwrap_or_else(|| {
                    do_append = true;
                    cur_char
                })
            }
        })
        .collect();
    if do_append {
        next_str.push(char::MAX);
    }
    next_str
}
