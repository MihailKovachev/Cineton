use axum::{http::StatusCode, response::IntoResponse, routing::Router, routing::post};
use std::sync::Arc;

use crate::SharedState;

pub mod login;
pub mod register;

pub fn routes() -> Router<Arc<SharedState>> {
    Router::new()
    .route("/register", post(register::auth_register))
    .route("/login", post(login::auth_login))
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