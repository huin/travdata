use std::{collections::BTreeMap, marker::PhantomData, ops::RangeInclusive};

/// A reference token unique to a listenable subject. It cannot be cloned or copied so that
/// multiple copies are not made and there can be confidence that a value will not be reused
/// externally to the [Listeners] that issued it.
pub struct SubjectHandle<E>(SubjectID, PhantomData<E>);

impl<E> SubjectHandle<E> {
    fn new(id: SubjectID) -> Self {
        Self(id, PhantomData)
    }
}

/// A reference token unique to a single listening closure on a single subject.  It cannot be
/// cloned or copied so that multiple copies are not made and there can be confidence that a value
/// will not be reused externally to the [Listeners] that issued it.
pub struct Subscription<E>(SubscriptionKey, PhantomData<E>);

impl<E> Subscription<E> {
    fn new(key: SubscriptionKey) -> Self {
        Self(key, PhantomData)
    }
}

/// Manages subjects and listeners subscribed to events on those subjects.
pub struct Subscriptions<E> {
    subscriptions: BTreeMap<SubscriptionKey, BoxedListener<E>>,

    // ID allocation:
    subject_ids: IDPool<SubjectID>,
    listener_ids: IDPool<ListenerID>,
}

impl<E> Subscriptions<E> {
    /// Creates an empty [Subscriptions].
    pub fn new() -> Self {
        Self {
            subscriptions: BTreeMap::new(),

            subject_ids: IDPool::new(),
            listener_ids: IDPool::new(),
        }
    }

    /// Creates a new subject that can be listened to for events.
    ///
    /// This method will panic if too many subjects have been created. A maximum of [ID::MAX] can
    /// exist at any given time.
    pub fn new_subject(&mut self) -> SubjectHandle<E> {
        self.checked_new_subject().expect("ran out of subject IDs")
    }

    /// Like [Self::new_subject] but returns [Option::None] instead of panicing if there are too
    /// many subjects in existance.
    pub fn checked_new_subject(&mut self) -> Option<SubjectHandle<E>> {
        self.subject_ids.checked_take_id().map(SubjectHandle::new)
    }

    /// Deletes a subject previously created though [Self::checked_new_subject].
    ///
    /// This does not notify the subscribers, they simply will not receive any further messages,
    /// and should [Self::unsubscribe] separately (before or after the subject is deleted) to fully
    /// free up their subscription.
    pub fn delete_subject(&mut self, subject: SubjectHandle<E>) {
        let subject_id = subject.0;

        // Remove all corresponding entries from `subscribers`.
        //
        // Annoyingly there isn't a method to remove a range efficiently. The `Cursor` feature in
        // nightly should be at least somewhat better than what is done here (when it lands in
        // stable).
        let listener_ids: Vec<ListenerID> = self
            .subscriptions
            .range(SubscriptionKey::range_bounds_for_subject(subject_id))
            .map(|(key, _)| key.listener_id)
            .collect();
        for listener_id in listener_ids {
            self.subscriptions.remove(&SubscriptionKey {
                subject_id,
                listener_id,
            });
        }

        self.subject_ids.free_id(subject_id);

        // The listener IDs cannot yet be returned to `self.listener_ids` as the [ListenerHandle]
        // may still be held elsewhere and we don't want return it to the freelist until it has
        // been voluntarily returned, lest it become duplicated.
    }

    /// Subscribes to events from the given subject. The returned [Subscription] must be cleaned up
    /// by calling [Self::unsubscribe] later, even if the subject has been deleted through
    /// [Self::delete_subject].
    ///
    /// This method will panic if too many subscribers have been created. A maximum of [ID::MAX]
    /// can exist at any given time.
    pub fn subscribe(
        &mut self,
        subject: &SubjectHandle<E>,
        listener: impl Fn(&E) + 'static,
    ) -> Subscription<E> {
        self.checked_subscribe(subject, listener)
            .expect("ran out of subscriber IDs")
    }

    /// Like [Self::subscribe] but returns [Option::None] instead of panicing if there are too many
    /// subscribers in existance.
    pub fn checked_subscribe(
        &mut self,
        subject: &SubjectHandle<E>,
        listener: impl Fn(&E) + 'static,
    ) -> Option<Subscription<E>> {
        let listener_key = self.checked_add_subscription(subject.0, Box::new(listener))?;
        Some(Subscription::new(listener_key))
    }

    /// Deallocates a [Subscription] previously created via [Self::subscribe] or
    /// [Self::checked_subscribe].
    pub fn unsubscribe(&mut self, handle: Subscription<E>) {
        // The entry may not exist if the subject was already removed, this is accepted.
        self.subscriptions.remove(&handle.0);
    }

    /// Sends the event to all subscribers to the subject.
    pub fn emit(&self, subject: &SubjectHandle<E>, event: &E) {
        for (_, subscription) in self
            .subscriptions
            .range(SubscriptionKey::range_bounds_for_subject(subject.0))
        {
            subscription(event);
        }
    }

    fn checked_add_subscription(
        &mut self,
        subject_id: SubjectID,
        listener: Box<dyn Fn(&E)>,
    ) -> Option<SubscriptionKey> {
        let listener_id = self.listener_ids.checked_take_id()?;
        let key = SubscriptionKey {
            subject_id,
            listener_id,
        };
        self.subscriptions.insert(key, listener);
        Some(key)
    }
}

type ID = u32;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SubjectID(ID);

impl SubjectID {
    fn min() -> Self {
        Self(ID::MIN)
    }

    fn max() -> Self {
        Self(ID::MAX)
    }
}

impl IDTrait for SubjectID {
    fn first() -> Self {
        Self(ID::MIN)
    }

    fn checked_next(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ListenerID(ID);

impl ListenerID {
    fn min() -> Self {
        Self(ID::MIN)
    }

    fn max() -> Self {
        Self(ID::MAX)
    }
}

impl IDTrait for ListenerID {
    fn first() -> Self {
        Self(ID::MIN)
    }

    fn checked_next(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SubscriptionKey {
    // The ordering of fields is significant for the `Ord` of this structure. Keys with the same
    // [SubjectID] must be grouped together.
    subject_id: SubjectID,
    listener_id: ListenerID,
}

impl SubscriptionKey {
    /// Returns a [RangeInclusive] that includes all possible [ListenerKey]s for the given
    /// [SubjectID].
    fn range_bounds_for_subject(subject_id: SubjectID) -> RangeInclusive<Self> {
        Self {
            subject_id,
            listener_id: ListenerID::min(),
        }..=Self {
            subject_id,
            listener_id: ListenerID::max(),
        }
    }
}

trait IDTrait: Copy {
    fn first() -> Self;
    fn checked_next(self) -> Option<Self>;
}

struct IDPool<T> {
    freelist: Vec<T>,
    next: T,
}

impl<T> IDPool<T>
where
    T: IDTrait,
{
    fn new() -> Self {
        Self {
            freelist: Vec::new(),
            next: T::first(),
        }
    }

    fn checked_take_id(&mut self) -> Option<T> {
        match self.freelist.pop() {
            Some(id) => Some(id),
            None => {
                let id = self.next;
                self.next = id.checked_next()?;
                Some(id)
            }
        }
    }

    fn free_id(&mut self, id: T) {
        self.freelist.push(id);
    }
}

/// A boxed closure for listening to events from subjects.
type BoxedListener<E> = Box<dyn Fn(&E)>;

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use googletest::prelude::*;

    use super::*;

    #[gtest]
    fn test_listener_key_ord() {
        let subject_a = SubjectID(5);
        let subject_b = SubjectID(8);
        let listener_a = ListenerID(3);
        let listener_b = ListenerID(6);

        expect_lt!(
            &SubscriptionKey {
                subject_id: subject_a,
                listener_id: listener_a,
            },
            &SubscriptionKey {
                subject_id: subject_b,
                listener_id: listener_a,
            },
        );
        expect_lt!(
            &SubscriptionKey {
                subject_id: subject_a,
                listener_id: listener_a,
            },
            &SubscriptionKey {
                subject_id: subject_a,
                listener_id: listener_b,
            },
        );
        expect_eq!(
            &SubscriptionKey {
                subject_id: subject_a,
                listener_id: listener_a,
            },
            &SubscriptionKey {
                subject_id: subject_a,
                listener_id: listener_a,
            },
        );
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum Event {
        Foo,
        Bar,
    }
    #[derive(Debug, Eq, PartialEq)]
    struct SubjectEvent {
        subject: u8,
        event: Event,
    }

    #[gtest]
    fn test_subscription_receives_events() {
        let mut subscribers = Subscriptions::<Event>::new();

        let subject_a = subscribers.new_subject();
        let sub_a: u8 = 1;
        let subject_b = subscribers.new_subject();
        let sub_b: u8 = 2;

        let events = Rc::new(RefCell::new(Vec::<SubjectEvent>::new()));

        let subscription_a = {
            let events_rc = events.clone();
            subscribers.subscribe(&subject_a, move |event| {
                events_rc.borrow_mut().push(SubjectEvent {
                    subject: sub_a,
                    event: event.clone(),
                });
            })
        };
        let subscription_b = {
            let events_rc = events.clone();
            subscribers.subscribe(&subject_b, move |event| {
                events_rc.borrow_mut().push(SubjectEvent {
                    subject: sub_b,
                    event: event.clone(),
                });
            })
        };

        subscribers.emit(&subject_a, &Event::Foo);
        subscribers.emit(&subject_b, &Event::Bar);
        subscribers.emit(&subject_b, &Event::Foo);

        expect_that!(
            *events.borrow(),
            eq(&vec![
                SubjectEvent {
                    subject: sub_a,
                    event: Event::Foo,
                },
                SubjectEvent {
                    subject: sub_b,
                    event: Event::Bar,
                },
                SubjectEvent {
                    subject: sub_b,
                    event: Event::Foo,
                },
            ]),
        );

        subscribers.unsubscribe(subscription_b);
        subscribers.emit(&subject_b, &Event::Foo);

        expect_that!(events.borrow().len(), eq(3));

        subscribers.unsubscribe(subscription_a);
        subscribers.emit(&subject_a, &Event::Foo);
        expect_that!(events.borrow().len(), eq(3));

        subscribers.delete_subject(subject_a);
        subscribers.delete_subject(subject_b);
    }
}
