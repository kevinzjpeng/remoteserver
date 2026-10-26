use crate::database::Database;
use crate::models::User;
use chrono::NaiveDateTime;
use rusqlite::params;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserRepoError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("User not found")]
    NotFound,
}

pub struct UserRepository<'a> {
    db: &'a Database,
}

impl<'a> UserRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn get_by_id(&self, id: i64) -> Result<User, UserRepoError> {
        let conn = self.db.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, github_id, username, email, avatar_url, is_admin, created_at, last_login
             FROM users WHERE id = ?",
            params![id],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    github_id: row.get(1)?,
                    username: row.get(2)?,
                    email: row.get(3)?,
                    avatar_url: row.get(4)?,
                    is_admin: row.get(5)?,
                    created_at: row.get(6)?,
                    last_login: row.get(7)?,
                })
            },
        )
        .map_err(|_| UserRepoError::NotFound)
    }

    pub fn get_by_github_id(&self, github_id: &str) -> Result<User, UserRepoError> {
        let conn = self.db.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, github_id, username, email, avatar_url, is_admin, created_at, last_login
             FROM users WHERE github_id = ?",
            params![github_id],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    github_id: row.get(1)?,
                    username: row.get(2)?,
                    email: row.get(3)?,
                    avatar_url: row.get(4)?,
                    is_admin: row.get(5)?,
                    created_at: row.get(6)?,
                    last_login: row.get(7)?,
                })
            },
        )
        .map_err(|_| UserRepoError::NotFound)
    }

    pub fn create_by_github(
        &self,
        github_id: &str,
        username: &str,
        email: &str,
        avatar_url: &str,
    ) -> Result<User, UserRepoError> {
        let conn = self.db.conn.lock().unwrap();

        // First user is admin
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
            .map_err(|e| UserRepoError::Database(e.to_string()))?;
        let is_admin = count == 0;

        let now = chrono::Utc::now().naive_utc();

        conn.execute(
            "INSERT INTO users (github_id, username, email, avatar_url, is_admin, created_at, last_login)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![github_id, username, email, avatar_url, is_admin, now, now],
        )
        .map_err(|e| UserRepoError::Database(e.to_string()))?;

        let id = conn.last_insert_rowid();
        drop(conn);

        self.get_by_id(id)
    }

    pub fn update_last_login(&self, id: i64) -> Result<(), UserRepoError> {
        let conn = self.db.conn.lock().unwrap();
        let now = chrono::Utc::now().naive_utc();
        conn.execute(
            "UPDATE users SET last_login = ? WHERE id = ?",
            params![now, id],
        )
        .map_err(|e| UserRepoError::Database(e.to_string()))?;
        Ok(())
    }
}
