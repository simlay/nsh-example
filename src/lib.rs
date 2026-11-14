//! The app's Rust UI: its root component, which `nsh run`, `nsh package`
//! and `cargo run` mount.

use nsh_core::prelude::*;
use nsh_base::theme;

// The nsh Android runtime, only when building for Android (a target-specific
// dependency in `Cargo.toml`). Cargo exposes it under its lib name, `nsh`:
// the runtime crate's own `[lib] name = "nsh"` (see the crate's Cargo.toml),
// which is why the built `libnsh.so` is named that too.
#[cfg(target_os = "android")]
use nsh as platform;

// Android entry point. The nsh Android runtime (`com.longbridge.nsh.AppActivity`,
// in the `nsh` runtime) loads this crate as `libnsh.so`, calls
// `Native.runRust(handle)`, which the `main!` macro below turns into
// `run_root(handle, Root::of::<App>())`: the engine runs the Rust root on its
// own thread while the activity drives the view. The library is built as a
// `cdylib` named `nsh` (see `Cargo.toml`), so the runtime's
// `System.loadLibrary("nsh")` finds it. Only compiled on Android.
#[cfg(target_os = "android")]
platform::main!(platform::Root::of::<App>());

/// A single TODO item.
#[derive(Clone, PartialEq)]
struct Todo {
    id: u32,
    text: String,
    completed: bool,
}

/// The app's UI set (`nsh.json` `"rust"`): what a TSX entry may import from
/// `nsh-example/native` (`README.md`, "Adding a TSX entry").
#[native_module("nsh-example/native")]
fn native(registry: &mut Registry) {
    registry.register::<App>();
}

/// The application's root component.
#[component]
pub fn App() -> impl IntoElement {
    let todos = signal_with(vec![], SignalEquals::Never);
    let next_id = signal(1_u32);
    let new_text = signal(String::from(""));
    let filter = signal(String::from("all"));

    view()
        .flex_1()
        .bg(move || theme().background)
        .text_color(move || theme().foreground)
        .child(
            view()
                .flex_1()
                .p_safe_area()
                .child(
                    view()
                        .flex_1()
                        .flex_col()
                        // Header
                        .child(
                            view()
                                .px_4()
                                .py_3()
                                .bg(move || theme().primary)
                                .child(
                                    text("My Todos")
                                        .text_size(px(24.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(move || theme().primary_foreground),
                                ),
                        )
                        // Input section
                        .child(
                            view()
                                .px_4()
                                .py_2()
                                .bg(move || theme().background)
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(
                                    nsh_base::input()
                                        .flex_1()
                                        .placeholder("Add a new todo...")
                                        .value(move || new_text.get())
                                        .on_change(move |text: &str| new_text.set(text.to_string()))
                                        .on_submit(move |text: &str| {
                                            let text = text.trim().to_string();
                                            if !text.is_empty() {
                                                let id = next_id.get();
                                                next_id.update(|v| *v += 1);
                                                todos.update(|list| {
                                                    list.push(Todo {
                                                        id,
                                                        text,
                                                        completed: false,
                                                    });
                                                });
                                                new_text.set(String::from(""));
                                            }
                                        })
                                        .return_key_type(ReturnKeyType::Done),
                                )
                                .child(
                                    view()
                                        .px_3()
                                        .py_2()
                                        .rounded(px(8.0))
                                        .bg(move || theme().primary)
                                        .active(|s| s.opacity(0.6))
                                        .on_press(move || {
                                            let text = new_text.get();
                                            let text = text.trim().to_string();
                                            if !text.is_empty() {
                                                let id = next_id.get();
                                                next_id.update(|v| *v += 1);
                                                todos.update(|list| {
                                                    list.push(Todo {
                                                        id,
                                                        text,
                                                        completed: false,
                                                    });
                                                });
                                                new_text.set(String::from(""));
                                            }
                                        })
                                        .child(
                                            text("Add")
                                                .text_size(px(15.0))
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(move || theme().primary_foreground),
                                        ),
                                ),
                        )
                        // Filter tabs
                        .child(
                            view()
                                .flex_row()
                                .justify_center()
                                .gap_3()
                                .py_2()
                                .child(filter_button(filter.clone(), "all".to_string(), "All".into()))
                                .child(filter_button(filter.clone(), "active".to_string(), "Active".into()))
                                .child(filter_button(filter.clone(), "completed".to_string(), "Done".into())),
                        )
                        // Todo list
                        .child(
                            scroll()
                                .flex_1()
                                .child(
                                    view()
                                        .flex_col()
                                        .gap_1()
                                        .py_2()
                                        .child(
                                            each(
                                                move || {
                                                    get_filtered(&todos.get(), &filter.get())
                                                        .into_iter()
                                                        .cloned()
                                                        .collect::<Vec<Todo>>()
                                                },
                                                |todo: &Todo| todo.id,
                                                move |todo_signal, _idx_signal| {
                                                    todo_item(todo_signal, todos)
                                                },
                                            )
                                        )
                                        .child(
                                            // Show message when no todos match filter
                                            when(
                                                move || {
                                                    let all_todos = todos.get();
                                                    let filtered = get_filtered(&all_todos, &filter.get());
                                                    filtered.is_empty() && !all_todos.is_empty()
                                                },
                                                || {
                                                    view()
                                                        .flex_1()
                                                        .items_center()
                                                        .justify_center()
                                                        .child(
                                                            text("No matching todos")
                                                                .text_size(px(16.0))
                                                                .text_color(move || theme().muted_foreground),
                                                        )
                                                },
                                            ),
                                        ),
                                ),
                        )
                        // Footer with count
                        .child(
                            view()
                                .px_4()
                                .py_2()
                                .bg(move || theme().secondary)
                                .child(
                                    text(move || {
                                        let remaining = get_remaining_count(&todos.get());
                                        format!("{} item{} left", remaining, if remaining == 1 { "" } else { "s" })
                                    })
                                    .text_size(px(13.0))
                                    .text_color(move || theme().muted_foreground),
                                ),
                        ),
                ),
        )
}

/// A filter button component.
fn filter_button(filter: Signal<String>, value: String, label: SharedString) -> impl IntoElement {
    let value_bg = value.clone();
    let value_color = value.clone();
    view()
        .px_3()
        .py_1()
        .rounded(px(6.0))
        .bg({
            move || {
                if filter.get() == value_bg {
                    theme().primary
                } else {
                    theme().secondary
                }
            }
        })
        .active(|s| s.opacity(0.7))
        .on_press(move || filter.set(value.clone()))
        .child(
            text(label.clone())
                .text_size(px(13.0))
                .text_color({
                    move || {
                        if filter.get() == value_color {
                            theme().primary_foreground
                        } else {
                            theme().muted_foreground
                        }
                    }
                }),
        )
}

/// Get filtered todos based on current filter.
fn get_filtered<'a>(todos: &'a [Todo], filter: &str) -> Vec<&'a Todo> {
    match filter {
        "active" => todos.iter().filter(|t| !t.completed).collect(),
        "completed" => todos.iter().filter(|t| t.completed).collect(),
        _ => todos.iter().collect(),
    }
}

/// Get count of remaining (incomplete) todos.
fn get_remaining_count(todos: &[Todo]) -> usize {
    todos.iter().filter(|t| !t.completed).count()
}

/// A single todo item with toggle and delete functionality.
fn todo_item(todo: ReadSignal<Todo>, todos: Signal<Vec<Todo>>) -> impl IntoElement {
    let todo_ref = todo;

    view()
        .flex_row()
        .items_center()
        .gap_3()
        .px_4()
        .py_2()
        .bg(move || {
            if todo_ref.get().completed {
                theme().secondary
            } else {
                theme().background
            }
        })
        .child(
            // Checkbox area (tappable)
            view()
                .w(px(24.0))
                .h(px(24.0))
                .rounded(px(4.0))
                .border(px(2.0))
                .border_color(move || theme().primary)
                .bg(move || {
                    if todo_ref.get().completed {
                        theme().primary
                    } else {
                        theme().background
                    }
                })
                .active(|s| s.opacity(0.6))
                .on_press(move || {
                    let id = todo_ref.get().id;
                    todos.update(|list| {
                        if let Some(item) = list.iter_mut().find(|t| t.id == id) {
                            item.completed = !item.completed;
                        }
                    });
                }),
        )
        .child(
            when(
                move || todo_ref.get().completed,
                move || {
                    text(todo_ref.get().text.clone())
                        .flex_1()
                        .text_size(px(16.0))
                        .text_color(move || theme().muted_foreground)
                        .line_through()
                },
            )
            .otherwise(move || {
                text(todo_ref.get().text.clone())
                    .flex_1()
                    .text_size(px(16.0))
                    .text_color(move || theme().foreground)
            }),
        )
        .child(
            view()
                .px_2()
                .py_1()
                .rounded(px(4.0))
                .bg(move || theme().destructive)
                .active(|s| s.opacity(0.6))
                .on_press(move || {
                    let id = todo_ref.get().id;
                    todos.update(|list| {
                        list.retain(|t| t.id != id);
                    });
                })
                .child(
                    text("✕")
                        .text_size(px(14.0))
                        .text_color(move || theme().background),
                ),
        )
}
