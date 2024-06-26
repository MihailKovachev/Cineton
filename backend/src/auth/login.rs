use axum::{
    extract::State, http::StatusCode, response::{IntoResponse, Response}, Json
};

use serde::Deserialize;
use sqlx::Row;
use crate::SharedState;
use core::fmt;
use std::sync::Arc;

use log::*;

use super::session::create_session;

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    username: String,
    password: String
}

#[derive(Debug)]
pub enum LoginError {
    UsernameEmpty,
    PasswordEmpty,
    InvalidCredentials
}

impl fmt::Display for LoginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::UsernameEmpty => write!(f, "Username cannot be empty."),
            Self::PasswordEmpty => write!(f, "Password cannot be empty."),
            Self::InvalidCredentials => write!(f, "Invalid username or password.")
        }
    }
}

impl std::error::Error for LoginError {}

impl IntoResponse for LoginError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::UsernameEmpty | Self::PasswordEmpty => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()).into_response(),
            Self::InvalidCredentials => (StatusCode::UNAUTHORIZED, self.to_string()).into_response()
        }
    }
}

pub async fn auth_login(State(state): State<Arc<SharedState>>, payload: Json<LoginPayload>) -> axum::response::Response {
    // Check credentials
    if payload.username.is_empty() { return LoginError::UsernameEmpty.into_response(); };
    if payload.password.is_empty() { return LoginError::PasswordEmpty.into_response(); };

    match sqlx::query(r#"SELECT user_id FROM users WHERE username = $1 AND password = crypt($2, password)"#)
    .bind(&payload.username)
    .bind(&payload.password).fetch_one(&state.database).await {
        Ok(row) => {
            let Ok::<i64, _>(user_id) = row.try_get("user_id") else {
                error!(target: "auth", "Failed to obtain user_id from query.");
                return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response();
            };

            match create_session(user_id, &state.rng, &state.database).await {
                Ok(session_id) => {
                    return Response::builder()
                    .status(200)
                    .header("Content-Type", "application/json")
                    .body(format!(r#"{{ "session_id" : "{}" }}"#, session_id)).unwrap().into_response();
                },
                Err(_) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response();
                }
            }
        },
        Err(err) => {
            error!(target: "auth", "Failed to authenticate user. Reason: {}", err.to_string());
            return LoginError::InvalidCredentials.into_response();
        }
    }
    
}