use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::todo::{Filter, Todo};

#[derive(Clone)]
pub struct TodoRepository {
    pool: PgPool,
}

impl TodoRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Todo>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id, title, completed, created_at
            FROM todos
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| map_todo_row(row.id, row.title, row.completed, row.created_at)))
    }

    pub async fn list(&self, filter: Filter) -> Result<Vec<Todo>, sqlx::Error> {
        match filter {
            Filter::All => {
                let rows = sqlx::query!(
                    r#"
                    SELECT id, title, completed, created_at
                    FROM todos
                    ORDER BY created_at ASC, id ASC
                    "#
                )
                .fetch_all(&self.pool)
                .await?;

                Ok(rows
                    .into_iter()
                    .map(|row| map_todo_row(row.id, row.title, row.completed, row.created_at))
                    .collect())
            }
            Filter::Active => {
                let rows = sqlx::query!(
                    r#"
                    SELECT id, title, completed, created_at
                    FROM todos
                    WHERE completed = FALSE
                    ORDER BY created_at ASC, id ASC
                    "#
                )
                .fetch_all(&self.pool)
                .await?;

                Ok(rows
                    .into_iter()
                    .map(|row| map_todo_row(row.id, row.title, row.completed, row.created_at))
                    .collect())
            }
            Filter::Completed => {
                let rows = sqlx::query!(
                    r#"
                    SELECT id, title, completed, created_at
                    FROM todos
                    WHERE completed = TRUE
                    ORDER BY created_at ASC, id ASC
                    "#
                )
                .fetch_all(&self.pool)
                .await?;

                Ok(rows
                    .into_iter()
                    .map(|row| map_todo_row(row.id, row.title, row.completed, row.created_at))
                    .collect())
            }
        }
    }

    pub async fn create(&self, title: &str) -> Result<Todo, sqlx::Error> {
        let id = Uuid::new_v4();
        let row = sqlx::query!(
            r#"
            INSERT INTO todos (id, title)
            VALUES ($1, $2)
            RETURNING id, title, completed, created_at
            "#,
            id,
            title
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(map_todo_row(
            row.id,
            row.title,
            row.completed,
            row.created_at,
        ))
    }

    pub async fn set_completed(
        &self,
        id: Uuid,
        completed: bool,
    ) -> Result<Option<Todo>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            UPDATE todos
            SET completed = $2
            WHERE id = $1
            RETURNING id, title, completed, created_at
            "#,
            id,
            completed
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| map_todo_row(row.id, row.title, row.completed, row.created_at)))
    }

    pub async fn update_title(&self, id: Uuid, title: &str) -> Result<Option<Todo>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            UPDATE todos
            SET title = $2
            WHERE id = $1
            RETURNING id, title, completed, created_at
            "#,
            id,
            title
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| map_todo_row(row.id, row.title, row.completed, row.created_at)))
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM todos
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    pub async fn toggle_all(&self, target_state: bool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            UPDATE todos
            SET completed = $1
            WHERE completed <> $1
            "#,
            target_state
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn clear_completed(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM todos
            WHERE completed = TRUE
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

fn map_todo_row(id: Uuid, title: String, completed: bool, created_at: DateTime<Utc>) -> Todo {
    Todo {
        id,
        title,
        completed,
        created_at,
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::TodoRepository;
    use crate::todo::Filter;
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
            let database_url = std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set for repository tests");
            let admin_pool = PgPoolOptions::new()
                .max_connections(1)
                .connect(&database_url)
                .await
                .expect("admin pool should connect");
            let schema = format!("todo_repo_test_{}", Uuid::new_v4().simple());

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

        fn repository(&self) -> TodoRepository {
            TodoRepository::new(self.pool.clone())
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

    #[tokio::test]
    #[serial]
    async fn create_get_and_list_cover_filters_and_empty_title() {
        let harness = TestHarness::new().await;
        let repository = harness.repository();

        let active = repository
            .create("write tests")
            .await
            .expect("active todo should be created");
        let empty = repository
            .create("")
            .await
            .expect("repository should allow empty titles");
        let completed = repository
            .create("ship feature")
            .await
            .expect("completed todo should be created");
        repository
            .set_completed(completed.id, true)
            .await
            .expect("completed todo should update");

        let fetched = repository
            .get(active.id)
            .await
            .expect("todo should fetch")
            .expect("created todo should exist");
        assert_eq!(fetched.title, "write tests");
        assert!(!fetched.completed);

        let all = repository
            .list(Filter::All)
            .await
            .expect("all todos should list");
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].id, active.id);
        assert_eq!(all[1].id, empty.id);
        assert_eq!(all[1].title, "");
        assert_eq!(all[2].id, completed.id);

        let active_only = repository
            .list(Filter::Active)
            .await
            .expect("active todos should list");
        assert_eq!(active_only.len(), 2);
        assert!(active_only.iter().all(|todo| !todo.completed));
        assert_eq!(active_only[0].id, active.id);
        assert_eq!(active_only[1].id, empty.id);

        let completed_only = repository
            .list(Filter::Completed)
            .await
            .expect("completed todos should list");
        assert_eq!(completed_only.len(), 1);
        assert_eq!(completed_only[0].id, completed.id);
        assert!(completed_only[0].completed);

        harness.cleanup().await;
    }

    #[tokio::test]
    #[serial]
    async fn update_delete_and_clear_completed_cover_missing_rows() {
        let harness = TestHarness::new().await;
        let repository = harness.repository();

        let keep = repository
            .create("keep me")
            .await
            .expect("todo should be created");
        let remove = repository
            .create("remove me")
            .await
            .expect("todo should be created");
        repository
            .set_completed(remove.id, true)
            .await
            .expect("todo should be completable");

        let updated = repository
            .update_title(keep.id, "keep me updated")
            .await
            .expect("title update should succeed")
            .expect("todo should still exist");
        assert_eq!(updated.title, "keep me updated");

        let cleared = repository
            .clear_completed()
            .await
            .expect("completed todos should clear");
        assert_eq!(cleared, 1);

        let remaining = repository
            .list(Filter::All)
            .await
            .expect("todos should list");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, keep.id);

        assert!(repository
            .delete(keep.id)
            .await
            .expect("delete should succeed"));
        assert!(!repository
            .delete(keep.id)
            .await
            .expect("second delete should not fail"));
        assert!(repository
            .get(remove.id)
            .await
            .expect("cleared todo lookup should succeed")
            .is_none());

        harness.cleanup().await;
    }

    #[tokio::test]
    #[serial]
    async fn toggle_all_handles_mixed_and_all_complete_sets() {
        let harness = TestHarness::new().await;
        let repository = harness.repository();

        let first = repository
            .create("first")
            .await
            .expect("todo should be created");
        let second = repository
            .create("second")
            .await
            .expect("todo should be created");
        let third = repository
            .create("third")
            .await
            .expect("todo should be created");

        repository
            .set_completed(second.id, true)
            .await
            .expect("todo should be completable");

        let mixed_toggled = repository
            .toggle_all(true)
            .await
            .expect("toggle_all(true) should succeed");
        assert_eq!(mixed_toggled, 2);
        assert!(repository
            .list(Filter::All)
            .await
            .expect("todos should list")
            .iter()
            .all(|todo| todo.completed));

        let all_complete_toggled = repository
            .toggle_all(false)
            .await
            .expect("toggle_all(false) should succeed");
        assert_eq!(all_complete_toggled, 3);
        assert!(repository
            .list(Filter::All)
            .await
            .expect("todos should list")
            .iter()
            .all(|todo| !todo.completed));

        let idempotent = repository
            .toggle_all(false)
            .await
            .expect("repeat toggle_all(false) should succeed");
        assert_eq!(idempotent, 0);

        let _ = (first, third);
        harness.cleanup().await;
    }
}
