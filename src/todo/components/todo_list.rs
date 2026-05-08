use leptos::prelude::*;

use crate::todo::{
    server::{list_todos, DeleteTodo, ToggleTodo},
    Filter, Todo,
};

#[component]
pub fn TodoList() -> impl IntoView {
    let todos_version = todo_list_version();
    let todos = Resource::new(
        move || todos_version.get(),
        move |_| async move { list_todos(Filter::All).await },
    );

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

    let toggle_id = todo.id;
    let delete_id = todo.id;

    view! {
        <li class:completed=todo.completed>
            <div class="view">
                <input
                    class="toggle"
                    type="checkbox"
                    prop:checked=todo.completed
                    on:change=move |_| {
                        toggle_todo.dispatch(ToggleTodo { id: toggle_id });
                    }
                />
                <label>{todo.title}</label>
                <button
                    class="destroy"
                    on:click=move |_| {
                        delete_todo.dispatch(DeleteTodo { id: delete_id });
                    }
                ></button>
            </div>
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
