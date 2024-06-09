use axum::{http::StatusCode, Json};
use axum::extract::State;
use log::{info, error};
use std::sync::Arc;
use serde::Deserialize;
use axum::response::IntoResponse;

use crate::SharedState;

use super::{validate_email, validate_username};

#[derive(Debug, Deserialize)]
pub struct RegisterPayload {
    username: String,
    email: String,
    password: String,
    first_name: String,
    last_name: String
}

pub async fn api_register(State(state): State<Arc<SharedState>>, payload: Json<RegisterPayload>) -> impl IntoResponse {
    
    match validate_username(&payload.username) {
        Ok(_) => (),
        Err(err) => { return (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()).into_response(); }
    }
    match validate_email(&payload.email) {
        Ok(_) => (),
        Err(err) => { return (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()).into_response(); }
    }

    match sqlx::query(
        r#"
        INSERT INTO Users (username, first_name, last_name, email, password_hash, account_status, user_perms, created, last_login)
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