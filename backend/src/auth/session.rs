use core::fmt;

use ring::rand::SystemRandom;
use base64::prelude::*;
use sqlx::{Postgres, Pool};
use sqlx::Row;

use log::*;

#[derive(Debug)]
pub enum SessionError {
    FailedToGenerateSessionID,
    SessionCreationFailed,
    SessionTerminationFailed
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FailedToGenerateSessionID => write!(f, "Failed to generate a session ID."),
            Self::SessionCreationFailed => write!(f, "Failed to create session."),
            Self::SessionTerminationFailed => write!(f, "Failed to terminate session")
        }
    }
}

impl std::error::Error for SessionError {}

/// Creates a session for the provided user.
/// Returns the generated session id.
pub async fn create_session(user_id: i64, rng: &SystemRandom, db: &Pool<Postgres>) -> Result<String, SessionError> {
    // Generate session ID
    let Ok(session_id_bytes) = ring::rand::generate::<[u8; 8]>(rng) else { return Err(SessionError::FailedToGenerateSessionID); };
    let session_id = BASE64_STANDARD.encode(&session_id_bytes.expose());

    if let Err(err) = 
    sqlx::query(r#"INSERT INTO sessions (user_id, session_id, created, expires) 
    VALUES ($1, $2, now(), now() + interval '1 hour')"#)
    .bind(&user_id)
    .bind(&session_id).execute(db).await {
        error!(target: "auth", "Failed to create session: {}", err.to_string());
        return Err(SessionError::SessionCreationFailed);
    }

    Ok(session_id)
}

/// Checks if the specified session_id represents an active session.
pub async fn is_session_active(session_id: &str, db: &Pool<Postgres>) -> bool {
    match 
    sqlx::query(r#"SELECT expires FROM sessions WHERE session_id = $1"#)
    .bind(&session_id).fetch_one(db).await {
        Ok(record) => {
            let Ok::<chrono::DateTime<chrono::Utc>, _>(expires) = record.try_get("expires") else {
                return false;
            };

            return chrono::Local::now() < expires;
        },
        Err(err) => {
            match err {
                sqlx::Error::RowNotFound => false,
                _ => { 
                    error!(target: "auth", "Failed to verify if session is active: {}", err.to_string());
                    false
                }
            }
            
        }
    }
}

/// Terminates a session by deleting its row from the sessions table
pub async fn terminate_session(session_id: &str, db: &Pool<Postgres>) -> Result<(), SessionError> {
    match 
    sqlx::query(r#"DELETE FROM sessions WHERE session_id = $1"#)
    .bind(session_id)
    .execute(db).await {
        Ok(_) => Ok(()),
        Err(err) => {
            error!(target: "auth", "Failed to terminate session: {}", err.to_string());
            return Err(SessionError::SessionTerminationFailed); }
    }
}