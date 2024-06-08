use std::{error::Error, net::SocketAddr};

pub mod api;
pub mod db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_router = axum::Router::new()
    .route("/ping", axum::routing::get(api::pong).post(api::pong));

    let app = axum::Router::new()
    .nest("/api", api_router);

    let database = db::connect().await?;

    sqlx::migrate!("./migrations").run(&database).await?;

    let host_addr = SocketAddr::from(([127,0,0,1], 8080));
    let listener = tokio::net::TcpListener::bind(host_addr).await.unwrap();

    println!("LISTENING ON {host_addr}\n");
    axum::serve(listener, app).await.unwrap();
    
    Ok(())
}


