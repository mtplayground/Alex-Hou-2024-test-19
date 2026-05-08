use leptos::{ev::KeyboardEvent, html, prelude::*, task::spawn_local};
use leptos_router::hooks::use_location;

use crate::todo::{
    server::{clear_completed, list_todos, toggle_all, DeleteTodo, EditTodo, ToggleTodo},
    Filter, Todo,
};

#[component]
pub fn TodoMain() -> impl IntoView {
    let todos = all_todos_resource();
    let toggle_all_pending = RwSignal::new(false);

    Effect::new(move |_| {
        if !toggle_all_pending.get() {
            return;
        }

        spawn_local(async move {
            let _ = toggle_all().await;
            toggle_all_pending.set(false);
            refresh_todos();
        });
    });

    view! {
        <section
            class="main"
            class:hidden=move || match todos.get() {
                Some(Ok(todos)) => todos.is_empty(),
                Some(Err(_)) | None => true,
            }
        >
            <input
                id="toggle-all"
                class="toggle-all"
                type="checkbox"
                prop:checked=move || match todos.get() {
                    Some(Ok(todos)) => !todos.is_empty() && todos.iter().all(|todo| todo.completed),
                    Some(Err(_)) | None => false,
                }
                on:change=move |_| {
                    toggle_all_pending.set(true);
                }
            />
            <label for="toggle-all">"Mark all as complete"</label>
            <TodoList/>
        </section>
    }
}

#[component]
pub fn TodoFooter() -> impl IntoView {
    let todos = all_todos_resource();
    let location = use_location();
    let clear_completed_pending = RwSignal::new(false);

    Effect::new(move |_| {
        if !clear_completed_pending.get() {
            return;
        }

        spawn_local(async move {
            let _ = clear_completed().await;
            clear_completed_pending.set(false);
            refresh_todos();
        });
    });

    view! {
        <footer
            class="footer"
            class:hidden=move || match todos.get() {
                Some(Ok(todos)) => todos.is_empty(),
                Some(Err(_)) | None => true,
            }
        >
            <span class="todo-count">
                <strong>{move || match todos.get() {
                    Some(Ok(todos)) => todos.iter().filter(|todo| !todo.completed).count(),
                    Some(Err(_)) | None => 0,
                }}</strong>
                {move || {
                    let active_count = match todos.get() {
                        Some(Ok(todos)) => todos.iter().filter(|todo| !todo.completed).count(),
                        Some(Err(_)) | None => 0,
                    };

                    format!(
                        " {} left",
                        if active_count == 1 { "item" } else { "items" }
                    )
                }}
            </span>
            <ul class="filters">
                <li>
                    <a class:selected=move || location.pathname.get() == "/" href="/">"All"</a>
                </li>
                <li>
                    <a class:selected=move || location.pathname.get() == "/active" href="/active">
                        "Active"
                    </a>
                </li>
                <li>
                    <a
                        class:selected=move || location.pathname.get() == "/completed"
                        href="/completed"
                    >
                        "Completed"
                    </a>
                </li>
            </ul>
            <button
                class="clear-completed"
                class:hidden=move || match todos.get() {
                    Some(Ok(todos)) => !todos.iter().any(|todo| todo.completed),
                    Some(Err(_)) | None => true,
                }
                on:click=move |_| {
                    clear_completed_pending.set(true);
                }
            >
                "Clear completed"
            </button>
        </footer>
    }
}

#[component]
pub fn TodoList() -> impl IntoView {
    let todos = all_todos_resource();

    view! {
        <ul class="todo-list">
            {move || match todos.get() {
                Some(Ok(todos)) => view! {
                    <For
                        each=move || todos.clone()
                        key=|todo| todo.id
                        children=move |todo| view! { <TodoItem todo/> }
                    />
                }
                .into_any(),
                Some(Err(_)) | None => ().into_any(),
            }}
        </ul>
    }
}

#[component]
fn TodoItem(todo: Todo) -> impl IntoView {
    let toggle_todo = ServerAction::<ToggleTodo>::new();
    let delete_todo = ServerAction::<DeleteTodo>::new();
    let edit_todo = ServerAction::<EditTodo>::new();
    let edit_input_ref = NodeRef::<html::Input>::new();
    let skip_blur_save = RwSignal::new(false);
    let original_title = todo.title.clone();
    let (is_editing, set_is_editing) = signal(false);
    let (draft_title, set_draft_title) = signal(original_title.clone());

    Effect::new({
        let toggle_todo = toggle_todo.clone();
        move |_| {
            if matches!(toggle_todo.value().get(), Some(Ok(_))) {
                refresh_todos();
            }
        }
    });

    Effect::new({
        let delete_todo = delete_todo.clone();
        move |_| {
            if matches!(delete_todo.value().get(), Some(Ok(_))) {
                refresh_todos();
            }
        }
    });

    Effect::new({
        let edit_todo = edit_todo.clone();
        move |_| {
            if matches!(edit_todo.value().get(), Some(Ok(_))) {
                refresh_todos();
            }
        }
    });

    Effect::new({
        let edit_input_ref = edit_input_ref.clone();
        move |_| {
            if is_editing.get() {
                if let Some(input) = edit_input_ref.get() {
                    let _ = input.focus();
                }
            }
        }
    });

    let toggle_id = todo.id;
    let delete_id = todo.id;
    let edit_id = todo.id;

    let save_edit = Callback::new({
        let original_title = original_title.clone();
        move |_| {
            let trimmed_title = draft_title.get_untracked().trim().to_string();
            set_is_editing.set(false);

            if trimmed_title.is_empty() {
                delete_todo.dispatch(DeleteTodo { id: delete_id });
            } else if trimmed_title != original_title {
                edit_todo.dispatch(EditTodo {
                    id: edit_id,
                    title: trimmed_title,
                });
            }
        }
    });

    let cancel_edit = Callback::new({
        let original_title = original_title.clone();
        move |_| {
            set_draft_title.set(original_title.clone());
            set_is_editing.set(false);
        }
    });

    view! {
        <li class:completed=todo.completed class:editing=move || is_editing.get()>
            <div class="view">
                <input
                    class="toggle"
                    type="checkbox"
                    prop:checked=todo.completed
                    on:change=move |_| {
                        toggle_todo.dispatch(ToggleTodo { id: toggle_id });
                    }
                />
                <label
                    on:dblclick=move |_| {
                        set_draft_title.set(original_title.clone());
                        set_is_editing.set(true);
                    }
                >
                    {todo.title}
                </label>
                <button
                    class="destroy"
                    on:click=move |_| {
                        delete_todo.dispatch(DeleteTodo { id: delete_id });
                    }
                ></button>
            </div>
            <input
                node_ref=edit_input_ref
                class="edit"
                prop:value=move || draft_title.get()
                on:input=move |event| {
                    set_draft_title.set(event_target_value(&event));
                }
                on:blur=move |_| {
                    if skip_blur_save.get_untracked() {
                        skip_blur_save.set(false);
                    } else {
                        save_edit.run(());
                    }
                }
                on:keydown=move |event: KeyboardEvent| match event.key().as_str() {
                    "Enter" => {
                        event.prevent_default();
                        skip_blur_save.set(true);
                        save_edit.run(());
                    }
                    "Escape" => {
                        event.prevent_default();
                        skip_blur_save.set(true);
                        cancel_edit.run(());
                    }
                    _ => {}
                }
            />
        </li>
    }
}

pub fn provide_todo_list_version() {
    provide_context(RwSignal::new(0_u64));
}

pub fn refresh_todos() {
    let todos_version = todo_list_version();
    todos_version.update(|version| *version += 1);
}

fn todo_list_version() -> RwSignal<u64> {
    use_context::<RwSignal<u64>>().expect("todo list version signal missing from Leptos context")
}

fn all_todos_resource() -> LocalResource<Result<Vec<Todo>, ServerFnError>> {
    let todos_version = todo_list_version();

    LocalResource::new(move || {
        let _ = todos_version.get();
        async move { list_todos(Filter::All).await }
    })
}
