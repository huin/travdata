/// Remains the same for the lifetime of a [NodeEditor]. Isn't displayed, but used in maintaining
/// consistent references to other nodes regardless of their [NodeEditor::id] changing.
#[derive(
    Copy, Clone, Debug, Default, Eq, PartialEq, Hash, serde::Deserialize, serde::Serialize,
)]
pub struct NodeRef(usize);

impl NodeRef {
    pub(crate) fn next_and_inc(&mut self) -> Result<Self, String> {
        let next = *self;
        self.0 = self.0.checked_add(1).ok_or("could not allocate NodeRef")?;
        Ok(next)
    }
}
