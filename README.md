# GPUI Signals

A reactive state management library for GPUI inspired by signals patterns from [dioxus-signals](https://crates.io/crates/dioxus-signals) and [generational-box](https://crates.io/crates/generational-box).

## Features

- **Copy-able handles**: All signal types implement `Copy` for ergonomic use in closures and event handlers
- **Automatic tracking**: Signals automatically track dependencies when read
- **Computed signals**: Derive reactive state from other signals with `Memo`
- **Memory safe**: Generational arena prevents use-after-free without unsafe code
- **GPUI integration**: functional `cx.create_signal` and `cx.create_memo` API automatically handles notifications
- **Ergonomic**: `IntoElement` support allows passing signals directly to UI views, `toggle()` helper for booleans, `Eq`/`Hash` support.

## Quick Start

Add `gpui_signals` to your dependencies:

```toml
[dependencies]
gpui_signals = { path = "../gpui_signals" }
```

## Core Concepts

### Signals

Signals are reactive containers for values. When a signal's value changes, all subscribers are automatically notified.

```rust
use gpui_signals::Signal;

// Create a signal
let count = Signal::new(0);

// Read the value
assert_eq!(count.get(), 0);

// Update the value
count.set(5);
count.update(|n| *n += 1);
count += 1; // Operator overload

// Toggle boolean signals
let visible = Signal::new(false);
visible.toggle();

// Subscribe to changes
count.subscribe(|| {
    println!("Count changed!");
});
```

### Computed Signals (Memos)

Memos are signals that derive their value from other signals. They automatically track dependencies and recompute when those dependencies change.

```rust
use gpui_signals::{Signal, Memo};

let count = Signal::new(5);
let doubled = Memo::new(move || count.get() * 2);

assert_eq!(doubled.get(), 10);

count.set(10);
assert_eq!(doubled.get(), 20); // Automatically updated!
```

### Integration with GPUI

The library provides a convenient `SignalContext` trait that extends GPUI's `Context`. Creating signals this way automatically wires up notification handlers, so your views rebuild when signals change.

```rust
use gpui::*;
use gpui_signals::prelude::*;

struct Counter {
    count: Signal<i32>,
}

impl Counter {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            // Signal matches lifecycle of the entity and auto-notifies on change
            count: cx.create_signal(0),
        }
    }
}

impl Render for Counter {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        // Signals implement IntoElement, so you can pass them directly to child()
        // No manual subscription needed!
        div()
            .child(self.count) 
            .child(
                button()
                    .child("Increment")
                    .on_click(cx.listener(move |view, _, _cx| {
                        view.count += 1;
                        // No cx.notify() needed!
                    }))
            )
    }
}
```

## API Reference

### `create_signal` / `create_memo`

The primary way to use signals in GPUI components.

- `cx.create_signal(initial)`: Creates a signal that auto-notifies the view on change.
- `cx.create_memo(compute)`: Creates a memo that auto-notifies the view on change.
- `cx.create_effect(effect)`: Runs `effect` whenever signals it reads change.

Example:

```rust,no_run
use gpui::*;
use gpui_signals::prelude::*;

struct Logger {
    count: Signal<i32>,
}

impl Logger {
    fn new(cx: &mut Context<Self>) -> Self {
        let count = cx.create_signal(0);
        cx.create_effect(move || {
            let value = count.get();
            println!("count is now {}", value);
        });
        Self { count }
    }
}
```

### `Signal<T>`

A reactive signal that holds a value of type `T`.

**Methods:**
- `get()`: Get value (tracks read)
- `set(val)`: Set value and notify
- `update(|v| ...)`: Modify value
- `update_with(|v| ...)`: Modify value and return a result
- `set_if_changed(val)`: Set only when the value changes
- `toggle()`: Toggle boolean value
- `read_only()`: Get read-only view

**Traits:**
- `Copy`, `Clone`
- `PartialEq`, `Eq`, `Hash`
- `IntoElement` (renders current value string)
- Ops: `+=`, `-=`, `*=`, `/=`

## Examples

Run the counter example:

```bash
cargo run --example counter
```

Run the todo list example:

```bash
cargo run --example todo
```

## Architecture

### Storage Backend

The library uses a generational arena pattern for memory-safe signal storage:

- **SlotMap**: Arena-based storage with stable IDs
- **Generational Checking**: Prevents use-after-free when signals are dropped
- **Thread-local**: Matches GPUI's single-threaded window model
- **Interior Mutability**: Copy handles can mutate values via `RefCell`-like access

### Reactivity Model

Signals use a push-based reactivity system:

1. **Reads are tracked** during reactive contexts (like `Memo::new`)
2. **Dependencies are recorded** automatically
3. **Writes notify subscribers** immediately
4. **Memos recompute** when dependencies change

## Design Decisions

### why Copy Handles?

Copy-able signal handles allow you to use them in `move` closures without cloning:

```rust
let count = Signal::new(0);
// No .clone() needed!
button.on_click(move |_| {
    count += 1; 
});
```

### Comparison to Alternatives

**Direct GPUI State:**
Requires manual `cx.notify()`.
```rust
view.count += 1;
cx.notify(); // Valid, but manual
```

**Signals:**
Auto-notify observers.
```rust
view.count += 1; // View automatically rebuilds
```

## License

Licensed under Apache-2.0 OR GPL-3.0-or-later, matching GPUI's licensing.
