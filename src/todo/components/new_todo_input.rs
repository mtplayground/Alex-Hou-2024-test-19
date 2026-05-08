use leptos::prelude::*;

#[component]
pub fn NewTodoInput() -> impl IntoView {
    view! {
        <form action="/todos" method="post">
            <input
                class="new-todo"
                type="text"
                name="title"
                placeholder="What needs to be done?"
                autofocus
            />
        </form>
    }
}
