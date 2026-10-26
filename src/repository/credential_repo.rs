use crate::crypto::encryption;
use crate::database::Database;
use crate::models::Credential;
use rusqlite::params;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CredentialRepoError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Credential not found")]
    NotFound,
    #[error("Encryption error: {0}")]
    Encryption(String),
}

pub struct CredentialRepository<'a> {
    db: &'a Database,
    encryption_key: Vec<u8>,
}

impl<'a> CredentialRepository<'a> {
    pub fn new(db: &'a Database, encryption_key: Vec<u8>) -> Self {
        Self { db, encryption_key }
    }

    fn encrypt_credentials(&self, data: &[u8]) -> Option<Vec<u8>> {
        if data.is_empty() {
            return None;
        }
        encryption::encrypt(data, &self.encryption_key).ok()
    }

    pub fn get_all(&self) -> Result<Vec<Credential>, CredentialRepoError> {
        let conn = self.db.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, service_type, config_json, is_active, created_at, updated_at FROM service_credentials ORDER BY name")
            .map_err(|e| CredentialRepoError::Database(e.to_string()))?;

        let creds = stmt
            .query_map([], |row| {
                Ok(Credential {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    service_type: row.get(2)?,
                    credentials_encrypted: None,
                    config_json: row.get(3)?,
                    is_active: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| CredentialRepoError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CredentialRepoError::Database(e.to_string()))?;

        Ok(creds)
    }

    pub fn get_by_id(&self, id: i64) -> Result<Credential, CredentialRepoError> {
        let conn = self.db.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, name, service_type, config_json, is_active, created_at, updated_at FROM service_credentials WHERE id = ?",
            params![id],
            |row| {
                Ok(Credential {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    service_type: row.get(2)?,
                    credentials_encrypted: None,
                    config_json: row.get(3)?,
                    is_active: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )
        .map_err(|_| CredentialRepoError::NotFound)
    }

    pub fn create(&self, cred: &Credential, credentials_data: &[u8]) -> Result<Credential, CredentialRepoError> {
        let conn = self.db.conn.lock().unwrap();
        let now = chrono::Utc::now().naive_utc();

        conn.execute(
            "INSERT INTO service_credentials (name, service_type, credentials_encrypted, config_json, is_active, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                cred.name, cred.service_type,
                self.encrypt_credentials(credentials_data),
                cred.config_json, cred.is_active, now, now,
            ],
        )
        .map_err(|e| CredentialRepoError::Database(e.to_string()))?;

        let id = conn.last_insert_rowid();
        drop(conn);

        self.get_by_id(id)
    }

    pub fn update(&self, cred: &Credential, credentials_data: &[u8]) -> Result<(), CredentialRepoError> {
        let conn = self.db.conn.lock().unwrap();
        let now = chrono::Utc::now().naive_utc();

        conn.execute(
            "UPDATE service_credentials SET name = ?, service_type = ?, credentials_encrypted = ?,
                                          config_json = ?, is_active = ?, updated_at = ?
             WHERE id = ?",
            params![
                cred.name, cred.service_type,
                self.encrypt_credentials(credentials_data),
                cred.config_json, cred.is_active, now, cred.id,
            ],
        )
        .map_err(|e| CredentialRepoError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<(), CredentialRepoError> {
        let conn = self.db.conn.lock().unwrap();
        conn.execute("DELETE FROM service_credentials WHERE id = ?", params![id])
            .map_err(|e| CredentialRepoError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn get_decrypted_credentials(&self, id: i64) -> Result<Vec<u8>, CredentialRepoError> {
        let conn = self.db.conn.lock().unwrap();
        let encrypted: Option<Vec<u8>> = conn
            .query_row(
                "SELECT credentials_encrypted FROM service_credentials WHERE id = ?",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| CredentialRepoError::Database(e.to_string()))?;

        match encrypted {
            None => Ok(Vec::new()),
            Some(bytes) if bytes.is_empty() => Ok(Vec::new()),
            Some(bytes) => encryption::decrypt(&bytes, &self.encryption_key)
                .map_err(|e| CredentialRepoError::Encryption(e.to_string())),
        }
    }
}
