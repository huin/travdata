use std::{collections::HashMap, hash::Hash, marker::PhantomData};

/// A reference token unique to a single listening closure.
pub struct Subscription<E>(ListenerID, PhantomData<E>);

impl<E> Subscription<E> {
    fn new(listener_id: ListenerID) -> Self {
        Self(listener_id, PhantomData)
    }
}

/// Manages subscriptions to events from a single subject.
pub struct SubjectSubscriptions<E> {
    subscriptions: HashMap<ListenerID, BoxedListener<E>>,

    // ID allocation:
    listener_ids: IDPool<ListenerID>,
}

impl<E> SubjectSubscriptions<E> {
    pub fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
            listener_ids: IDPool::new(),
        }
    }

    /// Subscribes to events from the given subject. The returned [Subscription] must be cleaned up
    /// by calling [Self::unsubscribe] later if the listener no longer wants to receive events
    /// relating to the subject.
    ///
    /// This method will panic if too many subscribers have been created. A maximum of [ID::MAX]
    /// can exist at any given time.
    pub fn subscribe(&mut self, listener: impl Fn(&E) + 'static) -> Subscription<E> {
        self.checked_subscribe(listener)
            .expect("ran out of listener IDs")
    }

    /// Like [Self::subscribe], but returns [Option::None] instead of panicing if there are too
    /// many subscribers in existance.
    pub fn checked_subscribe(
        &mut self,
        listener: impl Fn(&E) + 'static,
    ) -> Option<Subscription<E>> {
        let listener_key = self.checked_add_subscription(Box::new(listener))?;
        Some(Subscription::new(listener_key))
    }

    /// Unsubscribes a [Subscription] previously created via [Self::subscribe] or
    /// [Self::checked_subscribe] such that the listener's resources are freed and will no longer
    /// receive events.
    pub fn unsubscribe(&mut self, subscription: &Subscription<E>) {
        self.subscriptions.remove(&subscription.0);
    }

    /// Sends the event to all subscribers of the subject.
    pub fn emit(&self, event: &E) {
        for listener in self.subscriptions.values() {
            listener(event);
        }
    }

    fn has_subscriptions(&self) -> bool {
        !self.subscriptions.is_empty()
    }

    fn checked_add_subscription(&mut self, listener: Box<dyn Fn(&E)>) -> Option<ListenerID> {
        let listener_id = self.listener_ids.checked_take_id()?;
        self.subscriptions.insert(listener_id, listener);
        Some(listener_id)
    }
}

/// A reference token unique to a single listening closure on a single subject.
pub struct SubjectSubscription<S, E> {
    subject: S,
    subscription: Subscription<E>,
}

/// Manages subjects and listeners subscribed to events on those subjects.
///
/// This written with the assumption that there are relatively few subscriptions that exist at any
/// given time.
///
/// Type parameters:
/// - `S` an identifier/reference to the subject. Should be relatively small in memory, cloneable.
/// - `E` event types emitted to listeners.
pub struct MultiSubjectSubscriptions<S, E> {
    subject_subscriptions: HashMap<S, SubjectSubscriptions<E>>,
}

impl<S, E> MultiSubjectSubscriptions<S, E>
where
    S: Clone + Eq + Hash,
{
    /// Creates an empty [Subscriptions].
    pub fn new() -> Self {
        Self {
            subject_subscriptions: HashMap::new(),
        }
    }

    /// This does not notify the subscribers, they simply will not receive any further messages.
    pub fn remove_subject(&mut self, subject: &S) {
        self.subject_subscriptions.remove(subject);
    }

    /// Subscribes to events from the given subject. The returned [Subscription] must be cleaned up
    /// by calling [Self::unsubscribe] later if the listener no longer wants to receive events
    /// relating to the subject.
    ///
    /// This method will panic if too many subscribers have been created. A maximum of [ID::MAX]
    /// can exist at any given time.
    pub fn subscribe(
        &mut self,
        subject: &S,
        listener: impl Fn(&E) + 'static,
    ) -> SubjectSubscription<S, E> {
        self.checked_subscribe(subject, listener)
            .expect("ran out of listener IDs")
    }

    /// Like [Self::subscribe] but returns [Option::None] instead of panicing if there are too many
    /// subscribers in existance.
    pub fn checked_subscribe(
        &mut self,
        subject: &S,
        listener: impl Fn(&E) + 'static,
    ) -> Option<SubjectSubscription<S, E>> {
        let subscription = self
            .subject_subscriptions
            .entry(subject.clone())
            .or_insert_with(SubjectSubscriptions::new)
            .checked_subscribe(listener)?;

        Some(SubjectSubscription {
            subject: subject.clone(),
            subscription,
        })
    }

    /// Unsubscribes a [SubjectSubscription] previously created via [Self::subscribe] or
    /// [Self::checked_subscribe] such that the listener's resources are freed and will no longer
    /// receive events.
    pub fn unsubscribe(&mut self, subscription: &SubjectSubscription<S, E>) {
        // The entry may not exist if the subject was already removed, this is accepted.
        let has_subscriptions = match self.subject_subscriptions.get_mut(&subscription.subject) {
            Some(subject_subscriptions) => {
                subject_subscriptions.unsubscribe(&subscription.subscription);
                subject_subscriptions.has_subscriptions()
            }
            None => {
                return;
            }
        };

        if !has_subscriptions {
            self.subject_subscriptions.remove(&subscription.subject);
        }
    }

    /// Sends the event to all subscribers of the subject.
    pub fn emit(&self, subject: &S, event: &E) {
        if let Some(subscriptions) = self.subject_subscriptions.get(subject) {
            subscriptions.emit(event);
        }
    }
}

type ID = u32;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum Event {
        Foo,
        Bar,
    }

    #[derive(Debug, Eq, PartialEq)]
    struct SubjectEvent {
        subject: char,
        listener: u8,
        event: Event,
    }

    #[derive(Clone)]
    struct ReceivedEventStore {
        events: Rc<RefCell<Vec<SubjectEvent>>>,
    }

    impl ReceivedEventStore {
        fn new() -> Self {
            Self {
                events: Rc::new(RefCell::new(Vec::<SubjectEvent>::new())),
            }
        }

        fn listener(&self, subject: char, listener: u8) -> impl Fn(&Event) + 'static {
            let store = self.clone();
            move |event: &Event| {
                store.events.borrow_mut().push(SubjectEvent {
                    subject,
                    listener,
                    event: event.clone(),
                });
            }
        }

        fn take_events(&self) -> Vec<SubjectEvent> {
            self.events.borrow_mut().drain(..).collect()
        }
    }

    #[gtest]
    fn test_subject_subscriptions_sends_events_to_multiple_listeners() {
        let mut subscribers = SubjectSubscriptions::<Event>::new();
        let events = ReceivedEventStore::new();

        let subject = 'a';

        subscribers.subscribe(events.listener(subject, 1));
        subscribers.subscribe(events.listener(subject, 2));

        subscribers.emit(&Event::Foo);

        expect_that!(
            events.take_events(),
            unordered_elements_are![
                &SubjectEvent {
                    subject,
                    listener: 1,
                    event: Event::Foo
                },
                &SubjectEvent {
                    subject,
                    listener: 2,
                    event: Event::Foo
                },
            ],
        );
    }

    #[gtest]
    fn test_subject_subscriptions_stops_sending_when_unsubscribed() {
        let mut subscribers = SubjectSubscriptions::<Event>::new();
        let events = ReceivedEventStore::new();

        let subject = 'a';

        subscribers.subscribe(events.listener(subject, 1));
        let subscription_2 = subscribers.subscribe(events.listener(subject, 2));

        subscribers.unsubscribe(&subscription_2);

        subscribers.emit(&Event::Foo);

        expect_that!(
            events.take_events(),
            elements_are![&SubjectEvent {
                subject,
                listener: 1,
                event: Event::Foo
            }],
        );
    }

    #[gtest]
    fn test_multi_subject_subscriptions_sends_events() {
        let subject_a = 'a';
        let subject_b = 'b';

        let mut subscribers = MultiSubjectSubscriptions::<char, Event>::new();
        let events = ReceivedEventStore::new();

        let subscription_a_1 = subscribers.subscribe(&subject_a, events.listener(subject_a, 1));
        let subscription_b_2 = subscribers.subscribe(&subject_b, events.listener(subject_b, 2));

        subscribers.emit(&subject_a, &Event::Foo);
        expect_that!(
            events.take_events(),
            elements_are![eq(&SubjectEvent {
                subject: subject_a,
                listener: 1,
                event: Event::Foo,
            }),],
        );

        subscribers.emit(&subject_b, &Event::Bar);
        subscribers.emit(&subject_b, &Event::Foo);
        expect_that!(
            events.take_events(),
            elements_are![
                eq(&SubjectEvent {
                    subject: subject_b,
                    listener: 2,
                    event: Event::Bar,
                }),
                eq(&SubjectEvent {
                    subject: subject_b,
                    listener: 2,
                    event: Event::Foo,
                }),
            ],
        );

        subscribers.unsubscribe(&subscription_b_2);
        subscribers.emit(&subject_b, &Event::Foo);
        expect_that!(events.take_events(), empty());

        subscribers.unsubscribe(&subscription_a_1);
        subscribers.emit(&subject_a, &Event::Foo);
        expect_that!(events.take_events(), empty());

        subscribers.remove_subject(&subject_a);
        subscribers.remove_subject(&subject_b);
    }
}
