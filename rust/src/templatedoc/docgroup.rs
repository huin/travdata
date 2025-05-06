use std::collections::HashSet;

use super::{Document, EditError, Table, arena, edit};

pub type GroupToken = arena::TypedToken<Group>;

#[derive(Default)]
pub struct GroupData {
    pub name: String,
    pub tags: HashSet<String>,
    pub tables: Vec<Table>,
}

#[derive(Clone)]
pub struct Group {
    doc: Document,
    token: GroupToken,
}

/// Not in the [Group] impl so that it can avoid being re-exported by `templatedoc`.
pub fn new_group(doc: &Document, token: GroupToken) -> Group {
    Group {
        doc: doc.clone(),
        token,
    }
}

impl Group {
    pub fn token(&self) -> GroupToken {
        self.token
    }

    pub fn get_name(&self) -> Result<String, EditError> {
        let doc_inner = self.doc.get_inner();
        let group_data = doc_inner.state.group_arena.get_inner(self.token)?;
        Ok(group_data.name.clone())
    }

    pub fn edit_name(&self, new_name: String) -> Result<(), EditError> {
        let mut doc = self.doc.get_mut_inner();
        let old_name = doc.state.group_arena.get_inner(self.token)?.name.clone();
        doc.apply_edit(edit::EditDocumentState::Group {
            group: self.token,
            edit: edit::EditGroup::SetName { new_name, old_name },
        })?;
        Ok(())
    }
}

impl std::fmt::Debug for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let doc = self.doc.get_inner();
        let name = doc
            .state
            .group_arena
            .get_inner(self.token)
            .map(|group| group.name.as_str())
            .unwrap_or("<unknown>");
        f.debug_struct("Group")
            .field("doc", &self.doc)
            .field("token", &self.token)
            .field("name", &&name)
            .finish()
    }
}
