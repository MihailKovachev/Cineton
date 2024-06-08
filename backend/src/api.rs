use axum::response;

pub async fn pong() -> impl response::IntoResponse {
    response::Html("Pong!")
}