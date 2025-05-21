use std::collections::HashSet;

use super::{Document, EditError, Table, arena, edit};

pub type GroupToken = arena::TypedToken<Group>;

#[derive(Clone, Default)]
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

    /// Returns a clone of the group's data.
    pub fn get_data(&self) -> Result<GroupData, EditError> {
        let doc_inner = self.doc.get_inner();
        doc_inner
            .state
            .group_arena
            .get_inner(self.token)
            .map_err(EditError::from)
            .cloned()
    }

    pub fn get_name(&self) -> Result<String, EditError> {
        let doc = self.doc.get_inner();
        let group = doc.state.group_arena.get_inner(self.token)?;
        Ok(group.name.clone())
    }

    /// Requests an edit to set the group's name.
    pub fn edit_name(&self, new_name: String) -> Result<(), EditError> {
        let mut doc = self.doc.get_mut_inner();
        let old_name = doc.state.group_arena.get_inner(self.token)?.name.clone();
        doc.apply_edit(edit::EditDocumentState::Group {
            group: self.token,
            edit: edit::EditGroup::SetName { new_name, old_name },
        })?;
        Ok(())
    }

    /// Requests an edit to add a tag.
    pub fn add_tag(&self, tag: String) -> Result<(), EditError> {
        let mut doc = self.doc.get_mut_inner();
        doc.apply_edit(edit::EditDocumentState::Group {
            group: self.token,
            edit: edit::EditGroup::AddTag(tag),
        })
    }

    /// Requests an edit to remove a tag.
    pub fn remove_tag(&self, tag: String) -> Result<(), EditError> {
        let mut doc = self.doc.get_mut_inner();
        doc.apply_edit(edit::EditDocumentState::Group {
            group: self.token,
            edit: edit::EditGroup::RemoveTag(tag),
        })
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
