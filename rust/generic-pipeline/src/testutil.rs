#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct FakeNodeId(pub &'static str);

impl From<&'static str> for FakeNodeId {
    fn from(value: &'static str) -> Self {
        Self(value)
    }
}
