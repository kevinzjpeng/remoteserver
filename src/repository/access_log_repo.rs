use crate::database::Database;
use crate::models::AccessLog;
use rusqlite::params;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccessLogRepoError {
    #[error("Database error: {0}")]
    Database(String),
}

pub struct AccessLogRepository {
    db: Arc<Database>,
}

impl AccessLogRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn create(
        &self,
        server_id: i64,
        user_id: i64,
        connection_type: &str,
        success: bool,
        notes: &str,
    ) -> Result<(), AccessLogRepoError> {
        let conn = self.db.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO access_logs (server_id, user_id, accessed_at, connection_type, success, notes)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![server_id, user_id, now, connection_type, success, notes],
        )
        .map_err(|e| AccessLogRepoError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn get_all(&self, limit: i64) -> Result<Vec<AccessLog>, AccessLogRepoError> {
        let conn = self.db.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, server_id, user_id, accessed_at, connection_type, success, notes
                 FROM access_logs ORDER BY accessed_at DESC LIMIT ?",
            )
            .map_err(|e| AccessLogRepoError::Database(e.to_string()))?;

        let logs = stmt
            .query_map(params![limit], |row| {
                Ok(AccessLog {
                    id: row.get(0)?,
                    server_id: row.get(1)?,
                    user_id: row.get(2)?,
                    accessed_at: row.get(3)?,
                    connection_type: row.get(4)?,
                    success: row.get(5)?,
                    notes: row.get(6)?,
                })
            })
            .map_err(|e| AccessLogRepoError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AccessLogRepoError::Database(e.to_string()))?;

        Ok(logs)
    }

    pub fn get_by_server_id(
        &self,
        server_id: i64,
        limit: i64,
    ) -> Result<Vec<AccessLog>, AccessLogRepoError> {
        let conn = self.db.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, server_id, user_id, accessed_at, connection_type, success, notes
                 FROM access_logs WHERE server_id = ?
                 ORDER BY accessed_at DESC LIMIT ?",
            )
            .map_err(|e| AccessLogRepoError::Database(e.to_string()))?;

        let logs = stmt
            .query_map(params![server_id, limit], |row| {
                Ok(AccessLog {
                    id: row.get(0)?,
                    server_id: row.get(1)?,
                    user_id: row.get(2)?,
                    accessed_at: row.get(3)?,
                    connection_type: row.get(4)?,
                    success: row.get(5)?,
                    notes: row.get(6)?,
                })
            })
            .map_err(|e| AccessLogRepoError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AccessLogRepoError::Database(e.to_string()))?;

        Ok(logs)
    }
}
