use rusqlite::{Connection, Result};
use bcrypt;
use uuid::Uuid;
use chrono;
use std::fmt;

// define a custom error to wrap both rusqlite and bcrypt errors
#[derive(Debug)]
pub enum DatabaseError {
    SqliteError(rusqlite::Error),
    BcryptError(bcrypt::BcryptError),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DatabaseError::SqliteError(e) => write!(f, "SQLite error: {}", e),
            DatabaseError::BcryptError(e) => write!(f, "Bcrypt error: {}", e),
        }
    }
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(error: rusqlite::Error) -> Self {
        DatabaseError::SqliteError(error)
    }
}

impl From<bcrypt::BcryptError> for DatabaseError {
    fn from(error: bcrypt::BcryptError) -> Self {
        DatabaseError::BcryptError(error)
    }
}

pub fn establish_connection() -> Result<Connection> {
    let conn = Connection::open("fagan_inventory.db")?;
    Ok(conn)
}

pub fn init_database() -> std::result::Result<(), DatabaseError> {
    let conn = establish_connection()?;
    
    // create users table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )?;
    
    // create initial admin users if they don't exist
    let admin_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM users WHERE username IN ('fagan@admin_1', 'fagan@admin_2')",
        [],
        |row| row.get(0),
    )?;
    
    if admin_count < 2 {
        // create default admin users according to specs
        let admin1_hash = bcrypt::hash("fagan_glass", 10)?;
        let admin2_hash = bcrypt::hash("fagan_aluminum", 10)?;
        let now = chrono::Local::now().to_rfc3339();
        
        conn.execute(
            "INSERT OR IGNORE INTO users (id, username, password_hash, role, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            &[
                &Uuid::new_v4().to_string(),
                "fagan@admin_1",
                &admin1_hash,
                "admin",
                &now,
                &now,
            ],
        )?;
        
        conn.execute(
            "INSERT OR IGNORE INTO users (id, username, password_hash, role, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            &[
                &Uuid::new_v4().to_string(),
                "fagan@admin_2",
                &admin2_hash,
                "admin",
                &now,
                &now,
            ],
        )?;
    }
    
    Ok(())
}