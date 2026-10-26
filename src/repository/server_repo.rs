use crate::crypto::encryption;
use crate::database::Database;
use crate::models::Server;
use rusqlite::params;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerRepoError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Server not found")]
    NotFound,
    #[error("Encryption error: {0}")]
    Encryption(String),
}

pub struct ServerRepository {
    db: Arc<Database>,
    encryption_key: Vec<u8>,
}

impl ServerRepository {
    pub fn new(db: Arc<Database>, encryption_key: Vec<u8>) -> Self {
        Self { db, encryption_key }
    }

    fn encrypt_field(&self, value: &str) -> Option<Vec<u8>> {
        if value.is_empty() {
            return None;
        }
        encryption::encrypt(value.as_bytes(), &self.encryption_key).ok()
    }

    fn decrypt_field(&self, data: &Option<Vec<u8>>) -> Result<String, ServerRepoError> {
        match data {
            None => Ok(String::new()),
            Some(bytes) if bytes.is_empty() => Ok(String::new()),
            Some(bytes) => {
                let decrypted = encryption::decrypt(bytes, &self.encryption_key)
                    .map_err(|e| ServerRepoError::Encryption(e.to_string()))?;
                Ok(String::from_utf8_lossy(&decrypted).to_string())
            }
        }
    }

    pub fn get_all(&self) -> Result<Vec<Server>, ServerRepoError> {
        let conn = self.db.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT s.id, s.name, s.hostname, s.port, s.username, s.environment_id,
                        s.auth_type, s.description, s.os_type, s.tags, s.notes,
                        s.status, s.is_intranet,
                        s.created_at, s.updated_at, s.last_accessed
                 FROM servers s
                 ORDER BY s.name",
            )
            .map_err(|e| ServerRepoError::Database(e.to_string()))?;

        let servers = stmt
            .query_map([], |row| {
                Ok(Server {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    hostname: row.get(2)?,
                    port: row.get(3)?,
                    username: row.get(4)?,
                    environment_id: row.get(5)?,
                    auth_type: row.get::<_, Option<String>>(6)?.unwrap_or_else(|| "password".to_string()),
                    password_encrypted: None,
                    private_key_encrypted: None,
                    passphrase_encrypted: None,
                    description: row.get(7)?,
                    os_type: row.get::<_, Option<String>>(8)?.unwrap_or_else(|| "linux".to_string()),
                    tags: row.get(9)?,
                    notes: row.get(10)?,
                    status: row.get::<_, Option<String>>(11)?.unwrap_or_else(|| "unknown".to_string()),
                    is_intranet: row.get(12)?,
                    created_at: row.get(13)?,
                    updated_at: row.get(14)?,
                    last_accessed: row.get(15)?,
                })
            })
            .map_err(|e| ServerRepoError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ServerRepoError::Database(e.to_string()))?;

        Ok(servers)
    }

    pub fn get_by_id(&self, id: i64) -> Result<Server, ServerRepoError> {
        let conn = self.db.conn.lock().unwrap();
        conn.query_row(
            "SELECT s.id, s.name, s.hostname, s.port, s.username, s.environment_id,
                    s.auth_type, s.description, s.os_type, s.tags, s.notes,
                    s.status, s.is_intranet,
                    s.created_at, s.updated_at, s.last_accessed
             FROM servers s
             WHERE s.id = ?",
            params![id],
            |row| {
                Ok(Server {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    hostname: row.get(2)?,
                    port: row.get(3)?,
                    username: row.get(4)?,
                    environment_id: row.get(5)?,
                    auth_type: row.get::<_, Option<String>>(6)?.unwrap_or_else(|| "password".to_string()),
                    password_encrypted: None,
                    private_key_encrypted: None,
                    passphrase_encrypted: None,
                    description: row.get(7)?,
                    os_type: row.get::<_, Option<String>>(8)?.unwrap_or_else(|| "linux".to_string()),
                    tags: row.get(9)?,
                    notes: row.get(10)?,
                    status: row.get::<_, Option<String>>(11)?.unwrap_or_else(|| "unknown".to_string()),
                    is_intranet: row.get(12)?,
                    created_at: row.get(13)?,
                    updated_at: row.get(14)?,
                    last_accessed: row.get(15)?,
                })
            },
        )
        .map_err(|_| ServerRepoError::NotFound)
    }

    pub fn create(&self, server: &Server, password: &str, private_key: &str, passphrase: &str) -> Result<Server, ServerRepoError> {
        let conn = self.db.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO servers (name, hostname, port, username, environment_id, auth_type,
                                  password_encrypted, private_key_encrypted, passphrase_encrypted,
                                  description, os_type, tags, notes, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                server.name, server.hostname, server.port, server.username, server.environment_id,
                server.auth_type,
                self.encrypt_field(password),
                self.encrypt_field(private_key),
                self.encrypt_field(passphrase),
                server.description, server.os_type, server.tags, server.notes,
                now, now,
            ],
        )
        .map_err(|e| ServerRepoError::Database(e.to_string()))?;

        let id = conn.last_insert_rowid();
        drop(conn);

        self.get_by_id(id)
    }

    pub fn update(&self, server: &Server, password: &str, private_key: &str, passphrase: &str) -> Result<(), ServerRepoError> {
        let conn = self.db.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE servers SET name = ?, hostname = ?, port = ?, username = ?, environment_id = ?,
                                  auth_type = ?, password_encrypted = ?, private_key_encrypted = ?,
                                  passphrase_encrypted = ?, description = ?, os_type = ?, tags = ?,
                                  notes = ?, updated_at = ?
             WHERE id = ?",
            params![
                server.name, server.hostname, server.port, server.username, server.environment_id,
                server.auth_type,
                self.encrypt_field(password),
                self.encrypt_field(private_key),
                self.encrypt_field(passphrase),
                server.description, server.os_type, server.tags, server.notes,
                now, server.id,
            ],
        )
        .map_err(|e| ServerRepoError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<(), ServerRepoError> {
        let conn = self.db.conn.lock().unwrap();
        conn.execute("DELETE FROM servers WHERE id = ?", params![id])
            .map_err(|e| ServerRepoError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn get_password(&self, id: i64) -> Result<String, ServerRepoError> {
        let conn = self.db.conn.lock().unwrap();
        let encrypted: Option<Vec<u8>> = conn
            .query_row(
                "SELECT password_encrypted FROM servers WHERE id = ?",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| ServerRepoError::Database(e.to_string()))?;
        self.decrypt_field(&encrypted)
    }

    pub fn get_private_key(&self, id: i64) -> Result<String, ServerRepoError> {
        let conn = self.db.conn.lock().unwrap();
        let encrypted: Option<Vec<u8>> = conn
            .query_row(
                "SELECT private_key_encrypted FROM servers WHERE id = ?",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| ServerRepoError::Database(e.to_string()))?;
        self.decrypt_field(&encrypted)
    }

    pub fn update_status(&self, id: i64, status: &str, is_intranet: bool) -> Result<(), ServerRepoError> {
        let conn = self.db.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE servers SET status = ?, is_intranet = ?, updated_at = ? WHERE id = ?",
            params![status, is_intranet, now, id],
        )
        .map_err(|e| ServerRepoError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn update_last_accessed(&self, id: i64) -> Result<(), ServerRepoError> {
        let conn = self.db.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE servers SET last_accessed = ? WHERE id = ?",
            params![now, id],
        )
        .map_err(|e| ServerRepoError::Database(e.to_string()))?;
        Ok(())
    }
}
