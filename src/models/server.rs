use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: i64,
    pub name: String,
    pub hostname: String,
    pub port: i32,
    pub username: String,
    pub environment_id: Option<i64>,

    pub auth_type: String,
    #[serde(skip_serializing)]
    pub password_encrypted: Option<Vec<u8>>,
    #[serde(skip_serializing)]
    pub private_key_encrypted: Option<Vec<u8>>,
    #[serde(skip_serializing)]
    pub passphrase_encrypted: Option<Vec<u8>>,

    pub description: Option<String>,
    pub os_type: String,
    pub tags: Option<String>,
    pub notes: Option<String>,

    pub status: String,
    pub is_intranet: bool,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub last_accessed: Option<NaiveDateTime>,
}
