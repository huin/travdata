use std::{
    cell::RefCell,
    collections::HashMap,
    hash::Hash,
    rc::{Rc, Weak},
};

/// A reference token unique to a single listening closure.
pub struct Subscription<E> {
    state: Weak<RefCell<SubjectSubscriptionsState<E>>>,
    id: ListenerID,
}

impl<E> Subscription<E> {
    fn new(subscriptions: &SubjectSubscriptions<E>, id: ListenerID) -> Self {
        Self {
            state: Rc::downgrade(&subscriptions.state),
            id,
        }
    }

    /// Unsubscribes the [Subscription] previously created via [SubjectSubscriptions::subscribe] or
    /// [SubjectSubscriptions::checked_subscribe] such that the listener's resources are freed and
    /// will no longer receive events.
    pub fn unsubscribe(self) {
        // Implicitly lean on [Drop::drop].
    }

    /// Temporarily stops events propagating to the subscription. Any events emitted until the
    /// subscription is unpaused using [Self::unblock] will never be sent to the given
    /// subscription.
    pub fn block(&self) {
        if let Some(state) = self.state.upgrade() {
            state.borrow_mut().block_subscription(self.id);
        }
    }

    /// Resumes events propagating to the given subscription after a call to [Self::block].
    pub fn unblock(&self) {
        if let Some(state) = self.state.upgrade() {
            state.borrow_mut().unblock_subscription(self.id);
        }
    }
}

impl<E> Drop for Subscription<E> {
    fn drop(&mut self) {
        if let Some(state) = self.state.upgrade() {
            let listener = state.borrow_mut().remove_subscription(self.id);
            // Dropped outside of the borrow_mut statement in case the listener's [Drop] impl
            // itself results in a similar `state.borrow*` of the same `state`.
            drop(listener);
        }
    }
}

/// Manages subscriptions to events from a single subject.
pub struct SubjectSubscriptions<E> {
    state: Rc<RefCell<SubjectSubscriptionsState<E>>>,
}

impl<E> SubjectSubscriptions<E> {
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(SubjectSubscriptionsState::new())),
        }
    }

    /// Subscribes to events from the given subject. The returned [Subscription] must be cleaned up
    /// by calling [Self::unsubscribe] later if the listener no longer wants to receive events
    /// relating to the subject.
    ///
    /// This method will panic if too many subscribers have been created. A maximum of [ID::MAX]
    /// can exist at any given time.
    pub fn subscribe(&self, listener: impl Fn(&E) + 'static) -> Subscription<E> {
        self.checked_subscribe(listener)
            .expect("ran out of listener IDs")
    }

    /// Like [Self::subscribe], but returns [Option::None] instead of panicing if there are too
    /// many subscribers in existance.
    pub fn checked_subscribe(&self, listener: impl Fn(&E) + 'static) -> Option<Subscription<E>> {
        let id = self.state.borrow_mut().checked_subscribe(listener)?;
        Some(Subscription::new(self, id))
    }

    /// Sends the event to all subscribers of the subject.
    pub fn emit(&self, event: &E) {
        let snapshot = self.state.borrow().snapshot_for_emit();
        for listener in snapshot {
            listener(event);
        }
    }

    fn has_subscriptions(&self) -> bool {
        self.state.borrow().has_subscriptions()
    }
}

struct SubjectSubscriptionsState<E> {
    subscriptions: HashMap<ListenerID, ListenerSubscription<E>>,

    // ID allocation:
    listener_ids: IDPool<ListenerID>,
}

impl<E> SubjectSubscriptionsState<E> {
    fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
            listener_ids: IDPool::new(),
        }
    }

    fn checked_subscribe(&mut self, listener: impl Fn(&E) + 'static) -> Option<ListenerID> {
        self.checked_add_subscription(ListenerSubscription::new(listener))
    }

    /// Returns the unsubscribed listener so that it can be dropped outside of the caller's
    /// [std::cell::RefMut], in case the `Drop` impl also performs the same borrow.
    fn remove_subscription(&mut self, id: ListenerID) -> Option<ListenerSubscription<E>> {
        self.subscriptions.remove(&id)
    }

    fn block_subscription(&mut self, listener_id: ListenerID) {
        if let Some(sub) = self.subscriptions.get_mut(&listener_id) {
            sub.blocked = true;
        }
    }

    fn unblock_subscription(&mut self, listener_id: ListenerID) {
        if let Some(sub) = self.subscriptions.get_mut(&listener_id) {
            sub.blocked = false;
        }
    }

    fn snapshot_for_emit(&self) -> Vec<ListenerRc<E>> {
        self.subscriptions
            .values()
            .filter(|subscription| !subscription.blocked)
            .map(|subscription| subscription.listener.clone())
            .collect()
    }

    fn has_subscriptions(&self) -> bool {
        !self.subscriptions.is_empty()
    }

    fn checked_add_subscription(
        &mut self,
        subscription: ListenerSubscription<E>,
    ) -> Option<ListenerID> {
        let listener_id = self.listener_ids.checked_take_id()?;
        self.subscriptions.insert(listener_id, subscription);
        Some(listener_id)
    }
}

/// A reference token unique to a single listening closure on a single subject.
pub struct SubjectSubscription<S, E>
where
    S: Clone + Eq + Hash,
{
    state: Weak<RefCell<MultiSubjectSubscriptionsState<S, E>>>,
    subject: S,
    id: ListenerID,
}

impl<S, E> SubjectSubscription<S, E>
where
    S: Clone + Eq + Hash,
{
    fn new(subscriptions: &MultiSubjectSubscriptions<S, E>, subject: &S, id: ListenerID) -> Self {
        Self {
            state: Rc::downgrade(&subscriptions.state),
            subject: subject.clone(),
            id,
        }
    }

    /// Unsubscribes a [SubjectSubscription] previously created via
    /// [MultiSubjectSubscriptions::subscribe] or [MultiSubjectSubscriptions::checked_subscribe]
    /// such that the listener's resources are freed and will no longer receive events.
    pub fn unsubscribe(self) {
        // Implicitly lean on [Drop::drop].
    }

    /// Temporarily stops events propagating to the given subscription. Any events emitted until
    /// the subscription is unpaused using [Self::unblock_subscription] will never be sent to this
    /// subscription.
    pub fn block_subscription(&self) {
        if let Some(state) = self.state.upgrade() {
            state
                .borrow_mut()
                .block_subscription(&self.subject, self.id);
        }
    }

    /// Resumes events propagating to the given subscription after a call to
    /// [Self::block_subscription].
    pub fn unblock_subscription(&self) {
        if let Some(state) = self.state.upgrade() {
            state
                .borrow_mut()
                .unblock_subscription(&self.subject, self.id);
        }
    }
}

impl<S, E> Drop for SubjectSubscription<S, E>
where
    S: Clone + Eq + Hash,
{
    fn drop(&mut self) {
        if let Some(state) = self.state.upgrade() {
            let listener = state
                .borrow_mut()
                .remove_subscription(&self.subject, self.id);

            // Dropped outside of the borrow_mut statement in case the listener's [Drop] impl
            // itself results in a similar `state.borrow*` of the same `state`.
            drop(listener);
        }
    }
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
    state: Rc<RefCell<MultiSubjectSubscriptionsState<S, E>>>,
}

impl<S, E> MultiSubjectSubscriptions<S, E>
where
    S: Clone + Eq + Hash,
{
    /// Creates an empty [MultiSubjectSubscriptions].
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(MultiSubjectSubscriptionsState::new())),
        }
    }

    /// This does not notify the subscribers, they simply will not receive any further messages.
    pub fn remove_subject(&self, subject: &S) {
        let subscriptions = self.state.borrow_mut().remove_subject(subject);
        // Dropped outside of the borrow_mut statement in case a listener's [Drop] impl itself
        // results in a similar `self.state.borrow*`.
        drop(subscriptions);
    }

    /// Subscribes to events from the given subject. The returned [Subscription] must be cleaned up
    /// by calling [Self::unsubscribe] later if the listener no longer wants to receive events
    /// relating to the subject.
    ///
    /// This method will panic if too many subscribers have been created. A maximum of [ID::MAX]
    /// can exist at any given time.
    pub fn subscribe(
        &self,
        subject: &S,
        listener: impl Fn(&E) + 'static,
    ) -> SubjectSubscription<S, E> {
        self.checked_subscribe(subject, listener)
            .expect("ran out of listener IDs")
    }

    /// Like [Self::subscribe] but returns [Option::None] instead of panicing if there are too many
    /// subscribers in existance.
    pub fn checked_subscribe(
        &self,
        subject: &S,
        listener: impl Fn(&E) + 'static,
    ) -> Option<SubjectSubscription<S, E>> {
        let listener_subscription = ListenerSubscription::new(listener);
        let id = self
            .state
            .borrow_mut()
            .checked_subscribe(subject, listener_subscription)?;

        Some(SubjectSubscription::new(self, subject, id))
    }

    /// Sends the event to all subscribers of the subject.
    pub fn emit(&self, subject: &S, event: &E) {
        // Capture snapshot here to avoid borrow()ing for too long.
        let snapshot = self.state.borrow().snapshot_for_emit(subject);

        if let Some(snapshot) = snapshot {
            for listener in snapshot {
                listener(event);
            }
        }
    }
}

struct MultiSubjectSubscriptionsState<S, E> {
    subject_subscriptions: HashMap<S, SubjectSubscriptions<E>>,
}

impl<S, E> MultiSubjectSubscriptionsState<S, E>
where
    S: Clone + Eq + Hash,
{
    fn new() -> Self {
        Self {
            subject_subscriptions: HashMap::new(),
        }
    }

    fn remove_subject(&mut self, subject: &S) -> Option<SubjectSubscriptions<E>> {
        self.subject_subscriptions.remove(subject)
    }

    fn checked_subscribe(
        &mut self,
        subject: &S,
        listener_subscription: ListenerSubscription<E>,
    ) -> Option<ListenerID> {
        let subscriptions = self
            .subject_subscriptions
            .entry(subject.clone())
            .or_insert_with(SubjectSubscriptions::new);

        subscriptions
            .state
            .borrow_mut()
            .checked_add_subscription(listener_subscription)
    }

    /// Returns the unsubscribed listener so that it can be dropped outside of the caller's
    /// [std::cell::RefMut], in case the `Drop` impl also performs the same borrow.
    fn remove_subscription(
        &mut self,
        subject: &S,
        listener_id: ListenerID,
    ) -> Option<ListenerSubscription<E>> {
        // The subscription entry not being present indicates that the entire subject was removed,
        // and therefore so was the requested subscription.
        let subscriptions = self.subject_subscriptions.get(subject)?;

        let subscription = subscriptions
            .state
            .borrow_mut()
            .remove_subscription(listener_id);
        if !subscriptions.has_subscriptions() {
            self.subject_subscriptions.remove(subject);
        }

        subscription
    }

    fn block_subscription(&mut self, subject: &S, listener_id: ListenerID) {
        if let Some(subscriptions) = self.subject_subscriptions.get_mut(subject) {
            subscriptions
                .state
                .borrow_mut()
                .block_subscription(listener_id);
        }
    }

    fn unblock_subscription(&mut self, subject: &S, listener_id: ListenerID) {
        if let Some(subscriptions) = self.subject_subscriptions.get_mut(subject) {
            subscriptions
                .state
                .borrow_mut()
                .unblock_subscription(listener_id);
        }
    }

    fn snapshot_for_emit(&self, subject: &S) -> Option<Vec<ListenerRc<E>>> {
        self.subject_subscriptions
            .get(subject)
            .map(|subscriptions| subscriptions.state.borrow().snapshot_for_emit())
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

type ListenerRc<E> = Rc<dyn Fn(&E)>;

/// A boxed closure for listening to events from subjects.
struct ListenerSubscription<E> {
    listener: ListenerRc<E>,
    blocked: bool,
}

impl<E> ListenerSubscription<E> {
    fn new(listener: impl Fn(&E) + 'static) -> Self {
        Self {
            listener: Rc::new(listener),
            blocked: false,
        }
    }
}

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
        subject: &'static str,
        listener: &'static str,
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

        fn listener(
            &self,
            subject: &'static str,
            listener: &'static str,
        ) -> impl Fn(&Event) + 'static {
            let store = self.clone();
            move |event: &Event| {
                store.record(subject, listener, event);
            }
        }

        fn record(&self, subject: &'static str, listener: &'static str, event: &Event) {
            self.events.as_ref().borrow_mut().push(SubjectEvent {
                subject,
                listener,
                event: event.clone(),
            });
        }

        fn take_events(&self) -> Vec<SubjectEvent> {
            self.events.as_ref().borrow_mut().drain(..).collect()
        }
    }

    #[gtest]
    fn test_subject_subscriptions_sends_events_to_multiple_listeners() {
        let subscribers = SubjectSubscriptions::<Event>::new();
        let events = ReceivedEventStore::new();

        let subject = "a";

        let subscription_1 = subscribers.subscribe(events.listener(subject, "1"));
        let subscription_2 = subscribers.subscribe(events.listener(subject, "2"));

        subscribers.emit(&Event::Foo);

        expect_that!(
            events.take_events(),
            unordered_elements_are![
                &SubjectEvent {
                    subject,
                    listener: "1",
                    event: Event::Foo
                },
                &SubjectEvent {
                    subject,
                    listener: "2",
                    event: Event::Foo
                },
            ],
        );

        subscription_1.unsubscribe();
        subscription_2.unsubscribe();
    }

    #[gtest]
    fn test_subject_subscriptions_stops_sending_when_unsubscribed() {
        let subject = "a";
        let subscribers = SubjectSubscriptions::<Event>::new();
        let events = ReceivedEventStore::new();

        let subscription_1 = subscribers.subscribe(events.listener(subject, "1"));
        let subscription_2 = subscribers.subscribe(events.listener(subject, "2"));

        subscription_2.unsubscribe();

        subscribers.emit(&Event::Foo);
        expect_that!(
            events.take_events(),
            elements_are![&SubjectEvent {
                subject,
                listener: "1",
                event: Event::Foo
            }],
        );

        subscription_1.unsubscribe();
    }

    #[gtest]
    fn test_blocked_subscription() {
        let subject = "a";
        let subscribers = SubjectSubscriptions::<Event>::new();
        let events = ReceivedEventStore::new();

        let subscription_1 = subscribers.subscribe(events.listener(subject, "1"));
        let subscription_2 = subscribers.subscribe(events.listener(subject, "2"));

        subscription_2.block();

        subscribers.emit(&Event::Foo);
        expect_that!(
            events.take_events(),
            elements_are![&SubjectEvent {
                subject,
                listener: "1",
                event: Event::Foo
            }],
        );

        subscription_2.unblock();

        subscribers.emit(&Event::Foo);
        expect_that!(
            events.take_events(),
            unordered_elements_are![
                &SubjectEvent {
                    subject,
                    listener: "1",
                    event: Event::Foo
                },
                &SubjectEvent {
                    subject,
                    listener: "2",
                    event: Event::Foo
                }
            ],
        );

        subscription_1.unsubscribe();
    }

    #[gtest]
    fn test_subject_subscriptions_can_add_subscriber_during_emit() {
        let subs = Rc::new(SubjectSubscriptions::<Event>::new());
        let events = ReceivedEventStore::new();

        let subs_ref = subs.clone();
        let events_outer = events.clone();

        // GIVEN a single listener that subscribes another listener when it receives events.
        let inners = Rc::new(RefCell::new(Vec::new()));
        let outer = subs.subscribe(move |event| {
            events_outer.record("a", "outer", event);

            // When the outer subscriber receives an event, it subscribes another listener.
            let events_inner = events_outer.clone();
            let inner = subs_ref
                .subscribe(move |inner_event| events_inner.record("a", "inner", inner_event));

            inners.borrow_mut().push(inner);
        });

        // WHEN the first event is emitted.
        subs.emit(&Event::Foo);

        // THEN the only event received is from the outer listener.
        expect_that!(
            events.take_events(),
            elements_are![eq(&SubjectEvent {
                subject: "a",
                listener: "outer",
                event: Event::Foo,
            })],
        );

        // WHEN the second event is emitted.
        subs.emit(&Event::Foo);

        // THEN there should be two events received
        expect_that!(
            events.take_events(),
            unordered_elements_are![
                // One received from outer.
                eq(&SubjectEvent {
                    subject: "a",
                    listener: "outer",
                    event: Event::Foo,
                }),
                // One received from inner.
                eq(&SubjectEvent {
                    subject: "a",
                    listener: "inner",
                    event: Event::Foo,
                }),
            ],
        );

        outer.unsubscribe();
    }

    #[gtest]
    fn test_multi_subject_subscriptions_sends_events() {
        let subject_a = "a";
        let subject_b = "b";

        let subscribers = MultiSubjectSubscriptions::<&'static str, Event>::new();
        let events = ReceivedEventStore::new();

        let subscription_a_1 = subscribers.subscribe(&subject_a, events.listener(subject_a, "1"));
        let subscription_b_2 = subscribers.subscribe(&subject_b, events.listener(subject_b, "2"));

        subscribers.emit(&subject_a, &Event::Foo);
        expect_that!(
            events.take_events(),
            elements_are![eq(&SubjectEvent {
                subject: subject_a,
                listener: "1",
                event: Event::Foo,
            })],
        );

        subscribers.emit(&subject_b, &Event::Bar);
        subscribers.emit(&subject_b, &Event::Foo);
        expect_that!(
            events.take_events(),
            elements_are![
                eq(&SubjectEvent {
                    subject: subject_b,
                    listener: "2",
                    event: Event::Bar,
                }),
                eq(&SubjectEvent {
                    subject: subject_b,
                    listener: "2",
                    event: Event::Foo,
                }),
            ],
        );

        subscription_b_2.unsubscribe();
        subscribers.emit(&subject_b, &Event::Foo);
        expect_that!(events.take_events(), empty());

        subscription_a_1.unsubscribe();
        subscribers.emit(&subject_a, &Event::Foo);
        expect_that!(events.take_events(), empty());

        subscribers.remove_subject(&subject_a);
        subscribers.remove_subject(&subject_b);
    }

    #[gtest]
    fn test_multi_subject_subscriptions_can_add_subscriber_during_emit() {
        let subs = Rc::new(MultiSubjectSubscriptions::<&'static str, Event>::new());
        let events = ReceivedEventStore::new();

        let subs_ref = subs.clone();
        let events_outer = events.clone();

        // GIVEN a single listener that subscribes another listener when it receives events.
        let inners = Rc::new(RefCell::new(Vec::new()));
        let outer = subs.subscribe(&"a", move |event| {
            events_outer.record("a", "outer", event);

            // When the outer subscriber receives an event, it subscribes another listener.
            let inner = subs_ref.subscribe(&"a", events_outer.listener("a", "inner"));

            inners.borrow_mut().push(inner);
        });

        // WHEN the first event is emitted.
        subs.emit(&"a", &Event::Foo);

        // THEN the only event received is from the outer listener.
        expect_that!(
            events.take_events(),
            elements_are![eq(&SubjectEvent {
                subject: "a",
                listener: "outer",
                event: Event::Foo,
            })],
        );

        // WHEN the second event is emitted.
        subs.emit(&"a", &Event::Foo);

        // THEN there should be two events received
        expect_that!(
            events.take_events(),
            unordered_elements_are![
                // One received from outer.
                eq(&SubjectEvent {
                    subject: "a",
                    listener: "outer",
                    event: Event::Foo,
                }),
                // One received from inner.
                eq(&SubjectEvent {
                    subject: "a",
                    listener: "inner",
                    event: Event::Foo,
                }),
            ],
        );

        outer.unsubscribe();
    }
}
