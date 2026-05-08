#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use alex_hou_2024_test_19::{
        app::shell,
        config::AppConfig,
        db::create_pool,
        todo::{repository::TodoRepository, Filter, Todo},
    };
    use axum::{
        extract::{Form, FromRef, State},
        response::{Html, Redirect},
        routing::{get, post},
        Router,
    };
    use serde::Deserialize;
    use tracing::info;

    #[derive(Clone)]
    struct AppState {
        leptos_options: leptos::config::LeptosOptions,
        pool: sqlx::PgPool,
    }

    impl FromRef<AppState> for leptos::config::LeptosOptions {
        fn from_ref(state: &AppState) -> Self {
            state.leptos_options.clone()
        }
    }

    #[derive(Deserialize)]
    struct AddTodoForm {
        title: String,
    }

    fn escape_html(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    fn selected_class(current: Filter, target: Filter) -> &'static str {
        if current == target { "selected" } else { "" }
    }

    fn render_page(filter: Filter, todos: &[Todo]) -> String {
        let visible_todos: Vec<&Todo> = todos
            .iter()
            .filter(|todo| match filter {
                Filter::All => true,
                Filter::Active => !todo.completed,
                Filter::Completed => todo.completed,
            })
            .collect();
        let active_count = todos.iter().filter(|todo| !todo.completed).count();
        let any_completed = todos.iter().any(|todo| todo.completed);
        let main_hidden = if todos.is_empty() { " hidden" } else { "" };
        let footer_hidden = if todos.is_empty() { " hidden" } else { "" };
        let clear_hidden = if any_completed { "" } else { " hidden" };
        let item_word = if active_count == 1 { "item" } else { "items" };
        let list_items = visible_todos
            .into_iter()
            .map(|todo| {
                let completed_class = if todo.completed { " class=\"completed\"" } else { "" };
                let checked = if todo.completed { " checked" } else { "" };
                format!(
                    "<li{completed_class}><div class=\"view\"><input class=\"toggle\" type=\"checkbox\" disabled{checked}><label>{}</label><button class=\"destroy\" disabled></button></div></li>",
                    escape_html(&todo.title)
                )
            })
            .collect::<Vec<_>>()
            .join("");

        format!(
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>Leptos TodoMVC</title><link rel=\"stylesheet\" href=\"/node_modules/todomvc-common/base.css\"><link rel=\"stylesheet\" href=\"/node_modules/todomvc-app-css/index.css\"></head><body><main><section class=\"todoapp\"><header class=\"header\"><h1>todos</h1><form action=\"/todos\" method=\"post\"><input class=\"new-todo\" type=\"text\" name=\"title\" placeholder=\"What needs to be done?\" autofocus></form></header><section class=\"main{main_hidden}\"><input id=\"toggle-all\" class=\"toggle-all\" type=\"checkbox\" disabled><label for=\"toggle-all\">Mark all as complete</label><ul class=\"todo-list\">{list_items}</ul></section><footer class=\"footer{footer_hidden}\"><span class=\"todo-count\"><strong>{active_count}</strong> {item_word} left</span><ul class=\"filters\"><li><a href=\"/\" class=\"{}\">All</a></li><li><a href=\"/active\" class=\"{}\">Active</a></li><li><a href=\"/completed\" class=\"{}\">Completed</a></li></ul><button class=\"clear-completed{clear_hidden}\" disabled>Clear completed</button></footer></section><footer class=\"info\"><p>Double-click to edit a todo</p><p>Created by <a href=\"http://todomvc.com\">TodoMVC template</a></p><p>Part of <a href=\"http://todomvc.com\">TodoMVC</a></p></footer></main></body></html>",
            selected_class(filter, Filter::All),
            selected_class(filter, Filter::Active),
            selected_class(filter, Filter::Completed),
        )
    }

    async fn todos_page(State(state): State<AppState>, filter: Filter) -> Html<String> {
        let repository = TodoRepository::new(state.pool);
        let todos = repository.list(Filter::All).await.unwrap_or_default();
        Html(render_page(filter, &todos))
    }

    async fn root_handler(State(state): State<AppState>) -> Html<String> {
        todos_page(State(state), Filter::All).await
    }

    async fn active_handler(State(state): State<AppState>) -> Html<String> {
        todos_page(State(state), Filter::Active).await
    }

    async fn completed_handler(State(state): State<AppState>) -> Html<String> {
        todos_page(State(state), Filter::Completed).await
    }

    async fn add_todo_handler(
        State(state): State<AppState>,
        Form(form): Form<AddTodoForm>,
    ) -> Redirect {
        let title = form.title.trim();
        if title.is_empty() {
            return Redirect::to("/");
        }

        let repository = TodoRepository::new(state.pool);
        let _ = repository.create(title).await;
        Redirect::to("/")
    }

    let config = AppConfig::load()?;
    config.init_tracing()?;
    let pool = create_pool(&config.database_url).await?;

    let addr = config.leptos_options.site_addr;
    let database_url_configured = !config.database_url.is_empty();
    let leptos_options = config.leptos_options;

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        pool: pool.clone(),
    };

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/active", get(active_handler))
        .route("/completed", get(completed_handler))
        .route("/todos", post(add_todo_handler))
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .with_state(app_state);

    info!(
        site_addr = %addr,
        database_url_configured,
        "starting leptos axum server"
    );
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[cfg(not(feature = "ssr"))]
fn main() {}
