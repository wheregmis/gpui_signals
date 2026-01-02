//! # GPUI Signals
//!
//! A reactive state management library for GPUI inspired by signals patterns.
//!
//! ## Features
//!
//! - **Copy-able handles**: All signal types implement `Copy` for ergonomic use
//! - **Automatic tracking**: Views automatically subscribe to signals they read
//! - **Computed signals**: Derive reactive state from other signals with `Memo`
//! - **Memory safe**: Generational arena prevents use-after-free without unsafe code
//!
//! ## Example
//!
//! ```rust,no_run
//! use gpui::Context;
//! use gpui_signals::prelude::*;
//!
//! struct Counter {
//!     count: Signal<i32>,
//! }
//!
//! impl Counter {
//!     fn new(cx: &mut Context<Self>) -> Self {
//!         Self {
//!             count: cx.create_signal(0),
//!         }
//!     }
//! }
//! ```
//!
//! ```rust,no_run
//! use gpui::*;
//! use gpui_signals::prelude::*;
//!
//! struct Logger {
//!     count: Signal<i32>,
//! }
//!
//! impl Logger {
//!     fn new(cx: &mut Context<Self>) -> Self {
//!         let count = cx.create_signal(0);
//!         cx.create_effect(move || {
//!             let value = count.get();
//!             println!("count is now {}", value);
//!         });
//!         Self { count }
//!     }
//! }
//! ```

mod computed;
mod context;
mod global;
mod signal;
mod storage;


pub use computed::Memo;
pub use context::SignalContext;
pub use global::GlobalSignalContext;
pub use signal::{ReadOnlySignal, Signal};

// Re-export the prelude
pub mod prelude {
    pub use crate::{GlobalSignalContext, Memo, ReadOnlySignal, Signal, SignalContext};
}
