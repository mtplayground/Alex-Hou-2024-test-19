use leptos::{ev::KeyboardEvent, html, prelude::*, task::spawn_local};
use leptos_router::components::A;

use crate::todo::{
    server::{clear_completed, list_todos, toggle_all, DeleteTodo, EditTodo, ToggleTodo},
    Filter, Todo,
};

type AllTodosResource = Resource<Result<Vec<Todo>, ServerFnError>>;
type VisibleTodosResource = Resource<Result<Vec<Todo>, ServerFnError>>;

#[derive(Clone)]
struct TodoState {
    filter: Signal<Filter>,
    refresh_version: RwSignal<u64>,
    all_todos: AllTodosResource,
    visible_todos: VisibleTodosResource,
}

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
    let filter = todo_filter();
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
                    <A
                        href="/"
                        attr:class=move || if filter.get() == Filter::All { "selected" } else { "" }
                    >
                        "All"
                    </A>
                </li>
                <li>
                    <A
                        href="/active"
                        attr:class=move || if filter.get() == Filter::Active { "selected" } else { "" }
                    >
                        "Active"
                    </A>
                </li>
                <li>
                    <A
                        href="/completed"
                        attr:class=move || if filter.get() == Filter::Completed { "selected" } else { "" }
                    >
                        "Completed"
                    </A>
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
    let todos = visible_todos_resource();

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
                    let _ = input.set_selection_range(0, input.value().len() as u32);
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

pub fn provide_todo_state(filter: Signal<Filter>) {
    let refresh_version = RwSignal::new(0_u64);
    let all_todos = Resource::new(
        move || refresh_version.get(),
        move |_| async move { list_todos(Filter::All).await },
    );
    let visible_todos = Resource::new(
        move || (refresh_version.get(), filter.get()),
        move |(_, filter)| async move { list_todos(filter).await },
    );

    provide_context(TodoState {
        filter,
        refresh_version,
        all_todos,
        visible_todos,
    });
}

pub fn refresh_todos() {
    let state = todo_state();
    state.refresh_version.update(|version| *version += 1);
}

fn todo_filter() -> Signal<Filter> {
    todo_state().filter
}

fn todo_state() -> TodoState {
    use_context::<TodoState>().expect("todo state missing from Leptos context")
}

fn all_todos_resource() -> AllTodosResource {
    todo_state().all_todos
}

fn visible_todos_resource() -> VisibleTodosResource {
    todo_state().visible_todos
}
