use std::collections::HashSet;

use crate::extraction::tableextract;

use super::{Document, EditError, TablePortion, arena, edit};

pub type TableToken = arena::TypedToken<Table>;

#[derive(Default)]
pub struct TableData {
    pub name: String,
    pub tags: HashSet<String>,
    pub portions: Vec<TablePortion>,
    pub transform: tableextract::TableTransform,
}

#[derive(Clone)]
pub struct Table {
    doc: Document,
    token: TableToken,
}

/// Not in the [Table] impl so that it can avoid being re-exported by `templatedoc`.
pub fn new_table(doc: &Document, token: TableToken) -> Table {
    Table {
        doc: doc.clone(),
        token,
    }
}

impl Table {
    pub fn token(&self) -> TableToken {
        self.token
    }

    pub fn get_name(&self) -> Result<String, EditError> {
        let doc = self.doc.get_inner();
        let table = doc.state.table_arena.get_inner(self.token)?;
        Ok(table.name.clone())
    }

    pub fn edit_name(&self, new_name: String) -> Result<(), EditError> {
        let mut doc = self.doc.get_mut_inner();
        let old_name = doc.state.table_arena.get_inner(self.token)?.name.clone();
        doc.apply_edit(edit::EditDocumentState::Table {
            table: self.token,
            edit: edit::EditTable::SetName { new_name, old_name },
        })?;
        Ok(())
    }
}

impl std::fmt::Debug for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let doc = self.doc.get_inner();
        let name = doc
            .state
            .table_arena
            .get_inner(self.token)
            .map(|table| table.name.as_str())
            .unwrap_or("<unknown>");
        f.debug_struct("Table")
            .field("doc", &self.doc)
            .field("token", &self.token)
            .field("name", &&name)
            .finish()
    }
}
