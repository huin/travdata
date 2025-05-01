/// Events relating to a [crate::gui:tmplmodel::DocumentRc] as a whole.
pub enum DocumentEvent {
    /// Undoing an edit is or is not possible.
    UndoAvailable(bool),
    /// Redoing an edit is or is not possible.
    RedoAvailable(bool),
}

/// Events relating to a specific [crate::gui::tmplmodel::Group].
pub enum GroupEvent {
    /// At least one property of the group has changed.
    Updated,
    /// Group has been removed from the group hierarchy. This similar to a delete from a user
    /// perspective, but could be reverted by an undo operation.
    Unlinked,
}
