use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Filter {
    All,
    Active,
    Completed,
}
