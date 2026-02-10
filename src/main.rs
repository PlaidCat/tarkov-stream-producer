mod api;
mod db;
mod models;
mod stats;

use tracing::{info};
use tracing_subscriber::{self, EnvFilter};

use crate::api::state::AppState;
use crate::api::routes::api_router;

#[tokio::main]
async fn main() {
    // If the RUST_LOG is not set set the filter to INFO
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    info!("Logger initialized and application starting!");
    println!("Hello, world!");

    // Connect to SQLite database (create file if it doesn't exist)
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:dev.db?mode=rwc".to_string());
    let pool = db::create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    // Apply any pending database migrations
    db::run_migrations(&pool)
        .await
        .expect("Failed to run database migrations");

    info!("Database Initialized");

    // Build the router and attach the database pool
    let app = api_router().with_state(AppState::new(pool));

    // Start listening on localhost port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to port 3000");

    info!("Server listening on http://127.0.0.1:3000");

    // This line blocks forever, handleing incoming requests
    axum::serve(listener, app)
        .await
        .expect("Server Error");
}
