//! Simple counter example demonstrating gpui_signals.
//!
//! This example shows:
//! - Creating signals
//! - Updating signal values  
//! - Computing derived state with Memo
//! - Integrating with GPUI

use gpui::*;
use gpui_signals::prelude::*;

struct Counter {
    count: Signal<i32>,
    doubled: Memo<i32>,
    show_stats: Signal<bool>,
}

impl Counter {
    fn new(cx: &mut Context<Self>) -> Self {
        // Signals created with cx.create_signal() automatically notify on change
        let count = cx.create_signal(0);
        let doubled = cx.create_memo(move || count.get() * 2);
        let show_stats = cx.create_signal(true);

        Self {
            count,
            doubled,
            show_stats,
        }
    }

    fn change_by(&mut self, delta: i32, _cx: &mut Context<Self>) {
        self.count.update(|value| *value += delta);
    }

    fn reset(&mut self, _cx: &mut Context<Self>) {
        self.count.set(0);
    }
}

fn button(
    label: &'static str,
    cx: &mut Context<Counter>,
    on_click: impl Fn(&mut Counter, &mut Context<Counter>) + 'static,
) -> impl IntoElement {
    div()
        .id(label)
        .bg(rgb(0x3a3a3a))
        .border_1()
        .border_color(rgb(0x4f4f4f))
        .rounded_md()
        .px_4()
        .py_2()
        .text_sm()
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(0xffffff))
        .child(label)
        .on_click(cx.listener(move |this, _, _, cx| on_click(this, cx)))
}

impl Render for Counter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_6()
            .p_8()
            .bg(rgb(0x2d2d2d))
            .text_color(rgb(0xffffff))
            .size_full()
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .child("🎯 Counter with Signals"),
            )
            .child(
                div()
                    .text_lg()
                    .child("Update the counter and see derived state react."),
            )
            .child(
                if self.show_stats.get() {
                    div()
                        .flex()
                        .flex_col()
                        .gap_4()
                        .bg(rgb(0x1d1d1d))
                        .p_6()
                        .rounded_lg()
                        .border_1()
                        .border_color(rgb(0x444444))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0x50fa7b))
                                .child("Signal State:"),
                        )
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .items_center()
                                .py_2()
                                .child(div().text_sm().child("Count:"))
                                .child(
                                    div()
                                        .text_2xl()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0x4a9eff))
                                        .child(self.count),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .items_center()
                                .py_2()
                                .child(div().text_sm().child("Doubled (computed):"))
                                .child(
                                    div()
                                        .text_2xl()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0x50fa7b))
                                        .child(self.doubled),
                                ),
                        )
                } else {
                    div().child("Stats hidden")
                        .p_4()
                        .bg(rgb(0x1d1d1d))
                        .rounded_lg()
                        .text_color(rgb(0x888888))
                        .text_sm()
                }
            )
            .child(
                div()
                    .flex()
                    .gap_3()
                    .pt_2()
                    .child(button("+1", cx, |this, cx| this.change_by(1, cx)))
                    .child(button("-1", cx, |this, cx| this.change_by(-1, cx)))
                    .child(button("Reset", cx, |this, cx| this.reset(cx)))
                    .child(button("Toggle Stats", cx, |this, _| {
                        this.show_stats.toggle();
                    })),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x888888))
                    .child("Run with: cargo run --example counter"),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, Size::new(px(600.0), px(400.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_window, cx| cx.new(Counter::new),
        )
        .unwrap();
        cx.activate(true);
    });
}
