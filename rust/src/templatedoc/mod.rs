// TODO: Remove this allowance.
#![allow(dead_code)]

mod arena;
mod docgroup;
mod doctable;
mod doctableportion;
mod edit;
pub mod event;
#[cfg(test)]
mod tests;

use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashSet,
    rc::Rc,
};

use crate::{clock, template};
use arena::ArenaError;
pub use docgroup::{Group, GroupData};
pub use doctable::{Table, TableData};
pub use doctableportion::{TablePortion, TablePortionData};

/// Reference-counted [Document].
#[derive(Clone)]
pub struct Document(Rc<RefCell<DocumentInner>>);

impl Document {
    pub fn new_with_clock(clock: Rc<dyn clock::Clock>) -> Self {
        Document(Rc::new(RefCell::new(DocumentInner::new(clock))))
    }

    pub fn new() -> Self {
        let clock = clock::RealClock::new();
        Document(Rc::new(RefCell::new(DocumentInner::new(Rc::new(clock)))))
    }

    fn get_inner(&self) -> Ref<DocumentInner> {
        self.0.as_ref().borrow()
    }

    fn get_mut_inner(&self) -> RefMut<DocumentInner> {
        self.0.as_ref().borrow_mut()
    }

    pub fn undo(&self) -> Result<(), EditError> {
        self.get_mut_inner().undo()
    }

    pub fn redo(&self) -> Result<(), EditError> {
        self.get_mut_inner().redo()
    }

    pub fn get_book(&self) -> Book {
        Book { doc: self.clone() }
    }

    pub fn new_table(&self, table_data: TableData) -> Table {
        let token = self.get_mut_inner().state.table_arena.new_inner(table_data);
        doctable::new_table(self, token)
    }

    pub fn new_group(&self, group_data: GroupData) -> Group {
        let token = self.get_mut_inner().state.group_arena.new_inner(group_data);
        docgroup::new_group(self, token)
    }
}

impl std::fmt::Debug for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DocumentRc @ {:?}", self.0.as_ptr())
    }
}

struct DocumentInner {
    state: DocumentState,

    clock: Rc<dyn clock::Clock>,

    /// Linear history of edits, acting as an undo/redo history.
    edits: undo::Record<edit::TimestampedEdit, ()>,
}

impl DocumentInner {
    fn new(clock: Rc<dyn clock::Clock>) -> Self {
        Self {
            state: DocumentState::new(),
            clock,
            edits: Default::default(),
        }
    }

    fn apply_edit(&mut self, edit: edit::EditDocumentState) -> Result<(), EditError> {
        let ts_edit = edit::TimestampedEdit::new(self.clock.as_ref().now(), edit);
        self.edits.edit(&mut self.state, ts_edit)
    }

    fn undo(&mut self) -> Result<(), EditError> {
        self.edits
            .undo(&mut self.state)
            .ok_or(EditError::NothingToUndo)?
    }

    fn redo(&mut self) -> Result<(), EditError> {
        self.edits
            .redo(&mut self.state)
            .ok_or(EditError::NothingToRedo)?
    }
}

pub struct DocumentState {
    group_arena: arena::TypedArena<Group, GroupData>,
    table_arena: arena::TypedArena<Table, TableData>,
    table_portion_arena: arena::TypedArena<TablePortion, TablePortionData>,
    book: BookData,
}

impl DocumentState {
    fn new() -> Self {
        let mut group_arena = arena::TypedArena::new();
        let root_group_token = group_arena.new_inner(GroupData {
            name: "root".to_string(),
            tags: HashSet::new(),
            tables: Vec::new(),
        });

        Self {
            group_arena,
            table_arena: arena::TypedArena::new(),
            table_portion_arena: arena::TypedArena::new(),
            book: BookData::new(root_group_token),
        }
    }
}

#[derive(Clone)]
pub struct Book {
    doc: Document,
}

impl Book {
    pub fn get_root_group(&self) -> Group {
        let token = self.doc.get_inner().state.book.root_group_token;
        docgroup::new_group(&self.doc, token)
    }
}

struct BookData {
    scripts: Vec<template::Script>,
    root_group_token: docgroup::GroupToken,
}

impl BookData {
    fn new(root_group: docgroup::GroupToken) -> Self {
        Self {
            scripts: Vec::new(),
            root_group_token: root_group,
        }
    }
}

impl std::fmt::Debug for BookData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Book").finish()
    }
}

/// Describes an error encountered while applying an edit.
#[derive(Debug, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum EditError {
    NothingToUndo,
    NothingToRedo,
    ArenaError(ArenaError),
}

impl std::error::Error for EditError {}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use EditError::*;
        match self {
            NothingToUndo => {
                write!(f, "no actions to undo")
            }
            NothingToRedo => {
                write!(f, "no actions to redo")
            }
            ArenaError(error) => {
                write!(f, "arena error: {}", error)
            }
        }
    }
}

impl From<ArenaError> for EditError {
    fn from(value: ArenaError) -> Self {
        EditError::ArenaError(value)
    }
}

// TODO: One idea for interning strings:
//
// interned_strings: HashSet<Weak<str>>
//
// fn get_or_intern_string<S>(&mut self, s: S)
//   where S: Into<String> + AsRef<str> { todo!() }
