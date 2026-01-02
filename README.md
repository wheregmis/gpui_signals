# GPUI Signals

Reactive state for GPUI with copyable signals and automatic view notifications.

## Quick Start

```toml
[dependencies]
gpui_signals = { path = "../gpui_signals" }
```

```rust
use gpui::*;
use gpui_signals::prelude::*;

struct Counter {
    count: Signal<i32>,
}

impl Counter {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            count: cx.create_signal(0),
        }
    }
}

impl Render for Counter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child(self.count).child(
            button().child("Increment").on_click(cx.listener(|this, _, _| {
                this.count += 1;
            })),
        )
    }
}
```

## API

### `Signal<T>`

- `get()`, `get_untracked()`
- `set(val)`, `set_if_changed(val)`
- `update(|v| ...)`, `update_with(|v| ...)`
- `toggle()` (bool)
- `read_only()`

### `Memo<T>`

- `get()`, `get_untracked()`
- `with(|v| ...)`, `with_untracked(|v| ...)`
- `subscribe(|...| ...)`

### Context helpers

- `cx.create_signal(initial)`
- `cx.create_memo(compute)`
- `cx.create_effect(effect)`

## Examples

| Example | Focus |
| --- | --- |
| `examples/counter.rs` | Basic signal usage |
| `examples/async.rs` | Async updates with loading/error |
| `examples/todo.rs` | Collections + derived state |
| `examples/global.rs` | Global signals |

```bash
cargo run --example counter
cargo run --example async
cargo run --example todo
cargo run --example global
```

## Minimal patterns

### Async update (loading + error)

```rust,no_run
use gpui::*;
use gpui_signals::prelude::*;

struct AsyncDemo {
    loading: Signal<bool>,
    error: Signal<Option<String>>,
}

impl AsyncDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            loading: cx.create_signal(false),
            error: cx.create_signal(None),
        }
    }

    fn fetch(&mut self, cx: &mut Context<Self>) {
        self.loading.set(true);
        self.error.set(None);

        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(500))
                .await;
            this.update(cx, |this, _cx| {
                this.loading.set(false);
            })
            .ok();
        })
        .detach();
    }
}
```

### Global signal (theme toggle)

```rust,no_run
use gpui::*;
use gpui_signals::prelude::*;

#[derive(Clone, Copy)]
enum Theme {
    Light,
    Dark,
}

fn init(cx: &mut App) {
    cx.init_global(Theme::Light);
}

fn toggle(cx: &mut Context<impl Render>) {
    let theme = cx.global_signal::<Theme>();
    theme.update(|t| *t = match *t {
        Theme::Light => Theme::Dark,
        Theme::Dark => Theme::Light,
    });
}
```

## License
MIT License
