//! Core Signal type and operations.

use crate::storage::{with_signal_storage, SignalId};
use gpui::{IntoElement, SharedString};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;


/// A reactive signal that holds a value of type `T`.
///
/// Signals are Copy-able handles to reactive state. When a signal's value changes,
/// all subscribers are automatically notified.
///
/// # Examples
///
/// Signals are created using `cx.create_signal(initial_value)` in a GPUI context.
///
/// ```rust,no_run
/// use gpui::Context;
/// use gpui_signals::prelude::*;
///
/// struct MyView {
///     count: Signal<i32>,
/// }
///
/// impl MyView {
///     fn new(cx: &mut Context<Self>) -> Self {
///         Self {
///             count: cx.create_signal(0),
///         }
///     }
/// }
/// ```
pub struct Signal<T> {
    id: SignalId,
    generation: u32,
    _phantom: PhantomData<T>,
}

impl<T> Copy for Signal<T> {}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for Signal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for Signal<T> {}

impl<T> Hash for Signal<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Signal<bool> {
    /// Toggle the boolean value of the signal.
    pub fn toggle(&self) {
        self.update(|v| *v = !*v);
    }
}

impl<T: fmt::Display + Clone + 'static> IntoElement for Signal<T> {
    type Element = SharedString;

    fn into_element(self) -> Self::Element {
        self.get().to_string().into()
    }
}

impl<T: 'static> Signal<T> {
    /// Create a new signal with the given initial value.
    pub(crate) fn new(value: T) -> Self {
        with_signal_storage(|storage| {
            let id = storage.insert(value);
            Self {
                id,
                generation: 0,
                _phantom: PhantomData,
            }
        })
    }

    /// Get the current value of the signal.
    ///
    /// This will track the read if called within a reactive context.
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        with_signal_storage(|storage| {
            storage.track_read(self.id);
            storage
                .get::<T>(self.id, self.generation)
                .cloned()
                .expect("Signal value not found")
        })
    }

    /// Get a clone of the current value without tracking the read.
    ///
    /// Use this when you want to read a signal without subscribing to it.
    pub fn get_untracked(&self) -> T
    where
        T: Clone,
    {
        with_signal_storage(|storage| {
            storage
                .get::<T>(self.id, self.generation)
                .cloned()
                .expect("Signal value not found")
        })
    }

    /// Set the signal to a new value.
    ///
    /// This will notify all subscribers of the change.
    pub fn set(&self, value: T) {
        if let Some(callbacks) =
            with_signal_storage(|storage| storage.set(self.id, self.generation, value))
        {
            for callback in callbacks {
                callback();
            }
        }
    }

    /// Set the signal only if the value has changed.
    ///
    /// Returns true if the value was updated.
    pub fn set_if_changed(&self, value: T) -> bool
    where
        T: PartialEq,
    {
        let should_update = self.with_untracked(|current| current != &value);
        if should_update {
            self.set(value);
        }
        should_update
    }

    /// Update the signal's value with a closure.
    ///
    /// This will notify all subscribers of the change.
    ///
    /// This will notify all subscribers of the change.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        if let Some((_, callbacks)) =
            with_signal_storage(|storage| storage.update(self.id, self.generation, f))
        {
            for callback in callbacks {
                callback();
            }
        }
    }

    /// Update the signal's value with a closure and return a result.
    ///
    /// This will notify all subscribers of the change.
    pub fn update_with<R>(&self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        if let Some((result, callbacks)) =
            with_signal_storage(|storage| storage.update(self.id, self.generation, f))
        {
            for callback in callbacks {
                callback();
            }
            return Some(result);
        }
        None
    }

    /// Read the signal's value with a closure.
    ///
    /// This will track the read if called within a reactive context.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        with_signal_storage(|storage| {
            storage.track_read(self.id);
            let value = storage
                .get::<T>(self.id, self.generation)
                .expect("Signal value not found");
            f(value)
        })
    }

    /// Read the signal's value with a closure without tracking.
    pub fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        with_signal_storage(|storage| {
            let value = storage
                .get::<T>(self.id, self.generation)
                .expect("Signal value not found");
            f(value)
        })
    }

    /// Subscribe to changes on this signal.
    ///
    /// The callback will be called whenever the signal's value changes.
    pub fn subscribe(&self, callback: impl Fn() + 'static) {
        with_signal_storage(|storage| {
            storage.subscribe(self.id, callback);
        });
    }

    /// Convert this signal to a read-only signal.
    pub fn read_only(self) -> ReadOnlySignal<T> {
        ReadOnlySignal { inner: self }
    }

    /// Get the underlying signal ID (mainly for debugging).
    pub fn id(&self) -> SignalId {
        self.id
    }
}

impl<T: 'static + Default> Default for Signal<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: 'static + fmt::Debug + Clone> fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Signal")
            .field("id", &self.id)
            .field("value", &self.get_untracked())
            .finish()
    }
}

// Implement common trait operations that automatically call update()
impl<T: 'static + std::ops::AddAssign<T> + Clone> std::ops::AddAssign<T> for Signal<T> {
    fn add_assign(&mut self, rhs: T) {
        self.update(|v| *v += rhs);
    }
}

impl<T: 'static + std::ops::SubAssign<T> + Clone> std::ops::SubAssign<T> for Signal<T> {
    fn sub_assign(&mut self, rhs: T) {
        self.update(|v| *v -= rhs);
    }
}

impl<T: 'static + std::ops::MulAssign<T> + Clone> std::ops::MulAssign<T> for Signal<T> {
    fn mul_assign(&mut self, rhs: T) {
        self.update(|v| *v *= rhs);
    }
}

impl<T: 'static + std::ops::DivAssign<T> + Clone> std::ops::DivAssign<T> for Signal<T> {
    fn div_assign(&mut self, rhs: T) {
        self.update(|v| *v /= rhs);
    }
}

/// A read-only view of a signal.
///
/// This prevents accidental mutations while still allowing reads and subscriptions.
pub struct ReadOnlySignal<T> {
    inner: Signal<T>,
}

impl<T> Copy for ReadOnlySignal<T> {}

impl<T> Clone for ReadOnlySignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for ReadOnlySignal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Eq for ReadOnlySignal<T> {}

impl<T> Hash for ReadOnlySignal<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl<T: 'static> ReadOnlySignal<T> {
    /// Get the current value of the signal.
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.inner.get()
    }

    /// Get the current value without tracking the read.
    pub fn get_untracked(&self) -> T
    where
        T: Clone,
    {
        self.inner.get_untracked()
    }

    /// Read the signal's value with a closure.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.inner.with(f)
    }

    /// Read the signal's value with a closure without tracking.
    pub fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.inner.with_untracked(f)
    }

    /// Subscribe to changes on this signal.
    pub fn subscribe(&self, callback: impl Fn() + 'static) {
        self.inner.subscribe(callback);
    }
}

impl<T: 'static + fmt::Debug + Clone> fmt::Debug for ReadOnlySignal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReadOnlySignal")
            .field("value", &self.get_untracked())
            .finish()
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_creation() {
        let signal = Signal::new(42);
        assert_eq!(signal.get(), 42);
    }

    #[test]
    fn test_signal_set() {
        let signal = Signal::new(0);
        signal.set(10);
        assert_eq!(signal.get(), 10);
    }

    #[test]
    fn test_signal_update() {
        let signal = Signal::new(5);
        signal.update(|n| *n *= 2);
        assert_eq!(signal.get(), 10);
    }

    #[test]
    fn test_signal_update_with() {
        let signal = Signal::new(5);
        let result = signal.update_with(|n| {
            *n += 2;
            *n
        });
        assert_eq!(result, Some(7));
        assert_eq!(signal.get(), 7);
    }

    #[test]
    fn test_signal_with() {
        let signal = Signal::new(String::from("hello"));
        let len = signal.with(|s| s.len());
        assert_eq!(len, 5);
    }

    #[test]
    fn test_signal_subscribe() {
        use std::sync::Arc;
        use parking_lot::Mutex;

        let signal = Signal::new(0);
        let count = Arc::new(Mutex::new(0));
        let count_clone = count.clone();

        signal.subscribe(move || {
            *count_clone.lock() += 1;
        });

        signal.set(1);
        signal.set(2);
        signal.set(3);

        assert_eq!(*count.lock(), 3);
    }

    #[test]
    fn test_read_only_signal() {
        let signal = Signal::new(42);
        let read_only = signal.read_only();
        assert_eq!(read_only.get(), 42);
    }

    #[test]
    fn test_signal_add_assign() {
        let mut signal = Signal::new(5);
        signal += 3;
        assert_eq!(signal.get(), 8);
    }

    #[test]
    fn test_signal_toggle() {
        let signal = Signal::new(false);
        signal.toggle();
        assert!(signal.get());
        signal.toggle();
        assert!(!signal.get());
    }

    #[test]
    fn test_signal_set_if_changed() {
        let signal = Signal::new(5);
        assert!(!signal.set_if_changed(5));
        assert_eq!(signal.get(), 5);
        assert!(signal.set_if_changed(6));
        assert_eq!(signal.get(), 6);
    }

    #[test]
    fn test_signal_eq() {
        let s1 = Signal::new(10);
        let s2 = s1;
        let s3 = Signal::new(10); // Different signal, same value

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }
}
