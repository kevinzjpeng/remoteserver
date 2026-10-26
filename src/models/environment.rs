use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub display_order: i32,
    pub created_at: NaiveDateTime,
}
