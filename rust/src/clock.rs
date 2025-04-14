use std::fmt::Debug;

use chrono::prelude::*;

pub type Timestamp = DateTime<Utc>;

pub trait Clock: Debug {
    fn now(&self) -> Timestamp;
}

#[derive(Debug, Default)]
pub struct RealClock;

impl RealClock {
    pub fn new() -> Self {
        Self
    }
}

impl Clock for RealClock {
    fn now(&self) -> Timestamp {
        Utc::now()
    }
}

#[cfg(test)]
pub use fake::*;

#[cfg(test)]
mod fake {
    use std::cell::RefCell;
    use std::fmt::Debug;
    use std::rc::Rc;

    use chrono::TimeDelta;

    use super::{Clock, Timestamp};

    /// Reference counted fake clock. Clone creates a new reference.
    #[derive(Clone, Debug)]
    pub struct FakeClock(Rc<RefCell<Timestamp>>);

    impl FakeClock {
        pub fn new(start: Timestamp) -> Self {
            Self(Rc::new(RefCell::new(start)))
        }

        pub fn advance(&mut self, duration: TimeDelta) {
            let mut now_ref = self.0.borrow_mut();
            let new_now = *now_ref + duration;
            *now_ref = new_now;
        }
    }

    impl Clock for FakeClock {
        fn now(&self) -> Timestamp {
            *self.0.borrow()
        }
    }
}
