use leptos::prelude::*;
use uuid::Uuid;

use crate::todo::{Filter, Todo};

#[server]
pub async fn list_todos(filter: Filter) -> Result<Vec<Todo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return repository_from_context()?
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

#[server]
pub async fn add_todo(title: String) -> Result<Todo, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let title = normalize_title(&title)?;

        return repository_from_context()?
            .create(&title)
            .await
            .map_err(Into::into);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = title;
        unreachable!("add_todo is only available on the server");
    }
}

#[server]
pub async fn toggle_todo(id: Uuid) -> Result<Option<Todo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let repository = repository_from_context()?;
        let Some(todo) = repository
            .get(id)
            .await
            .map_err(|error| ServerFnError::new(error.to_string()))?
        else {
            return Ok(None);
        };

        return repository
            .set_completed(id, !todo.completed)
            .await
            .map_err(Into::into);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = id;
        unreachable!("toggle_todo is only available on the server");
    }
}

#[server]
pub async fn edit_todo(id: Uuid, title: String) -> Result<Option<Todo>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let title = normalize_title(&title)?;

        return repository_from_context()?
            .update_title(id, &title)
            .await
            .map_err(Into::into);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = id;
        let _ = title;
        unreachable!("edit_todo is only available on the server");
    }
}

#[server]
pub async fn delete_todo(id: Uuid) -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return repository_from_context()?
            .delete(id)
            .await
            .map_err(Into::into);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = id;
        unreachable!("delete_todo is only available on the server");
    }
}

#[cfg(feature = "ssr")]
fn normalize_title(title: &str) -> Result<String, ServerFnError> {
    let trimmed = title.trim();

    if trimmed.is_empty() {
        Err(ServerFnError::new("todo title cannot be empty"))
    } else {
        Ok(trimmed.to_string())
    }
}

#[cfg(feature = "ssr")]
fn repository_from_context() -> Result<crate::todo::repository::TodoRepository, ServerFnError> {
    use sqlx::PgPool;

    let pool = use_context::<PgPool>()
        .ok_or_else(|| ServerFnError::new("database pool missing from Leptos context"))?;

    Ok(crate::todo::repository::TodoRepository::new(pool))
}
