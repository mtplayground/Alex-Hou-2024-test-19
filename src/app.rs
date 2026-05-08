use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

use crate::todo::{provide_todo_list_version, NewTodoInput, TodoList};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <title>"Leptos TodoMVC"</title>
                <link rel="stylesheet" href="/node_modules/todomvc-common/base.css"/>
                <link rel="stylesheet" href="/node_modules/todomvc-app-css/index.css"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    provide_todo_list_version();

    view! {
        <section class="todoapp">
            <header class="header">
                <h1>"todos"</h1>
                <NewTodoInput/>
            </header>
            <section class="main">
                <input id="toggle-all" class="toggle-all" type="checkbox"/>
                <label for="toggle-all">"Mark all as complete"</label>
                <TodoList/>
            </section>
            <footer class="footer">
                <span class="todo-count">
                    <strong>"0"</strong>
                    " item left"
                </span>
                <ul class="filters">
                    <li>
                        <a class="selected" href="#/">"All"</a>
                    </li>
                    <li>
                        <a href="#/active">"Active"</a>
                    </li>
                    <li>
                        <a href="#/completed">"Completed"</a>
                    </li>
                </ul>
                <button class="clear-completed">"Clear completed"</button>
            </footer>
        </section>
        <footer class="info">
            <p>"Double-click to edit a todo"</p>
            <p>
                "Created by "
                <a href="http://todomvc.com">"TodoMVC template"</a>
            </p>
            <p>
                "Part of "
                <a href="http://todomvc.com">"TodoMVC"</a>
            </p>
        </footer>
    }
}
