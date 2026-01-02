//! Async Signal Example
//!
//! Demonstrates how to update signals from async tasks.

use gpui::*;
use gpui_signals::prelude::*;
use std::time::Duration;

struct AsyncDemo {
    user: Signal<Option<String>>,
    loading: Signal<bool>,
}

impl AsyncDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            user: cx.create_signal(None),
            loading: cx.create_signal(false),
        }
    }

    fn fetch_user(&mut self, cx: &mut Context<Self>) {
        if self.loading.get() {
            return;
        }

        self.loading.set(true);
        self.user.set(None);

        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            // Simulate network delay
            cx.background_executor()
                .timer(Duration::from_secs(1))
                .await;

            // Simulate random success/failure or different users
            let user = if rand::random() {
                "Alice (Admin)".to_string()
            } else {
                "Bob (User)".to_string()
            };

            this.update(cx, |this, _cx| {
                this.user.set(Some(user));
                this.loading.set(false);
            })
            .ok();
        })
        .detach();
    }
}

impl Render for AsyncDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let loading = self.loading.get();
        let user = self.user.get();

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .gap_4()
            .bg(rgb(0x2d2d2d))
            .text_color(rgb(0xffffff))
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .child("Async Signal Demo"),
            )
            .child(
                div()
                    .w_64()
                    .h_24()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_md()
                    .bg(rgb(0x1d1d1d))
                    .border_1()
                    .border_color(rgb(0x444444))
                    .child(if loading {
                        div().child("Loading...")
                    } else if let Some(name) = user {
                        div().text_lg().child(format!("User: {}", name))
                    } else {
                        div().text_color(rgb(0x888888)).child("No user loaded")
                    }),
            )
            .child(
                div()
                    .px_4()
                    .py_2()
                    .bg(if loading {
                        rgb(0x555555)
                    } else {
                        rgb(0x4a9eff)
                    })
                    .rounded_md()
                    .cursor(if loading {
                        CursorStyle::Arrow
                    } else {
                        CursorStyle::PointingHand
                    })
                    .child(if loading { "Fetching..." } else { "Fetch User" })
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.fetch_user(cx);
                    })),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, Size::new(px(400.0), px(300.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_window, cx| cx.new(AsyncDemo::new),
        )
        .unwrap();
        cx.activate(true);
    });
}
