mod migrations;

use rusqlite::Connection;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Failed to create database directory: {0}")]
    CreateDir(String),
    #[error("Failed to open database: {0}")]
    Open(String),
    #[error("Failed to run migrations: {0}")]
    Migration(String),
}

pub struct Database {
    pub conn: Mutex<Connection>,
}

pub fn setup_database(db_path: &str) -> Result<Database, DatabaseError> {
    if let Some(parent) = std::path::Path::new(db_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| DatabaseError::CreateDir(e.to_string()))?;
    }

    let conn = Connection::open(db_path)
        .map_err(|e| DatabaseError::Open(e.to_string()))?;

    conn.execute("PRAGMA foreign_keys = ON", [])
        .map_err(|e| DatabaseError::Open(e.to_string()))?;

    migrations::run_migrations(&conn)
        .map_err(|e| DatabaseError::Migration(e.to_string()))?;

    Ok(Database {
        conn: Mutex::new(conn),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_setup_database_creates_tables() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        let db = setup_database(db_path.to_str().unwrap()).unwrap();

        let conn = db.conn.lock().unwrap();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert!(tables.contains(&"users".to_string()));
        assert!(tables.contains(&"servers".to_string()));
        assert!(tables.contains(&"environments".to_string()));
        assert!(tables.contains(&"service_credentials".to_string()));
        assert!(tables.contains(&"access_logs".to_string()));
    }
}
