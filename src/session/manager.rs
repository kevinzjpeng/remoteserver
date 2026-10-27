use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, Utc};
use russh::client::Handle;
use tokio::sync::{broadcast, mpsc};

use crate::handlers::terminal::SshCommand;

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
    pub ssh_client: Arc<Mutex<Option<Handle<ClientHandler>>>>,
    pub command_sender: mpsc::Sender<SshCommand>,
    pub data_broadcaster: broadcast::Sender<Vec<u8>>,
    pub active_connections: Arc<Mutex<usize>>,
    pub output_buffer: Arc<Mutex<Vec<u8>>>,
}

pub struct ClientHandler;

#[async_trait::async_trait]
impl russh::client::Handler for ClientHandler {
    type Error = anyhow::Error;

    async fn check_server_key(
        self,
        _key: &russh_keys::key::PublicKey,
    ) -> Result<(Self, bool), Self::Error> {
        Ok((self, true))
    }
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
        let manager = Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            idle_timeout,
        };

        // Start cleanup task
        let sessions = Arc::clone(&manager.sessions);
        let idle_timeout = manager.idle_timeout;
        tokio::spawn(async move {
            Self::cleanup_task(sessions, idle_timeout).await;
        });

        manager
    }

    async fn cleanup_task(
        sessions: Arc<Mutex<HashMap<String, Arc<Session>>>>,
        idle_timeout: Duration,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;

            let mut sessions = sessions.lock().unwrap();
            let now = Utc::now();
            let mut to_remove = Vec::new();

            for (id, session) in sessions.iter() {
                let last_active = *session.last_active.lock().unwrap();
                let connections = *session.active_connections.lock().unwrap();

                if connections == 0
                    && now.signed_duration_since(last_active).to_std().unwrap() > idle_timeout
                {
                    to_remove.push(id.clone());
                }
            }

            for id in to_remove {
                tracing::info!("Cleaning up inactive session: {}", id);
                if let Some(session) = sessions.remove(&id) {
                    let _ = session.command_sender.try_send(SshCommand::Close);
                }
            }
        }
    }

    pub fn get_session(&self, server_id: i64, session_name: &str) -> Option<Arc<Session>> {
        let session_id = format!("{}:{}", server_id, session_name);
        let sessions = self.sessions.lock().unwrap();
        sessions.get(&session_id).cloned()
    }

    /// Store a new session. Returns Ok(session) if this is a new session,
    /// or Err(existing_session) if a session with this ID already exists.
    /// The caller is responsible for cleaning up the unused SSH connection if Err is returned.
    pub fn store_session(
        &self,
        server_id: i64,
        session_name: &str,
        ssh_client: Handle<ClientHandler>,
        command_sender: mpsc::Sender<SshCommand>,
        data_broadcaster: broadcast::Sender<Vec<u8>>,
    ) -> Result<Arc<Session>, Arc<Session>> {
        let session_id = format!("{}:{}", server_id, session_name);

        let mut sessions = self.sessions.lock().unwrap();

        // Check if session already exists (handles race condition where two
        // connections arrive simultaneously for the same session)
        if let Some(existing) = sessions.get(&session_id) {
            tracing::warn!("Session {} already exists, returning existing session", session_id);
            return Err(Arc::clone(existing));
        }

        let session = Arc::new(Session {
            id: session_id.clone(),
            server_id,
            name: session_name.to_string(),
            created_at: Utc::now(),
            last_active: Arc::new(Mutex::new(Utc::now())),
            ssh_client: Arc::new(Mutex::new(Some(ssh_client))),
            command_sender,
            data_broadcaster,
            active_connections: Arc::new(Mutex::new(0)),
            output_buffer: Arc::new(Mutex::new(Vec::new())),
        });

        sessions.insert(session_id, Arc::clone(&session));
        tracing::info!("Stored SSH session: {}", session.id);
        Ok(session)
    }

    pub fn increment_connections(&self, server_id: i64, session_name: &str) {
        let session_id = format!("{}:{}", server_id, session_name);
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(&session_id) {
            *session.active_connections.lock().unwrap() += 1;
            *session.last_active.lock().unwrap() = Utc::now();
        }
    }

    pub fn decrement_connections(&self, server_id: i64, session_name: &str) {
        let session_id = format!("{}:{}", server_id, session_name);
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(&session_id) {
            let mut count = session.active_connections.lock().unwrap();
            if *count > 0 {
                *count -= 1;
            }
            *session.last_active.lock().unwrap() = Utc::now();
        }
    }

    pub fn close_session(&self, server_id: i64, session_name: &str) -> Result<(), String> {
        let session_id = format!("{}:{}", server_id, session_name);
        let mut sessions = self.sessions.lock().unwrap();

        if let Some(session) = sessions.remove(&session_id) {
            let _ = session.command_sender.try_send(SshCommand::Close);
            tracing::info!("Closed SSH session: {}", session_id);
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }

    pub fn list_sessions(&self, server_id: i64) -> Vec<SessionInfo> {
        let sessions = self.sessions.lock().unwrap();
        let prefix = format!("{}:", server_id);

        // First, collect session references while holding the sessions lock
        let session_refs: Vec<_> = sessions
            .values()
            .filter(|s| s.id.starts_with(&prefix))
            .cloned()
            .collect();

        // Release the sessions lock before acquiring individual session locks
        drop(sessions);

        // Now safely acquire individual locks without risk of deadlock
        session_refs
            .into_iter()
            .map(|s| SessionInfo {
                id: s.id.clone(),
                name: s.name.clone(),
                created_at: s.created_at,
                last_active: *s.last_active.lock().unwrap(),
                client_count: *s.active_connections.lock().unwrap(),
                active: *s.active_connections.lock().unwrap() > 0,
            })
            .collect()
    }

    const BUFFER_SIZE: usize = 65536; // 64KB

    pub fn append_output(&self, server_id: i64, session_name: &str, data: &[u8]) {
        let session_id = format!("{}:{}", server_id, session_name);
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(&session_id) {
            // Clone the reference to release sessions lock before acquiring output_buffer lock
            let output_buffer = Arc::clone(&session.output_buffer);
            drop(sessions);
            let mut buf = output_buffer.lock().unwrap();
            buf.extend_from_slice(data);
            // Trim from front if over capacity
            if buf.len() > Self::BUFFER_SIZE {
                let excess = buf.len() - Self::BUFFER_SIZE;
                buf.drain(..excess);
            }
        }
    }

    pub fn get_buffer(&self, server_id: i64, session_name: &str) -> Vec<u8> {
        let session_id = format!("{}:{}", server_id, session_name);
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(&session_id) {
            let output_buffer = Arc::clone(&session.output_buffer);
            drop(sessions);
            let result = output_buffer.lock().unwrap().clone();
            result
        } else {
            Vec::new()
        }
    }
}
