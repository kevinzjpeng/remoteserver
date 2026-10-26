use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessLog {
    pub id: i64,
    pub server_id: i64,
    pub user_id: i64,
    pub accessed_at: NaiveDateTime,
    pub connection_type: String,
    pub success: bool,
    pub notes: Option<String>,
}
