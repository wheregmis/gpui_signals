//! Todo list example demonstrating collections and computed state.
//!
//! This example shows:
//! - Working with Vec signals
//! - Computed signals for derived state
//! - More complex state management patterns

use gpui::prelude::*;
use gpui::*;
use gpui_signals::prelude::*;

#[derive(Clone, Debug)]
struct Todo {
    id: usize,
    text: String,
    completed: bool,
}

struct TodoList {
    todos: Signal<Vec<Todo>>,
    next_id: Signal<usize>,
    filter: Signal<Filter>,
    input_text: Signal<String>,
    completed_count: Memo<usize>,
    active_count: Memo<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Filter {
    All,
    Active,
    Completed,
}

impl TodoList {
    fn new(cx: &mut Context<Self>) -> Self {
        // Signals created with cx.create_signal() automatically notify on change
        let todos = cx.create_signal(vec![
            Todo {
                id: 0,
                text: "Learn GPUI".to_string(),
                completed: false,
            },
            Todo {
                id: 1,
                text: "Build with signals".to_string(),
                completed: false,
            },
        ]);

        let next_id = cx.create_signal(2);
        let filter = cx.create_signal(Filter::All);
        let input_text = cx.create_signal(String::new());

        let completed_count =
            cx.create_memo(move || todos.get().iter().filter(|todo| todo.completed).count());

        let active_count =
            cx.create_memo(move || todos.get().iter().filter(|todo| !todo.completed).count());

        Self {
            todos,
            next_id,
            filter,
            input_text,
            completed_count,
            active_count,
        }
    }

    fn add_todo(&mut self, text: Option<String>, _cx: &mut Context<Self>) {
        let text = text
            .or_else(|| {
                let text = self.input_text.get();
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            })
            .unwrap_or_else(|| {
                let id = self.next_id.get();
                format!("Todo {}", id)
            });

        let id = self.next_id.get();
        self.next_id.set(id + 1);

        self.todos.update(|todos| {
            todos.push(Todo {
                id,
                text,
                completed: false,
            });
        });

        self.input_text.set(String::new());
    }

    fn toggle_todo(&mut self, id: usize, _cx: &mut Context<Self>) {
        self.todos.update(|todos| {
            if let Some(todo) = todos.iter_mut().find(|t| t.id == id) {
                todo.completed = !todo.completed;
            }
        });
    }

    fn remove_todo(&mut self, id: usize, _cx: &mut Context<Self>) {
        self.todos.update(|todos| {
            todos.retain(|todo| todo.id != id);
        });
    }

    fn set_filter(&mut self, filter: Filter, _cx: &mut Context<Self>) {
        self.filter.set(filter);
    }

    fn filtered_todos(&self) -> Vec<Todo> {
        let filter = self.filter.get();
        self.todos
            .get()
            .into_iter()
            .filter(|todo| match filter {
                Filter::All => true,
                Filter::Active => !todo.completed,
                Filter::Completed => todo.completed,
            })
            .collect()
    }
}

impl Render for TodoList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let filtered = self.filtered_todos();
        let completed = self.completed_count.get();
        let active = self.active_count.get();
        let current_filter = self.filter.get();
        let input_text = self.input_text.get();

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .bg(rgb(0x2d2d2d))
            .text_color(rgb(0xffffff))
            .size_full()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .child("Todo List Example"),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().child(format!("Active: {} todos", active)))
                    .child(div().child(format!("Completed: {}", completed))),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .px_3()
                            .py_2()
                            .bg(rgb(0x1d1d1d))
                            .border_1()
                            .border_color(rgb(0x444444))
                            .rounded_md()
                            .text_sm()
                            .text_color(rgb(0xffffff))
                            .child(
                                div()
                                    .text_sm()
                                    .when(input_text.is_empty(), |this| {
                                        this.text_color(rgb(0x888888)).child("Add a new todo...")
                                    })
                                    .when(!input_text.is_empty(), |this| {
                                        this.text_color(rgb(0xffffff)).child(input_text.clone())
                                    }),
                            )
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|_this, _, _, _cx| {
                                    // Simple input handling - in a real app you'd use Editor
                                    // For this example, clicking Add will yes todos automatically
                                }),
                            ),
                    )
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .bg(rgb(0x4a9eff))
                            .rounded_md()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .cursor(CursorStyle::PointingHand)
                            .child("Add")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.add_todo(None, cx);
                                }),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .text_sm()
                            .cursor(CursorStyle::PointingHand)
                            .bg(if current_filter == Filter::All {
                                rgb(0x4a9eff)
                            } else {
                                rgb(0x3a3a3a)
                            })
                            .child("All")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    this.set_filter(Filter::All, cx);
                                }),
                            ),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .text_sm()
                            .cursor(CursorStyle::PointingHand)
                            .bg(if current_filter == Filter::Active {
                                rgb(0x4a9eff)
                            } else {
                                rgb(0x3a3a3a)
                            })
                            .child("Active")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    this.set_filter(Filter::Active, cx);
                                }),
                            ),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .text_sm()
                            .cursor(CursorStyle::PointingHand)
                            .bg(if current_filter == Filter::Completed {
                                rgb(0x4a9eff)
                            } else {
                                rgb(0x3a3a3a)
                            })
                            .child("Completed")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    this.set_filter(Filter::Completed, cx);
                                }),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .bg(rgb(0x1d1d1d))
                    .p_4()
                    .rounded_md()
                    .min_h(px(200.0))
                    .when(filtered.is_empty(), |this| {
                        this.child(
                            div()
                                .text_sm()
                                .text_color(rgb(0x888888))
                                .text_align(TextAlign::Center)
                                .py_8()
                                .child("No todos to show"),
                        )
                    })
                    .when(!filtered.is_empty(), |this| {
                        this.child(
                            div()
                                .text_sm()
                                .text_color(rgb(0x888888))
                                .pb_2()
                                .child(format!("Showing {} items", filtered.len())),
                        )
                    })
                    .children(filtered.into_iter().map(move |todo| {
                        let id = todo.id;
                        let text = todo.text.clone();
                        let completed = todo.completed;
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .p_2()
                            .rounded_md()
                            .bg(rgb(0x252525))
                            .child(
                                div()
                                    .w(px(20.0))
                                    .h(px(20.0))
                                    .rounded_md()
                                    .border_1()
                                    .border_color(rgb(0x4a9eff))
                                    .bg(if completed {
                                        rgb(0x4a9eff)
                                    } else {
                                        rgb(0x1d1d1d)
                                    })
                                    .cursor(CursorStyle::PointingHand)
                                    .child(
                                        div()
                                            .size_full()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(if completed { "✓" } else { "" }),
                                    )
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            this.toggle_todo(id, cx);
                                        }),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .text_color(if completed {
                                        rgb(0x888888)
                                    } else {
                                        rgb(0xffffff)
                                    })
                                    .when(completed, |this| this.line_through())
                                    .child(text),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .bg(rgb(0xcc4444))
                                    .rounded_md()
                                    .text_xs()
                                    .cursor(CursorStyle::PointingHand)
                                    .child("×")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            this.remove_todo(id, cx);
                                        }),
                                    ),
                            )
                            .into_any_element()
                    })),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, Size::new(px(600.0), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_window, cx| cx.new(TodoList::new),
        )
        .unwrap();
        cx.activate(true);
    });
}
