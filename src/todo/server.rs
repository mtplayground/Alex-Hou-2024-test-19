use leptos::prelude::*;

use crate::todo::{Filter, Todo};

#[server]
pub async fn list_todos(filter: Filter) -> Result<Vec<Todo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::PgPool;

        use crate::todo::repository::TodoRepository;

        let pool = use_context::<PgPool>()
            .ok_or_else(|| ServerFnError::new("database pool missing from Leptos context"))?;

        return TodoRepository::new(pool)
            .list(filter)
            .await
            .map_err(Into::into);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = filter;
        unreachable!("list_todos is only available on the server");
    }
}
