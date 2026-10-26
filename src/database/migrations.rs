use rusqlite::Connection;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MigrationError {
    #[error("Failed to execute migration: {0}")]
    Execute(String),
}

pub fn run_migrations(conn: &Connection) -> Result<(), MigrationError> {
    let migration = include_str!("migrations/001_initial_schema.sql");

    conn.execute_batch(migration)
        .map_err(|e| MigrationError::Execute(e.to_string()))?;

    Ok(())
}
