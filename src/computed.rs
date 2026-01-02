//! Computed signals (memos) that derive from other signals.

use crate::signal::Signal;
use crate::storage::with_signal_storage;
use gpui::{IntoElement, SharedString};
use std::hash::{Hash, Hasher};
use std::{cell::Cell, fmt, marker::PhantomData, rc::Rc};

/// A computed signal that derives its value from other signals.
///
/// Memos automatically track dependencies and recompute when any dependency changes.
///
/// # Examples
///
/// Memos are created using `cx.create_memo(compute_fn)` in a GPUI context.
///
/// ```rust,no_run
/// use gpui::Context;
/// use gpui_signals::prelude::*;
///
/// struct MyView {
///     count: Signal<i32>,
///     doubled: Memo<i32>,
/// }
///
/// impl MyView {
///     fn new(cx: &mut Context<Self>) -> Self {
///         let count = cx.create_signal(5);
///         let doubled = cx.create_memo(move || count.get() * 2);
///         Self { count, doubled }
///     }
/// }
/// ```
pub struct Memo<T> {
    signal: Signal<T>,
    _phantom: PhantomData<fn() -> T>,
}

impl<T> Copy for Memo<T> {}

impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for Memo<T> {
    fn eq(&self, other: &Self) -> bool {
        self.signal == other.signal
    }
}

impl<T> Eq for Memo<T> {}

impl<T> Hash for Memo<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.signal.hash(state);
    }
}

impl<T: fmt::Display + Clone + 'static> IntoElement for Memo<T> {
    type Element = SharedString;

    fn into_element(self) -> Self::Element {
        self.get().to_string().into()
    }
}

impl<T: 'static + Clone> Memo<T> {
    /// Create a new memo from a computation function.
    ///
    /// The function will be called immediately and whenever dependencies change.
    pub(crate) fn new(compute: impl Fn() -> T + 'static) -> Self {
        let compute = Rc::new(compute);
        let recomputing = Rc::new(Cell::new(false));
        let signal = Signal::new(compute());
        let recompute_signal = signal;

        let recompute: Rc<dyn Fn()> = {
            let compute = compute.clone();
            let signal = recompute_signal;
            let recomputing = recomputing.clone();
            Rc::new(move || {
                if recomputing.replace(true) {
                    return;
                }

                // Track dependencies while we compute the new value so updates
                // to those signals will notify this memo's signal.
                let previous =
                    with_signal_storage(|storage| storage.set_observer(Some(signal.id())));
                let value = compute();
                with_signal_storage(|storage| storage.set_observer(previous));

                signal.set(value);
                recomputing.set(false);
            })
        };

        // Run once to register dependencies and seed the memo value.
        recompute();

        // Dependencies are automatically tracked via track_read() during recompute().
        // When dependencies change, they notify this memo's signal via notify_subscribers().
        // We need to subscribe to our own signal to receive those notifications.
        // However, we must be careful: when we update our signal via signal.set(), it will
        // notify subscribers including ourselves. The recomputing flag prevents infinite loops.
        signal.subscribe({
            let recompute = recompute.clone();
            move || recompute()
        });

        Self {
            signal,
            _phantom: PhantomData,
        }
    }

    /// Get the underlying signal.
    pub fn signal(&self) -> Signal<T> {
        self.signal
    }

    /// Get the current computed value.
    pub fn get(&self) -> T {
        self.signal.get()
    }

    /// Get the current value without tracking the read.
    pub fn get_untracked(&self) -> T {
        self.signal.get_untracked()
    }

    /// Read the computed value with a closure.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.signal.with(f)
    }

    /// Subscribe to changes in the computed value.
    pub fn subscribe(&self, callback: impl Fn() + 'static) {
        self.signal.subscribe(callback);
    }
}

impl<T: 'static + Clone + fmt::Debug> fmt::Debug for Memo<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Memo")
            .field("value", &self.get_untracked())
            .finish()
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::Signal;

    #[test]
    fn test_memo_basic() {
        let count = Signal::new(5);
        let doubled = Memo::new(move || count.get() * 2);

        assert_eq!(doubled.get(), 10);
    }

    #[test]
    fn test_memo_with_manual_updates() {
        let count = Signal::new(5);
        let doubled_signal = Signal::new(count.get() * 2);

        assert_eq!(doubled_signal.get(), 10);

        count.set(10);
        // Manually update the derived signal
        doubled_signal.set(count.get() * 2);
        assert_eq!(doubled_signal.get(), 20);
    }
}
