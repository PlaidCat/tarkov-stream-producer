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

fn add_two(a: i32) -> i32 {
    a + 2
}

#[cfg(test)]
mod test {
    use super::*; // Import everything from the outerscope

    #[test]
    fn it_works() {
        assert_eq!(add_two(2), 4);
        assert_ne!(add_two(3), 4);
    }
}
