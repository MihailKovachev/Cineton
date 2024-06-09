use regex::Regex;

use axum::{http::StatusCode, response::IntoResponse, routing::Router, routing::post};
use std::sync::Arc;

use crate::SharedState;

pub mod login;
pub mod register;

pub fn routes() -> Router<Arc<SharedState>> {
    Router::new().route("/register", post(register::api_register))
}

#[derive(Debug)]
pub enum AuthError {
    LoginNotFound,
    RegistrationFailed,
    UsernameAlreadyExists,
    UsernameNotAllowed,
    EmailAlreadyExists,
    UserExists,
    EmailNotAllowed,
    Other(String)
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::UsernameNotAllowed => (StatusCode::UNPROCESSABLE_ENTITY, "Usernames may contain only Latin letters, digits 0-9, dots (.) and underscores (_) and must start with a letter or underscore.").into_response(),
            Self::EmailNotAllowed => (StatusCode::UNPROCESSABLE_ENTITY, "Invalid email address.").into_response(),
            Self::UserExists => (StatusCode::CONFLICT, "Username / email is already registered.").into_response(),
            Self::Other(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_ERROR").into_response()
        }
        
    }
}

pub fn validate_username(username: &str) -> Result<(), AuthError> {
    let username_regex = Regex::new(r"^[\w.]+$").unwrap();
    if username.is_ascii() 
    && !username.starts_with(char::is_numeric) 
    && !username.starts_with('.') 
    && username_regex.is_match(&username) {
        return Ok(());
    }

    Err(AuthError::UsernameNotAllowed)
}

pub fn validate_email(email: &str) -> Result<(), AuthError> {
    if serde_email::is_valid_email(&email) {
        return Ok(());
    }

    Err(AuthError::EmailNotAllowed)
}