use std::{error::Error, net::SocketAddr, sync::Arc};

pub mod api;
pub mod db;
pub mod auth;

#[derive(Clone)]
pub struct SharedState {
    database: sqlx::Pool<sqlx::Postgres>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let state = Arc::new(SharedState {database: db::connect().await? });
    
    sqlx::migrate!("./migrations").run(&state.database).await?;

    let api_router = axum::Router::new()
    .route("/ping", axum::routing::get(api::pong).post(api::pong));

    let app = axum::Router::new()
    .nest("/api", api_router)
    .nest("/auth", auth::routes())
    .with_state(state);

    let host_addr = SocketAddr::from(([127,0,0,1], 8080));
    let listener = tokio::net::TcpListener::bind(host_addr).await.unwrap();

    println!("LISTENING ON {host_addr}\n");
    axum::serve(listener, app).await.unwrap();
    
    Ok(())
}


