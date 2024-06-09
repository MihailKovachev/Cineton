use axum::{http::StatusCode, Json};
use axum::extract::State;
use std::sync::Arc;
use serde::Deserialize;
use axum::response::IntoResponse;

use crate::SharedState;

use super::{validate_email, validate_username, AuthError};

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
        Err(err) => {return err.into_response(); }
    }
    match validate_email(&payload.email) {
        Ok(_) => (),
        Err(err) => {return err.into_response(); }
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
        Ok(_) => (StatusCode::CREATED, "User registered successfully.").into_response(),
        Err(err) => AuthError::Other(err.to_string()).into_response()
    }
}