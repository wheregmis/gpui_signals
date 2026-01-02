//! Generational arena storage for signals.
//!
//! Uses a slot map with generational indices to provide memory-safe Copy handles
//! to signal values. This prevents use-after-free bugs when signals are dropped
//! and their slots are reused.

use slotmap::{new_key_type, SlotMap};
use std::any::Any;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;

new_key_type! {
    /// Unique identifier for a signal in the storage.
    pub struct SignalId;
}

/// A type-erased signal value with generational checking.
pub(crate) struct SignalValue {
    /// The actual value, boxed and type-erased.
    pub value: Box<dyn Any>,
    /// Generation counter to detect stale handles.
    pub generation: u32,
}

/// Subscriber callback for signal changes.
pub(crate) type Subscriber = Rc<dyn Fn()>;

/// Thread-local storage for all signals.
///
/// This is the backing store for all signal values and their subscribers.
/// It uses interior mutability to allow Copy handles to access and modify values.
pub(crate) struct SignalStorage {
    /// Arena of signal values indexed by SignalId.
    values: SlotMap<SignalId, SignalValue>,
    /// Subscribers for each signal.
    subscribers: BTreeMap<SignalId, Vec<Subscriber>>,
    /// Dependencies tracked for each observer (observer -> set of signals read).
    dependencies: BTreeMap<SignalId, HashSet<SignalId>>,
    /// The current observer (if any) for dependency tracking.
    current_observer: Option<SignalId>,
}

impl SignalStorage {
    /// Create a new empty signal storage.
    pub fn new() -> Self {
        Self {
            values: SlotMap::with_key(),
            subscribers: BTreeMap::new(),
            dependencies: BTreeMap::new(),
            current_observer: None,
        }
    }

    /// Insert a new signal value and return its ID.
    pub fn insert<T: 'static>(&mut self, value: T) -> SignalId {
        let signal_value = SignalValue {
            value: Box::new(value),
            generation: 0,
        };
        self.values.insert(signal_value)
    }

    /// Get a reference to a signal value.
    pub fn get<T: 'static>(&self, id: SignalId, generation: u32) -> Option<&T> {
        self.values.get(id).and_then(|signal_value| {
            if signal_value.generation == generation {
                signal_value.value.downcast_ref()
            } else {
                None
            }
        })
    }

    /// Get a mutable reference to a signal value.
    pub fn get_mut<T: 'static>(&mut self, id: SignalId, generation: u32) -> Option<&mut T> {
        self.values.get_mut(id).and_then(|signal_value| {
            if signal_value.generation == generation {
                signal_value.value.downcast_mut()
            } else {
                None
            }
        })
    }

    /// Update a signal value and notify subscribers.
    pub fn set<T: 'static>(
        &mut self,
        id: SignalId,
        generation: u32,
        value: T,
    ) -> Option<Vec<Subscriber>> {
        if let Some(signal_value) = self.values.get_mut(id) {
            if signal_value.generation == generation {
                signal_value.value = Box::new(value);
                let callbacks: Vec<Subscriber> = self
                    .subscribers
                    .get(&id)
                    .map(|subs| subs.iter().cloned().collect())
                    .unwrap_or_default();
                return Some(callbacks);
            }
        }
        None
    }

    /// Update a signal value with a closure and notify subscribers.
    pub fn update<T: 'static, R>(
        &mut self,
        id: SignalId,
        generation: u32,
        f: impl FnOnce(&mut T) -> R,
    ) -> Option<(R, Vec<Subscriber>)> {
        if let Some(value) = self.get_mut::<T>(id, generation) {
            let result = f(value);
            let callbacks = self
                .subscribers
                .get(&id)
                .map(|subs| subs.iter().cloned().collect())
                .unwrap_or_default();
            Some((result, callbacks))
        } else {
            None
        }
    }



    /// Subscribe to changes on a signal.
    pub fn subscribe(&mut self, id: SignalId, callback: impl Fn() + 'static) {
        self.subscribers
            .entry(id)
            .or_default()
            .push(Rc::new(callback));
    }

    /// Track a read for the current observer.
    pub fn track_read(&mut self, id: SignalId) {
        if let Some(observer_id) = self.current_observer {
            let deps = self.dependencies.entry(observer_id).or_default();
            if deps.insert(id) {
                // Only subscribe once per observer/dependency pair.
                let observer_ptr = observer_id;
                self.subscribe(id, move || notify_subscribers(observer_ptr));
            }
        }
    }

    /// Set the current observer for dependency tracking.
    pub fn set_observer(&mut self, observer: Option<SignalId>) -> Option<SignalId> {
        // Don't clear dependencies when recomputing. This prevents duplicate subscriptions:
        // - If a dependency was already tracked, deps.insert() returns false, so no new subscription
        // - If it's a new dependency, deps.insert() returns true, so we subscribe
        // This way, each dependency only gets one subscription per observer.
        std::mem::replace(&mut self.current_observer, observer)
    }
}

thread_local! {
    static STORAGE: RefCell<SignalStorage> = RefCell::new(SignalStorage::new());
}

/// Access the thread-local signal storage.
pub(crate) fn with_signal_storage<R>(f: impl FnOnce(&mut SignalStorage) -> R) -> R {
    STORAGE.with(|storage| f(&mut storage.borrow_mut()))
}

/// Notify all subscribers of a signal by temporarily borrowing storage.
pub(crate) fn notify_subscribers(id: SignalId) {
    let callbacks: Vec<Subscriber> = with_signal_storage(|storage| {
        storage
            .subscribers
            .get(&id)
            .map(|subs| subs.iter().cloned().collect())
            .unwrap_or_default()
    });

    for callback in callbacks {
        callback();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        with_signal_storage(|storage| {
            let id = storage.insert(42i32);
            assert_eq!(storage.get::<i32>(id, 0), Some(&42));
        });
    }

    #[test]
    fn test_update() {
        with_signal_storage(|storage| {
            let id = storage.insert(10i32);
            let callbacks = storage
                .update::<i32, _>(id, 0, |value| *value += 5)
                .map(|(_, callbacks)| callbacks)
                .unwrap();
            for callback in callbacks {
                callback();
            }
            assert_eq!(storage.get::<i32>(id, 0), Some(&15));
        });
    }

    #[test]
    fn test_subscribe_and_notify() {
        use parking_lot::Mutex;
        use std::sync::Arc;

        with_signal_storage(|storage| {
            let id = storage.insert(0i32);
            let called = Arc::new(Mutex::new(false));
            let called_clone = called.clone();

            storage.subscribe(id, move || {
                *called_clone.lock() = true;
            });

            let callbacks = storage.set(id, 0, 10).unwrap();
            for callback in callbacks {
                callback();
            }
            assert!(*called.lock());
        });
    }
}
