use axum::{http::StatusCode, Json};
use axum::extract::State;
use log::{info, error};
use std::sync::Arc;
use core::fmt;
use regex::Regex;
use serde::Deserialize;
use axum::response::IntoResponse;

use crate::SharedState;

#[derive(Debug)]
enum RegistrationError {
    UsernameStartsWithDigit,
    UsernameStartsWithPeriod,
    UsernameContainsForbiddenCharacters,
    InvalidEmail,
    InsecurePassword,
    PasswordContainsForbiddenCharacters
}

impl fmt::Display for RegistrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::UsernameStartsWithDigit => write!(f, "Username cannot start with a digit."),
            Self::UsernameStartsWithPeriod => write!(f, "Username cannot start with a period."),
            Self::UsernameContainsForbiddenCharacters => write!(f, "Username may only contain upper- and lowercase Latin letters, digits, periods and underscores, but cannot begin with a digit or period."),
            Self::InvalidEmail => write!(f, "Not a valid email address."),
            Self::InsecurePassword => write!(f, r###"Insecure password - password should be between 8 and 72 characters long, contain a mixture of upper- and lowercase letters, at least one number and at least one special symbol like !@#$%^&*()_+:;<>/?"###),
            Self::PasswordContainsForbiddenCharacters => write!(f, "Password contains invalid characters.")
        }
    }
}

impl IntoResponse for RegistrationError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()).into_response()
    }
}

impl std::error::Error for RegistrationError {}

fn validate_username(username: &str) -> Result<(), RegistrationError> {
    // Usernames may contain only a-zA-Z0-9._
    let username_regex = Regex::new(r"^[\w.]+$").unwrap();

    if !username.is_ascii() || !username_regex.is_match(&username) {
        return Err(RegistrationError::UsernameContainsForbiddenCharacters);
    }

    if username.starts_with('.') {
        return Err(RegistrationError::UsernameStartsWithPeriod);
    }

    if username.starts_with(char::is_numeric) {
        return Err(RegistrationError::UsernameStartsWithDigit);
    }

    Ok(())
}

fn validate_email(email: &str) -> Result<(), RegistrationError> {
    if serde_email::is_valid_email(&email) {
        return Ok(());
    }

    Err(RegistrationError::InvalidEmail)
}

fn validate_password(password: &str) -> Result<(), RegistrationError> {
    if !password.is_ascii() { return Err(RegistrationError::PasswordContainsForbiddenCharacters); };
    
    if password.len() < 8
    || password.len() > 72 
    || !password.chars().any(|c| c.is_digit(10)) 
    || !password.chars().any(|c| c.is_ascii_lowercase())
    || !password.chars().any(|c| c.is_ascii_uppercase())
    || !password.chars().any(|c| "!@#$%^&*()_+:;<>/?".contains(c)) {
        return Err(RegistrationError::InsecurePassword);
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct RegisterPayload {
    username: String,
    email: String,
    password: String,
    first_name: String,
    last_name: String
}

pub async fn auth_register(State(state): State<Arc<SharedState>>, payload: Json<RegisterPayload>) -> impl IntoResponse {

    match validate_username(&payload.username) {
        Ok(_) => (),
        Err(err) => {
            error!(target: "auth", "Failed to register user {{username: {}, first_name: {}, last_name: {}, email: {} }}. Reason: {}", 
            payload.username, payload.first_name, payload.last_name, payload.email, err.to_string());
            return err.into_response(); 
        }
    }
    match validate_email(&payload.email) {
        Ok(_) => (),
        Err(err) => {
            error!(target: "auth", "Failed to register user {{username: {}, first_name: {}, last_name: {}, email: {} }}. Reason: {}", 
            payload.username, payload.first_name, payload.last_name, payload.email, err.to_string());
            return err.into_response(); 
        }
    }

    match validate_password(&payload.password) {
        Ok(_) => (),
        Err(err) => { 
            error!(target: "auth", "Failed to register user {{username: {}, first_name: {}, last_name: {}, email: {} }}. Reason: {}", 
            payload.username, payload.first_name, payload.last_name, payload.email, err.to_string());
            return err.into_response(); 
        }
    }

    match sqlx::query(
        r#"
        INSERT INTO users (username, first_name, last_name, email, password, account_status, user_perms, created, last_login)
        VALUES ($1, $2, $3, $4, crypt($5, gen_salt('bf')), 'active', 'photographer', now(), NULL);
        "#
    )
    .bind(&payload.username)
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&payload.email)
    .bind(&payload.password)
    .execute(&state.database).await {
        Ok(_) => {
            info!(target: "auth", "User {{username: {}, first_name: {}, last_name: {}, email: {} }} registered successfully.", 
            payload.username, payload.first_name, payload.last_name, payload.email);
            return (StatusCode::CREATED, "Registration successful.").into_response(); 
        },
        Err(err) => {
            match &err {
                sqlx::Error::Database(err) => {
                    match err.kind() {
                        sqlx::error::ErrorKind::UniqueViolation => { 
                            error!(target: "auth", 
                            "Failed to register user {{username: {}, first_name: {}, last_name: {}, email: {} }}. Reason: {}",
                            payload.username, payload.first_name, payload.last_name, payload.email, err.to_string());
                            
                            return (StatusCode::CONFLICT, "A user with this username / email has already been registered.").into_response(); },
                        _ => ()
                    }
                },
                _ => ()
            }
            
            error!(target: "auth", "Failed to register user {{username: {}, first_name: {}, last_name: {}, email: {} }}. Reason: {}", 
            payload.username, payload.first_name, payload.last_name, payload.email, err.to_string());

            return (StatusCode::INTERNAL_SERVER_ERROR, "Registration failed.").into_response();
        }
    }
}
