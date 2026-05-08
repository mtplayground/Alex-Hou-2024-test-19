use sqlx::{
    types::{
        chrono::{DateTime, Utc},
    },
    PgPool,
};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TodoFilter {
    All,
    Active,
    Completed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TodoRecord {
    pub id: Uuid,
    pub title: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TodoRepository {
    pool: PgPool,
}

impl TodoRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, filter: TodoFilter) -> Result<Vec<TodoRecord>, sqlx::Error> {
        match filter {
            TodoFilter::All => {
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
            TodoFilter::Active => {
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
            TodoFilter::Completed => {
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

    pub async fn create(&self, title: &str) -> Result<TodoRecord, sqlx::Error> {
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
    ) -> Result<Option<TodoRecord>, sqlx::Error> {
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

    pub async fn update_title(
        &self,
        id: Uuid,
        title: &str,
    ) -> Result<Option<TodoRecord>, sqlx::Error> {
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

fn map_todo_row(id: Uuid, title: String, completed: bool, created_at: DateTime<Utc>) -> TodoRecord {
    TodoRecord {
        id,
        title,
        completed,
        created_at,
    }
}
