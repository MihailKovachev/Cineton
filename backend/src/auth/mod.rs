use regex::Regex;

use axum::{http::StatusCode, response::IntoResponse, routing::Router, routing::post};
use core::fmt;
use std::{error::Error, sync::Arc};

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
    Other(String)
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Other(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_ERROR").into_response()
        }
        
    }
}

#[derive(Debug)]
pub enum UsernameParseError {
    StartsWithDigit,
    StartsWithPeriod,
    ForbiddenCharacters
}

impl fmt::Display for UsernameParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::StartsWithDigit => write!(f, "Username cannot start with a digit."),
            Self::StartsWithPeriod => write!(f, "Username cannot start with a period."),
            Self::ForbiddenCharacters => write!(f, "Username may only contain upper- and lowercase Latin letters, digits, periods and underscores, but cannot begin with a digit or period.")
        }
    }
}

impl std::error::Error for UsernameParseError {}

pub fn validate_username(username: &str) -> Result<(), UsernameParseError> {
    // Usernames may contain only a-zA-Z0-9._
    let username_regex = Regex::new(r"^[\w.]+$").unwrap();

    if !username.is_ascii() || !username_regex.is_match(&username) {
        return Err(UsernameParseError::ForbiddenCharacters);
    }

    if username.starts_with('.') {
        return Err(UsernameParseError::StartsWithPeriod);
    }

    if username.starts_with(char::is_numeric) {
        return Err(UsernameParseError::StartsWithDigit);
    }

    Ok(())
}

#[derive(Debug)]
pub enum EmailParseError {
    InvalidEmail
}

impl std::error::Error for EmailParseError {}

impl fmt::Display for EmailParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::InvalidEmail => write!(f, "Not a valid email address.")
        }
    }
}

pub fn validate_email(email: &str) -> Result<(), EmailParseError> {
    if serde_email::is_valid_email(&email) {
        return Ok(());
    }

    Err(EmailParseError::InvalidEmail)
}