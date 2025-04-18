// TODO: Remove this allowance.
#![allow(dead_code)]

mod edit;
mod subscribers;
#[cfg(test)]
mod tests;

use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashSet,
    rc::Rc,
};

use crate::{clock, extraction::tableextract, template};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GroupToken(atree::Token);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TableToken(atree::Token);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TablePortionToken(atree::Token);

/// Reference-counted [Document].
#[derive(Clone)]
pub struct DocumentRc(Rc<RefCell<Document>>);

impl DocumentRc {
    pub fn new_with_clock(clock: Rc<dyn clock::Clock>) -> Self {
        DocumentRc(Rc::new(RefCell::new(Document::new(clock))))
    }

    pub fn new() -> Self {
        let clock = clock::RealClock::new();
        DocumentRc(Rc::new(RefCell::new(Document::new(Rc::new(clock)))))
    }

    fn get_doc(&self) -> Ref<Document> {
        self.0.as_ref().borrow()
    }

    fn get_mut_doc(&self) -> RefMut<Document> {
        self.0.as_ref().borrow_mut()
    }

    pub fn undo(&self) -> Result<(), EditError> {
        self.get_mut_doc().undo()
    }

    pub fn redo(&self) -> Result<(), EditError> {
        self.get_mut_doc().redo()
    }

    pub fn get_book(&self) -> Book {
        Book { doc: self.clone() }
    }

    pub fn new_table(&self, table_data: TableData) -> Table {
        let mut doc = self.get_mut_doc();
        let token = doc.state.allocs.new_table(table_data);

        Table {
            doc: self.clone(),
            token,
        }
    }

    pub fn new_group(&self, group_data: GroupData) -> Group {
        let mut doc = self.get_mut_doc();
        let token = doc.state.allocs.new_group(group_data);

        Group {
            doc: self.clone(),
            token,
        }
    }
}

impl std::fmt::Debug for DocumentRc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DocumentRc @ {:?}", self.0.as_ptr())
    }
}

struct Document {
    state: DocumentState,

    clock: Rc<dyn clock::Clock>,

    /// Linear history of edits, acting as an undo/redo history.
    edits: undo::Record<edit::TimestampedEdit, ()>,
}

impl Document {
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
    allocs: DocumentAllocs,
    book: BookData,
}

impl DocumentState {
    fn new() -> Self {
        let mut allocs = DocumentAllocs::new();
        let root_group = allocs.new_group(GroupData {
            name: "root".to_string(),
            tags: HashSet::new(),
            tables: Vec::new(),
        });
        let book = BookData::new(root_group);

        Self { allocs, book }
    }
}

struct DocumentAllocs {
    // TODO: Consider interning strings.
    group_arena: atree::Arena<GroupData>,
    table_arena: atree::Arena<TableData>,
    table_portion_arena: atree::Arena<template::TablePortion>,
}

impl DocumentAllocs {
    fn new() -> Self {
        Self {
            group_arena: atree::Arena::new(),
            table_arena: atree::Arena::new(),
            table_portion_arena: atree::Arena::new(),
        }
    }

    fn get_mut_group(&mut self, token: GroupToken) -> Result<&mut GroupData, EditError> {
        self.group_arena
            .get_mut(token.0)
            .map(|node| &mut node.data)
            .ok_or(EditError::InvalidGroupToken(token))
    }

    fn get_mut_table(&mut self, token: TableToken) -> Result<&mut TableData, EditError> {
        self.table_arena
            .get_mut(token.0)
            .map(|node| &mut node.data)
            .ok_or(EditError::InvalidTableToken(token))
    }

    fn new_group(&mut self, group_data: GroupData) -> GroupToken {
        GroupToken(self.group_arena.new_node(group_data))
    }

    fn new_table(&mut self, table_data: TableData) -> TableToken {
        TableToken(self.table_arena.new_node(table_data))
    }

    fn get_group(&self, token: GroupToken) -> Result<&GroupData, EditError> {
        self.group_arena
            .get(token.0)
            .map(|node| &node.data)
            .ok_or(EditError::InvalidGroupToken(token))
    }

    fn get_table(&self, token: TableToken) -> Result<&TableData, EditError> {
        self.table_arena
            .get(token.0)
            .map(|node| &node.data)
            .ok_or(EditError::InvalidTableToken(token))
    }
}

#[derive(Clone)]
pub struct Book {
    doc: DocumentRc,
}

impl Book {
    pub fn get_root_group(&self) -> Group {
        Group {
            doc: self.doc.clone(),
            token: self.doc.get_doc().state.book.root_group,
        }
    }
}

struct BookData {
    scripts: Vec<template::Script>,
    root_group: GroupToken,
}

impl BookData {
    fn new(root_group: GroupToken) -> Self {
        Self {
            scripts: Vec::new(),
            root_group,
        }
    }
}

impl std::fmt::Debug for BookData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Book").finish()
    }
}

#[derive(Default)]
pub struct GroupData {
    pub name: String,
    pub tags: HashSet<String>,
    pub tables: Vec<TableToken>,
}

#[derive(Clone)]
pub struct Group {
    doc: DocumentRc,
    token: GroupToken,
}

impl Group {
    pub fn token(&self) -> GroupToken {
        self.token
    }

    pub fn get_name(&self) -> Result<String, EditError> {
        let doc = self.doc.get_doc();
        let group = doc.state.allocs.get_group(self.token)?;
        Ok(group.name.clone())
    }

    pub fn edit_name(&self, new_name: String) -> Result<(), EditError> {
        let mut doc = self.doc.get_mut_doc();
        let old_name = doc.state.allocs.get_group(self.token)?.name.clone();
        doc.apply_edit(edit::EditDocumentState::Group {
            group: self.token,
            edit: edit::EditGroup::SetName { new_name, old_name },
        })?;
        Ok(())
    }
}

impl std::fmt::Debug for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let doc = self.doc.get_doc();
        let name = doc
            .state
            .allocs
            .get_group(self.token)
            .map(|group| group.name.as_str())
            .unwrap_or("<unknown>");
        f.debug_struct("Group")
            .field("doc", &self.doc)
            .field("token", &self.token)
            .field("name", &&name)
            .finish()
    }
}

#[derive(Default)]
pub struct TableData {
    pub name: String,
    pub tags: HashSet<String>,
    pub portions: Vec<TablePortionToken>,
    pub transform: tableextract::TableTransform,
}

#[derive(Clone)]
pub struct Table {
    doc: DocumentRc,
    token: TableToken,
}

impl Table {
    pub fn token(&self) -> TableToken {
        self.token
    }

    pub fn get_name(&self) -> Result<String, EditError> {
        let doc = self.doc.get_doc();
        let table = doc.state.allocs.get_table(self.token)?;
        Ok(table.name.clone())
    }

    pub fn edit_name(&self, new_name: String) -> Result<(), EditError> {
        let mut doc = self.doc.get_mut_doc();
        let old_name = doc.state.allocs.get_table(self.token)?.name.clone();
        doc.apply_edit(edit::EditDocumentState::Table {
            table: self.token,
            edit: edit::EditTable::SetName { new_name, old_name },
        })?;
        Ok(())
    }
}

impl std::fmt::Debug for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let doc = self.doc.get_doc();
        let name = doc
            .state
            .allocs
            .get_table(self.token)
            .map(|table| table.name.as_str())
            .unwrap_or("<unknown>");
        f.debug_struct("Table")
            .field("doc", &self.doc)
            .field("token", &self.token)
            .field("name", &&name)
            .finish()
    }
}

pub struct TablePortion<'node>(&'node atree::Node<template::TablePortion>);

impl<'node> TablePortion<'node> {
    pub fn token(&'node self) -> TablePortionToken {
        TablePortionToken(self.0.token())
    }
}

/// Describes an error encountered while applying an edit.
#[derive(Debug, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum EditError {
    NothingToUndo,
    NothingToRedo,
    InvalidGroupToken(GroupToken),
    InvalidTableToken(TableToken),
    InvalidTablePortionToken(TablePortionToken),
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
            InvalidGroupToken(token) => {
                write!(f, "invalid group token: {:?}", token)
            }
            InvalidTableToken(token) => {
                write!(f, "invalid table token: {:?}", token)
            }
            InvalidTablePortionToken(token) => {
                write!(f, "invalid table portion token: {:?}", token)
            }
        }
    }
}

// TODO: One idea for interning strings:
//
// interned_strings: HashSet<Weak<str>>
//
// fn get_or_intern_string<S>(&mut self, s: S)
//   where S: Into<String> + AsRef<str> { todo!() }
