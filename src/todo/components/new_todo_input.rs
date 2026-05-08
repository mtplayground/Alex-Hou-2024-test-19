use leptos::{ev::SubmitEvent, html, prelude::*};

use crate::todo::{refresh_todos, server::AddTodo};

#[component]
pub fn NewTodoInput() -> impl IntoView {
    let add_todo = ServerAction::<AddTodo>::new();
    let input_ref = NodeRef::<html::Input>::new();

    Effect::new({
        let input_ref = input_ref.clone();
        move |_| {
            if matches!(add_todo.value().get(), Some(Ok(_))) {
                if let Some(input) = input_ref.get() {
                    input.set_value("");
                }

                refresh_todos();
            }
        }
    });

    let on_submit = {
        let input_ref = input_ref.clone();

        move |event: SubmitEvent| {
            let Some(input) = input_ref.get() else {
                event.prevent_default();
                return;
            };

            let trimmed_title = input.value().trim().to_string();
            if trimmed_title.is_empty() {
                event.prevent_default();
                return;
            }

            input.set_value(&trimmed_title);
        }
    };

    view! {
        <ActionForm action=add_todo on:submit=on_submit>
            <input
                node_ref=input_ref
                class="new-todo"
                type="text"
                name="title"
                placeholder="What needs to be done?"
                autofocus
            />
        </ActionForm>
    }
}
