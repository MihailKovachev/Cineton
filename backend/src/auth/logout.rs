use crate::SharedState;
use super::{session::is_session_active, session::terminate_session};

use axum::{
    extract::State, http::StatusCode, response::IntoResponse, Json
};

use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct LogoutPayload {
    session_id: String
}

pub async fn auth_logout(State(state): State<Arc<SharedState>>, payload: Json<LogoutPayload>) -> axum::response::Response {
    if !is_session_active(&payload.session_id, &state.database).await 
    {
        return (StatusCode::UNAUTHORIZED, "Could not logout as user was not logged in.").into_response();
    }

    match terminate_session(&payload.session_id, &state.database).await {
        Ok(_) => (StatusCode::OK, "Logout successful.").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Logout failed").into_response()
    }
}