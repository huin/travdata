use std::mem::swap;

use docgroup::GroupToken;
use doctable::TableToken;

use crate::{clock, templatedoc::docgroup::GroupDataMutator};

use super::*;

/// Maximum time delta between edits that can be merged into a single edit.
static MAX_MERGE_EDIT_TIME_DELTA: std::sync::Mutex<chrono::TimeDelta> =
    std::sync::Mutex::<chrono::TimeDelta>::new(chrono::TimeDelta::seconds(10));

pub fn set_max_merge_edit_time_delta(td: chrono::TimeDelta) {
    let mut td_guard = MAX_MERGE_EDIT_TIME_DELTA.lock().unwrap();
    *td_guard = td;
}

pub fn get_max_merge_edit_time_delta() -> chrono::TimeDelta {
    *MAX_MERGE_EDIT_TIME_DELTA.lock().unwrap()
}

pub trait CheckedEdit {
    type Target;

    fn check(&self, target: &Self::Target) -> Result<(), EditError>;
}

/// An edit that includes a timestamp.
///
/// It will only allow a merge if the timestamps of two [TimestampedEdit]s are within
/// [get_max_merge_edit_time_delta] of each other.
pub struct TimestampedEdit {
    ts: clock::Timestamp,
    edit: EditDocumentState,
}

impl TimestampedEdit {
    pub fn new(ts: clock::Timestamp, edit: EditDocumentState) -> Self {
        Self { ts, edit }
    }
}

impl undo::Edit for TimestampedEdit {
    type Target = DocumentState;

    type Output = Result<(), EditError>;

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        self.edit.edit(target)
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        self.edit.undo(target)
    }

    fn merge(&mut self, other: Self) -> undo::Merged<Self>
    where
        Self: Sized,
    {
        use undo::Merged::*;

        if other.ts - self.ts > get_max_merge_edit_time_delta() {
            return No(other);
        }

        let TimestampedEdit {
            ts: other_ts,
            edit: other_edit,
        } = other;
        map_merge(self.edit.merge(other_edit), |other_edit| TimestampedEdit {
            ts: other_ts,
            edit: other_edit,
        })
    }
}

/// A discrete change to an open [DocumentState].
pub enum EditDocumentState {
    Group { group: GroupToken, edit: EditGroup },
    Table { table: TableToken, edit: EditTable },
}

impl undo::Edit for EditDocumentState {
    type Target = DocumentState;

    type Output = Result<(), EditError>;

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        use EditDocumentState::*;

        match self {
            Group { group, edit } => {
                let mut mutator = target.group_mutator(*group)?;
                edit.edit(&mut mutator)
            }
            Table { table, edit } => {
                let table = target.table_arena.get_mut_inner(*table)?;
                edit.edit(table)
            }
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        use EditDocumentState::*;

        match self {
            Group { group, edit } => {
                let mut mutator = target.group_mutator(*group)?;
                edit.undo(&mut mutator)
            }
            Table { table, edit } => {
                let table = target.table_arena.get_mut_inner(*table)?;
                edit.undo(table)
            }
        }
    }

    fn merge(&mut self, other: Self) -> undo::Merged<Self>
    where
        Self: Sized,
    {
        use EditDocumentState::*;

        match (self, other) {
            (
                Group { group, edit },
                Group {
                    group: other_group,
                    edit: other_edit,
                },
            ) if group == &other_group => map_merge(edit.merge(other_edit), |other_edit| {
                EditDocumentState::Group {
                    group: other_group,
                    edit: other_edit,
                }
            }),
            (
                Table { table, edit },
                Table {
                    table: other_table,
                    edit: other_edit,
                },
            ) if table == &other_table => map_merge(edit.merge(other_edit), |other_edit| {
                EditDocumentState::Table {
                    table: other_table,
                    edit: other_edit,
                }
            }),
            (_, other) => undo::Merged::No(other),
        }
    }
}

impl CheckedEdit for EditDocumentState {
    type Target = DocumentState;

    fn check(&self, target: &Self::Target) -> Result<(), EditError> {
        use EditDocumentState::*;

        match self {
            Group { group, edit } => {
                let group = target.group_arena.get_inner(*group)?;
                edit.check(group)
            }
            Table { table, edit } => {
                let table = target.table_arena.get_inner(*table)?;
                edit.check(table)
            }
        }
    }
}

pub enum EditGroup {
    SetName { new_name: String, old_name: String },
    AddTag(String),
    RemoveTag(String),
}

XXX urg this associated type :(

impl<'a> undo::Edit for EditGroup {
    type Target = GroupDataMutator<'a>;

    type Output = Result<(), EditError>;

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        use EditGroup::*;

        match self {
            SetName { new_name, .. } => {
                target.set_name(new_name.clone());
            }
            AddTag(tag) => {
                target.add_tag(tag.clone());
            }
            RemoveTag(tag) => {
                target.remove_tag(&tag);
            }
        }

        Ok(())
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        use EditGroup::*;

        match self {
            SetName { old_name, .. } => {
                target.set_name(old_name.clone());
            }
            AddTag(tag) => {
                target.remove_tag(tag.as_str());
            }
            RemoveTag(tag) => {
                target.add_tag(tag.clone());
            }
        }

        Ok(())
    }

    fn merge(&mut self, mut other: Self) -> undo::Merged<Self>
    where
        Self: Sized,
    {
        use EditGroup::*;

        match (self, &mut other) {
            (
                SetName {
                    new_name: self_new_name,
                    ..
                },
                SetName {
                    new_name: other_new_name,
                    ..
                },
            ) => {
                swap(other_new_name, self_new_name);
                undo::Merged::Yes
            }
            _ => undo::Merged::No(other),
        }
    }
}

impl CheckedEdit for EditGroup {
    type Target = GroupData;

    fn check(&self, target: &Self::Target) -> Result<(), EditError> {
        use EditGroup::*;

        match self {
            SetName { .. } => {}
            AddTag(tag) => {
                if target.tags.contains(tag) {
                    // Adding a present tag will not preserve accurate undo history.
                    return Err(EditError::RedundantEdit("tag already present on group"));
                }
            }
            RemoveTag(tag) => {
                if !target.tags.contains(tag) {
                    // Removing an absent tag will not preserve accurate undo history.
                    return Err(EditError::RedundantEdit("tag not present on group"));
                }
            }
        }

        Ok(())
    }
}

pub enum EditTable {
    SetName { new_name: String, old_name: String },
}

impl undo::Edit for EditTable {
    type Target = TableData;

    type Output = Result<(), EditError>;

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        use EditTable::*;

        match self {
            SetName { new_name, .. } => {
                target.name = new_name.clone();
            }
        }

        Ok(())
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        use EditTable::*;

        match self {
            SetName { old_name, .. } => {
                target.name = old_name.clone();
            }
        }

        Ok(())
    }

    fn merge(&mut self, mut other: Self) -> undo::Merged<Self>
    where
        Self: Sized,
    {
        use EditTable::*;

        match (self, &mut other) {
            (
                SetName {
                    new_name: self_new_name,
                    ..
                },
                SetName {
                    new_name: other_new_name,
                    ..
                },
            ) => {
                swap(other_new_name, self_new_name);
                undo::Merged::Yes
            }
        }
    }
}

impl CheckedEdit for EditTable {
    type Target = TableData;

    fn check(&self, _target: &Self::Target) -> Result<(), EditError> {
        use EditTable::*;

        match self {
            SetName { .. } => Ok(()),
        }
    }
}

fn map_merge<T, U, F>(child: undo::Merged<T>, f: F) -> undo::Merged<U>
where
    F: FnOnce(T) -> U,
{
    use undo::Merged::*;

    match child {
        Yes => Yes,
        No(child_edit) => No(f(child_edit)),
        Annul => Annul,
    }
}
