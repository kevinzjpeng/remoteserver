use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub id: i64,
    pub name: String,
    pub service_type: String,
    #[serde(skip_serializing)]
    pub credentials_encrypted: Option<Vec<u8>>,
    pub config_json: Option<String>,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
