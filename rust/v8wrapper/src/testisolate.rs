use crate::IsolateThreadHandle;

/// Provides a shared [crate::IsolateThreadHandle] for tests. At the time of writing [v8] only supports
/// creating one [v8::Isolate] per process (even after removing the first).
pub struct IsolateThreadHandleForTest {
    handle: IsolateThreadHandle,
}

// Unclear if this is safe to implement, but it's for tests only.
impl std::panic::RefUnwindSafe for IsolateThreadHandleForTest {}

impl std::ops::Deref for IsolateThreadHandleForTest {
    type Target = IsolateThreadHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl googletest::fixtures::StaticFixture for IsolateThreadHandleForTest {
    fn set_up_once() -> googletest::Result<Self> {
        Ok(Self {
            handle: IsolateThreadHandle::new(),
        })
    }
}
