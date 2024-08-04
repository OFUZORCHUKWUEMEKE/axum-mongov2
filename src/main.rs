mod db;
mod models;
mod error;
mod auth;
mod routes;
use dotenv::dotenv;

use axum::Server;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::new(
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
    ))
    .with(tracing_subscriber::fmt::layer())
    .init();

let db = db::connect_db().await.expect("Failed to connect to database");
let app = routes::route::create_router(db);

tracing::info!("Starting server on 0.0.0.0:3000");
Server::bind(&"0.0.0.0:3000".parse().unwrap())
    .serve(app.into_make_service())
    .await
    .unwrap();
    println!("Hello, world!");
}
