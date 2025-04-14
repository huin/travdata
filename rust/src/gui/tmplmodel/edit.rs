use std::mem::swap;

use crate::clock;

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

/// An edit that includes a timestamp.
///
/// It will only allow a merge if the timestamps of two [TimestampedEdit]s are within
/// [get_max_merge_edit_time_delta] of each other.
pub struct TimestampedEdit {
    ts: clock::Timestamp,
    edit: Edit,
}

impl TimestampedEdit {
    pub fn new(ts: clock::Timestamp, edit: Edit) -> Self {
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
        match self.edit.merge(other_edit) {
            Yes => Yes,
            No(other_edit) => No(TimestampedEdit {
                ts: other_ts,
                edit: other_edit,
            }),
            Annul => Annul,
        }
    }
}

/// A discrete change to an open [Document].
pub enum Edit {
    SetGroupName {
        group: GroupToken,
        new_name: String,
        old_name: String,
    },
    SetTableName {
        table: TableToken,
        new_name: String,
        old_name: String,
    },
}

impl undo::Edit for Edit {
    type Target = DocumentState;

    type Output = Result<(), EditError>;

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        use Edit::*;

        match &self {
            SetGroupName {
                group,
                new_name,
                old_name: _,
            } => {
                target.allocs.get_mut_group(*group)?.name = new_name.clone();
            }
            SetTableName {
                table,
                new_name,
                old_name: _,
            } => {
                target.allocs.get_mut_table(*table)?.name = new_name.clone();
            }
        }

        Ok(())
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        use Edit::*;

        match &self {
            SetGroupName {
                group,
                new_name: _,
                old_name,
            } => {
                target.allocs.get_mut_group(*group)?.name = old_name.clone();
            }
            SetTableName {
                table,
                new_name: _,
                old_name,
            } => {
                target.allocs.get_mut_table(*table)?.name = old_name.clone();
            }
        }

        Ok(())
    }

    fn merge(&mut self, mut other: Self) -> undo::Merged<Self>
    where
        Self: Sized,
    {
        // TODO: Decide if implementing `merge` is worth the trouble.
        use Edit::*;

        match (self, &mut other) {
            (
                SetTableName {
                    table,
                    new_name,
                    old_name,
                },
                SetTableName {
                    table: new_table,
                    new_name: new_new_name,
                    old_name: _,
                },
            ) if table == new_table && new_new_name != old_name => {
                swap(new_name, new_new_name);
                undo::Merged::Yes
            }
            (
                SetGroupName {
                    group,
                    new_name,
                    old_name,
                },
                SetGroupName {
                    group: new_group,
                    new_name: new_new_name,
                    old_name: _,
                },
            ) if group == new_group && new_new_name != old_name => {
                swap(new_name, new_new_name);
                undo::Merged::Yes
            }
            _ => undo::Merged::No(other),
        }
    }
}
