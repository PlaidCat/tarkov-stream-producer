use tracing::{info};
use tracing_subscriber::{self, EnvFilter};

fn main() {
    // If the RUST_LOG is not set set the filter to INFO
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    info!("Logger initialized and application starting!");
    println!("Hello, world!");
}
