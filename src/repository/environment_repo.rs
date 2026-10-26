use crate::database::Database;
use crate::models::Environment;
use rusqlite::params;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EnvironmentRepoError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Environment not found")]
    NotFound,
}

pub struct EnvironmentRepository<'a> {
    db: &'a Database,
}

impl<'a> EnvironmentRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn get_all(&self) -> Result<Vec<Environment>, EnvironmentRepoError> {
        let conn = self.db.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, description, color, display_order, created_at FROM environments ORDER BY display_order")
            .map_err(|e| EnvironmentRepoError::Database(e.to_string()))?;

        let envs = stmt
            .query_map([], |row| {
                Ok(Environment {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    color: row.get(3)?,
                    display_order: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| EnvironmentRepoError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| EnvironmentRepoError::Database(e.to_string()))?;

        Ok(envs)
    }

    pub fn get_by_id(&self, id: i64) -> Result<Environment, EnvironmentRepoError> {
        let conn = self.db.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, name, description, color, display_order, created_at FROM environments WHERE id = ?",
            params![id],
            |row| {
                Ok(Environment {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    color: row.get(3)?,
                    display_order: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )
        .map_err(|_| EnvironmentRepoError::NotFound)
    }

    pub fn create(
        &self,
        name: &str,
        description: &str,
        color: &str,
        display_order: i32,
    ) -> Result<Environment, EnvironmentRepoError> {
        let conn = self.db.conn.lock().unwrap();
        let now = chrono::Utc::now().naive_utc();

        conn.execute(
            "INSERT INTO environments (name, description, color, display_order, created_at) VALUES (?, ?, ?, ?, ?)",
            params![name, description, color, display_order, now],
        )
        .map_err(|e| EnvironmentRepoError::Database(e.to_string()))?;

        let id = conn.last_insert_rowid();
        drop(conn);

        self.get_by_id(id)
    }

    pub fn update(
        &self,
        id: i64,
        name: &str,
        description: &str,
        color: &str,
        display_order: i32,
    ) -> Result<(), EnvironmentRepoError> {
        let conn = self.db.conn.lock().unwrap();
        conn.execute(
            "UPDATE environments SET name = ?, description = ?, color = ?, display_order = ? WHERE id = ?",
            params![name, description, color, display_order, id],
        )
        .map_err(|e| EnvironmentRepoError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<(), EnvironmentRepoError> {
        let conn = self.db.conn.lock().unwrap();
        conn.execute("DELETE FROM environments WHERE id = ?", params![id])
            .map_err(|e| EnvironmentRepoError::Database(e.to_string()))?;
        Ok(())
    }
}
