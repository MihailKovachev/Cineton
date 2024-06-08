use std::net::SocketAddr;

mod api;

#[tokio::main]
async fn main() {
    let api_router = axum::Router::new()
    .route("/ping", axum::routing::get(api::pong).post(api::pong));

    let app = axum::Router::new()
    .nest("/api", api_router);
    
    let host_addr = SocketAddr::from(([127,0,0,1], 8080));
    let listener = tokio::net::TcpListener::bind(host_addr).await.unwrap();
    println!("LISTENING ON {host_addr}\n");
    axum::serve(listener, app).await.unwrap();
}


