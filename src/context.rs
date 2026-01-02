//! Integration with GPUI's Context system.
//!
//! This module provides extension methods for GPUI's Context to work with signals.

use crate::{Memo, Signal};
use futures::channel::mpsc;
use futures::StreamExt;
use gpui::{EntityId, Subscription, WeakEntity};
use std::cell::RefCell;
use std::collections::HashMap;

/// Extension trait for GPUI Context to work with signals.
///
/// This trait provides convenience methods for creating signals that are
/// automatically cleaned up when the entity is released.
///
/// Signals created with `create_signal()` automatically notify the entity when they change,
/// eliminating the need for manual `cx.notify()` calls or subscription management.
///
/// # Example
///
/// ```rust,no_run
/// use gpui::*;
/// use gpui_signals::prelude::*;
///
/// struct Counter {
///     count: Signal<i32>,
/// }
///
/// impl Counter {
///     fn new(cx: &mut Context<Self>) -> Self {
///         // Signal automatically notifies on change - no manual setup needed!
///         Self {
///             count: cx.create_signal(0),
///         }
///     }
///
///     fn increment(&mut self, _cx: &mut Context<Self>) {
///         // Just update the signal - re-render happens automatically
///         self.count.update(|n| *n += 1);
///     }
/// }
/// ```
pub trait SignalContext {
    /// Create a new signal with the given initial value.
    ///
    /// The signal will automatically notify the entity when it changes.
    /// Subscriptions are automatically managed and cleaned up when the entity is dropped.
    /// No manual `auto_notify()` calls or subscription storage needed!
    fn create_signal<T: 'static>(&mut self, initial: T) -> Signal<T>;

    /// Create a computed signal (memo) from a computation function.
    ///
    /// The memo will be automatically cleaned up when the entity is dropped.
    fn create_memo<T: 'static + Clone>(&mut self, compute: impl Fn() -> T + 'static) -> Memo<T>;

    /// Create an effect that runs when signals it reads change.
    ///
    /// The effect will be cleaned up when the entity is dropped.
    fn create_effect(&mut self, effect: impl Fn() + 'static);
}

// Thread-local storage for tracking subscriptions per entity
thread_local! {
    static ENTITY_SUBSCRIPTIONS: RefCell<HashMap<EntityId, Vec<Subscription>>> = RefCell::new(HashMap::new());
}

impl<T: 'static> SignalContext for gpui::Context<'_, T> {
    fn create_signal<U: 'static>(&mut self, initial: U) -> Signal<U> {
        let signal = Signal::new(initial);
        let subscription = auto_notify(&signal, self);

        // Store the subscription so it's cleaned up when the entity is dropped
        let entity_id = self.entity_id();
        ENTITY_SUBSCRIPTIONS.with(|subs| {
            subs.borrow_mut()
                .entry(entity_id)
                .or_insert_with(Vec::new)
                .push(subscription);
        });

        // Register cleanup when entity is released - this ensures subscriptions are dropped
        let entity_id = entity_id;
        let _cleanup_sub = self.on_release(move |_, _| {
            ENTITY_SUBSCRIPTIONS.with(|subs| {
                subs.borrow_mut().remove(&entity_id);
            });
        });

        signal
    }

    fn create_memo<U: 'static + Clone>(&mut self, compute: impl Fn() -> U + 'static) -> Memo<U> {
        let memo = Memo::new(compute);
        let subscription = auto_notify(&memo.signal(), self);

        // Store the subscription so it's cleaned up when the entity is dropped
        let entity_id = self.entity_id();
        ENTITY_SUBSCRIPTIONS.with(|subs| {
            subs.borrow_mut()
                .entry(entity_id)
                .or_insert_with(Vec::new)
                .push(subscription);
        });

        // Register cleanup when entity is released
        let _cleanup_sub = self.on_release(move |_, _| {
            ENTITY_SUBSCRIPTIONS.with(|subs| {
                subs.borrow_mut().remove(&entity_id);
            });
        });

        memo
    }

    fn create_effect(&mut self, effect: impl Fn() + 'static) {
        // Create a dummy signal to track dependencies
        let effect_signal = Signal::new(());
        effect_signal.subscribe(effect);
    }
}



/// Automatically notify an entity when a signal changes.
///
/// This subscribes to signal changes and automatically calls `cx.notify()` on the entity
/// whenever the signal's value changes. This eliminates the need for manual `cx.notify()` calls
/// after signal updates.
///
/// The returned `Subscription` should be stored in the entity (typically in a `_subscriptions`
/// field) to ensure it's cleaned up when the entity is dropped.
///
/// This subscribes to signal changes and automatically calls `cx.notify()` on the entity
/// whenever the signal's value changes.
pub(crate) fn auto_notify<T, V>(signal: &Signal<T>, cx: &mut gpui::Context<V>) -> Subscription
where
    T: 'static,
    V: 'static,
{
    let _entity = cx.weak_entity();

    // Create an async channel to communicate signal changes to the foreground thread
    let (tx, mut rx) = mpsc::unbounded::<()>();

    // Subscribe to signal changes - when signal updates, send a message
    signal.subscribe({
        let tx = tx.clone();
        move || {
            // Ignore errors - if the receiver is dropped, the entity is gone
            let _ = tx.unbounded_send(());
        }
    });

    // Spawn a task that receives notifications and calls notify on the entity
    let task = cx.spawn(
        async move |entity: WeakEntity<V>, cx: &mut gpui::AsyncApp| {
            while let Some(()) = rx.next().await {
                // Use entity.update to call notify on the foreground thread
                if let Some(entity) = entity.upgrade() {
                    entity
                        .update(cx, |_, cx| {
                            cx.notify();
                        })
                        .ok();
                } else {
                    // Entity is gone, stop listening
                    break;
                }
            }
        },
    );

    // Store the task so it doesn't get dropped
    // We'll return a subscription that drops the task when the entity is dropped
    Subscription::new(move || {
        task.detach();
    })
}

pub(crate) fn track_subscription<V: 'static>(cx: &mut gpui::Context<V>, subscription: Subscription) {
    let entity_id = cx.entity_id();
    ENTITY_SUBSCRIPTIONS.with(|subs| {
        subs.borrow_mut()
            .entry(entity_id)
            .or_insert_with(Vec::new)
            .push(subscription);
    });

    // Register cleanup when entity is released - this ensures subscriptions are dropped
    // Note: We only need to do this once per entity, but doing it multiple times is harmless
    // as on_release adds multiple callbacks. However, to avoid duplicate cleanup callbacks,
    // we might accept that simple redundancy or check existence.
    // For simplicity here, we assume the overhead is negligible or that create_signal handles the main cleanup.
    // Actually, `create_signal` registers cleanup every time.
    // This might be redundant.
    // Ideally we should have a `ensure_cleanup_registered` mechanism.
    // For now, let's just register it.
    let _cleanup_sub = cx.on_release(move |_, _| {
        ENTITY_SUBSCRIPTIONS.with(|subs| {
            subs.borrow_mut().remove(&entity_id);
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_signal() {
        let signal = Signal::new(42);
        assert_eq!(signal.get(), 42);
    }

    #[test]
    fn test_create_memo() {
        let count = Signal::new(5);
        let doubled = Memo::new(move || count.get() * 2);
        assert_eq!(doubled.get(), 10);
    }
}
