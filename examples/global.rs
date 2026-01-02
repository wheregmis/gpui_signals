//! Global Signal Example
//!
//! Demonstrates how to use global signals to share state across views.

use gpui::*;
use gpui_signals::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Theme {
    Light,
    Dark,
}

struct ThemeToggle;

impl Render for ThemeToggle {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Subscribe to global theme signal
        let theme_signal = cx.use_global::<Theme>();
        let theme = theme_signal.get();

        div()
            .flex()
            .items_center()
            .justify_center()
            .p_4()
            .bg(match theme {
                Theme::Light => rgb(0xffffff),
                Theme::Dark => rgb(0x000000),
            })
            .text_color(match theme {
                Theme::Light => rgb(0x000000),
                Theme::Dark => rgb(0xffffff),
            })
            .child(
                div()
                    .cursor_pointer()
                    .p_2()
                    .border_1()
                    .rounded_md()
                    .child(match theme {
                        Theme::Light => "Switch to Dark Mode",
                        Theme::Dark => "Switch to Light Mode",
                    })
                    .on_mouse_down(MouseButton::Left, cx.listener(move |_, _, _, cx| {
                        // Update global signal without needing subscription (can use global_signal for this)
                        let theme_signal = cx.global_signal::<Theme>();
                        theme_signal.update(|t| *t = match *t {
                            Theme::Light => Theme::Dark,
                            Theme::Dark => Theme::Light,
                        });
                    })),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        // Initialize global signal
        cx.init_global(Theme::Light);

        let bounds = Bounds::centered(None, Size::new(px(400.0), px(300.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_window, cx| cx.new(|_cx| ThemeToggle),
        )
        .unwrap();
        cx.activate(true);
    });
}
