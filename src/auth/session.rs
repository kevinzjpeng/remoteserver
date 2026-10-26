use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: i64,
    pub username: String,
    pub is_admin: bool,
    pub created_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
}

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Invalid session")]
    Invalid,
    #[error("Session expired")]
    Expired,
    #[error("Decode error: {0}")]
    Decode(String),
}

#[derive(Clone)]
pub struct SessionManager {
    secret: String,
    duration: Duration,
}

impl SessionManager {
    pub fn new(secret: String, duration_secs: i64) -> Self {
        Self {
            secret,
            duration: Duration::seconds(duration_secs),
        }
    }

    pub fn create_session(&self, user_id: i64, username: String, is_admin: bool) -> String {
        let now = Utc::now().naive_utc();
        let session = SessionData {
            user_id,
            username,
            is_admin,
            created_at: now,
            expires_at: now + self.duration,
        };

        let json = serde_json::to_string(&session).unwrap();
        BASE64.encode(json.as_bytes())
    }

    pub fn get_session(&self, cookie_value: &str) -> Result<SessionData, SessionError> {
        let decoded = BASE64
            .decode(cookie_value)
            .map_err(|e| SessionError::Decode(e.to_string()))?;

        let session: SessionData =
            serde_json::from_slice(&decoded).map_err(|e| SessionError::Decode(e.to_string()))?;

        let now = Utc::now().naive_utc();
        if now > session.expires_at {
            return Err(SessionError::Expired);
        }

        Ok(session)
    }
}
