use std::{error::Error, net::SocketAddr, sync::Arc};
use ring::rand::{SecureRandom, SystemRandom};

use log::*;

pub mod api;
pub mod db;
pub mod auth;

#[derive(Clone)]
pub struct SharedState {
    database: sqlx::Pool<sqlx::Postgres>,
    rng: SystemRandom
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialise logging
    log4rs::init_file("log4rs.yaml", Default::default()).expect("Failed to read configuration for logging!");
    trace!(target: "main", "Initialised logging.");

    // Initialise a secure random number generator
    let rng = SystemRandom::new();
    {
        // This is done in order to reduce latency in future calls to "fill"
        let mut initialisation_buffer = [0; 128];
        rng.fill(&mut initialisation_buffer).expect("Failed to initialize random number generator!");
    }

    trace!(target: "main", "Initialised secure random number generator.");

    let state = Arc::new(SharedState { database: db::connect().await?, rng });
    
    sqlx::migrate!("./migrations").run(&state.database).await?;
    trace!(target: "main", "Initialised database.");

    let api_router = axum::Router::new()
    .route("/ping", axum::routing::get(api::pong).post(api::pong));

    let app = axum::Router::new()
    .nest("/api", api_router)
    .nest("/auth", auth::routes())
    .with_state(state);

    let host_addr = SocketAddr::from(([127,0,0,1], 31337));
    let listener = tokio::net::TcpListener::bind(host_addr).await.unwrap();

    trace!(target: "main", "Listening on {host_addr}\n");
    axum::serve(listener, app).await.unwrap();
    
    Ok(())
}


