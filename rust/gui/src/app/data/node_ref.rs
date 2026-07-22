/// Remains the same for the lifetime of a [NodeEditor]. Isn't displayed, but used in maintaining
/// consistent references to other nodes regardless of their [NodeEditor::id] changing.
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    PartialOrd,
    Ord,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct NodeRef(usize);

impl NodeRef {
    pub const MIN: NodeRef = NodeRef(usize::MIN);
    pub const MAX: NodeRef = NodeRef(usize::MAX);

    pub(crate) fn next_and_inc(&mut self) -> Result<Self, String> {
        let next = *self;
        self.0 = self.0.checked_add(1).ok_or("could not allocate NodeRef")?;
        Ok(next)
    }

    #[cfg(test)]
    pub(crate) fn next_and_inc_for_test(&mut self) -> Self {
        self.next_and_inc().expect("unexpectedly failed")
    }
}

impl hashbrown::Equivalent<NodeRef> for &NodeRef {
    fn equivalent(&self, key: &NodeRef) -> bool {
        *self == key
    }
}
