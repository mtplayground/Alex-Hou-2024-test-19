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

#[server]
pub async fn toggle_all() -> Result<u64, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let repository = repository_from_context()?;
        let todos = repository
            .list(Filter::All)
            .await
            .map_err(|error| ServerFnError::new(error.to_string()))?;
        let target_state = !todos.iter().all(|todo| todo.completed);

        return repository
            .toggle_all(target_state)
            .await
            .map_err(Into::into);
    }

    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("toggle_all is only available on the server");
    }
}

#[server]
pub async fn clear_completed() -> Result<u64, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return repository_from_context()?
            .clear_completed()
            .await
            .map_err(Into::into);
    }

    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("clear_completed is only available on the server");
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

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::{
        add_todo, clear_completed, delete_todo, edit_todo, list_todos, toggle_all, toggle_todo,
    };
    use crate::todo::Filter;
    use leptos::prelude::{provide_context, Owner};
    use serial_test::serial;
    use sqlx::{postgres::PgPoolOptions, Executor, PgPool};
    use uuid::Uuid;

    struct TestHarness {
        admin_pool: PgPool,
        pool: PgPool,
        schema: String,
    }

    impl TestHarness {
        async fn new() -> Self {
            let database_url =
                std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for server tests");
            let admin_pool = PgPoolOptions::new()
                .max_connections(1)
                .connect(&database_url)
                .await
                .expect("admin pool should connect");
            let schema = format!("todo_server_test_{}", Uuid::new_v4().simple());

            sqlx::query(&format!(r#"CREATE SCHEMA "{schema}""#))
                .execute(&admin_pool)
                .await
                .expect("test schema should be created");

            let search_path = format!(r#"SET search_path TO "{schema}", public"#);
            let pool = PgPoolOptions::new()
                .max_connections(1)
                .after_connect(move |connection, _meta| {
                    let search_path = search_path.clone();
                    Box::pin(async move {
                        connection.execute(search_path.as_str()).await?;
                        Ok(())
                    })
                })
                .connect(&database_url)
                .await
                .expect("test pool should connect");

            sqlx::migrate!()
                .run(&pool)
                .await
                .expect("migrations should run");

            Self {
                admin_pool,
                pool,
                schema,
            }
        }

        async fn cleanup(self) {
            self.pool.close().await;
            sqlx::query(&format!(
                r#"DROP SCHEMA "{schema}" CASCADE"#,
                schema = self.schema
            ))
            .execute(&self.admin_pool)
            .await
            .expect("test schema should be dropped");
            self.admin_pool.close().await;
        }
    }

    fn owner_with_pool(pool: PgPool) -> Owner {
        let owner = Owner::new();
        owner.set();
        provide_context(pool);
        owner
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn add_edit_list_functions_trim_and_validate_titles() {
        let harness = TestHarness::new().await;
        let _owner = owner_with_pool(harness.pool.clone());

        let added = add_todo("  write server tests  ".to_string())
            .await
            .expect("todo should add");
        assert_eq!(added.title, "write server tests");

        let empty_add = add_todo("   ".to_string())
            .await
            .expect_err("blank title should fail");
        assert!(empty_add.to_string().contains("todo title cannot be empty"));

        let edited = edit_todo(added.id, "  trimmed title  ".to_string())
            .await
            .expect("edit should succeed")
            .expect("todo should exist");
        assert_eq!(edited.title, "trimmed title");

        let empty_edit = edit_todo(added.id, "   ".to_string())
            .await
            .expect_err("blank edit should fail");
        assert!(empty_edit
            .to_string()
            .contains("todo title cannot be empty"));

        let listed = list_todos(Filter::All).await.expect("todos should list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].title, "trimmed title");

        harness.cleanup().await;
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn toggle_and_delete_functions_cover_existing_and_missing_rows() {
        let harness = TestHarness::new().await;
        let _owner = owner_with_pool(harness.pool.clone());

        let added = add_todo("toggle me".to_string())
            .await
            .expect("todo should add");

        let toggled_once = toggle_todo(added.id)
            .await
            .expect("toggle should succeed")
            .expect("todo should exist");
        assert!(toggled_once.completed);

        let toggled_twice = toggle_todo(added.id)
            .await
            .expect("toggle should succeed")
            .expect("todo should exist");
        assert!(!toggled_twice.completed);

        assert!(delete_todo(added.id).await.expect("delete should succeed"));
        assert!(!delete_todo(added.id)
            .await
            .expect("repeat delete should return false"));
        assert!(toggle_todo(added.id)
            .await
            .expect("missing toggle should not fail")
            .is_none());

        harness.cleanup().await;
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn bulk_server_functions_use_current_state_and_filters() {
        let harness = TestHarness::new().await;
        let _owner = owner_with_pool(harness.pool.clone());

        let first = add_todo("first".to_string())
            .await
            .expect("todo should add");
        let second = add_todo("second".to_string())
            .await
            .expect("todo should add");
        let third = add_todo("third".to_string())
            .await
            .expect("todo should add");

        let _ = second;
        toggle_todo(third.id).await.expect("toggle should succeed");

        let active_before = list_todos(Filter::Active)
            .await
            .expect("active todos should list");
        let completed_before = list_todos(Filter::Completed)
            .await
            .expect("completed todos should list");
        assert_eq!(active_before.len(), 2);
        assert_eq!(completed_before.len(), 1);

        let mixed_toggle = toggle_all().await.expect("toggle_all should succeed");
        assert_eq!(mixed_toggle, 2);
        assert!(list_todos(Filter::All)
            .await
            .expect("todos should list")
            .iter()
            .all(|todo| todo.completed));

        let all_complete_toggle = toggle_all().await.expect("toggle_all should succeed");
        assert_eq!(all_complete_toggle, 3);
        assert!(list_todos(Filter::All)
            .await
            .expect("todos should list")
            .iter()
            .all(|todo| !todo.completed));

        toggle_todo(first.id).await.expect("toggle should succeed");
        let cleared = clear_completed()
            .await
            .expect("clear_completed should succeed");
        assert_eq!(cleared, 1);
        assert_eq!(
            list_todos(Filter::All)
                .await
                .expect("todos should list")
                .len(),
            2
        );

        harness.cleanup().await;
    }
}
