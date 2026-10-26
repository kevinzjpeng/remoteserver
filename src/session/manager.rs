use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, Utc};

pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, Arc<Session>>>>,
    idle_timeout: Duration,
}

pub struct Session {
    pub id: String,
    pub server_id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_active: Arc<Mutex<DateTime<Utc>>>,
}

pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub client_count: usize,
    pub active: bool,
}

impl SessionManager {
    pub fn new(idle_timeout: Duration) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            idle_timeout,
        }
    }

    pub fn get_or_create_session(&self, server_id: i64, session_name: &str) -> Arc<Session> {
        let session_id = format!("{}:{}", server_id, session_name);

        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(&session_id) {
            return Arc::clone(session);
        }

        let session = Arc::new(Session {
            id: session_id.clone(),
            server_id,
            name: session_name.to_string(),
            created_at: Utc::now(),
            last_active: Arc::new(Mutex::new(Utc::now())),
        });

        sessions.insert(session_id, Arc::clone(&session));
        session
    }

    pub fn close_session(&self, server_id: i64, session_name: &str) -> Result<(), String> {
        let session_id = format!("{}:{}", server_id, session_name);
        let mut sessions = self.sessions.lock().unwrap();

        if sessions.remove(&session_id).is_some() {
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }

    pub fn list_sessions(&self, server_id: i64) -> Vec<SessionInfo> {
        let sessions = self.sessions.lock().unwrap();
        let prefix = format!("{}:", server_id);

        sessions
            .values()
            .filter(|s| s.id.starts_with(&prefix))
            .map(|s| SessionInfo {
                id: s.id.clone(),
                name: s.name.clone(),
                created_at: s.created_at,
                last_active: *s.last_active.lock().unwrap(),
                client_count: 0,
                active: true,
            })
            .collect()
    }
}
