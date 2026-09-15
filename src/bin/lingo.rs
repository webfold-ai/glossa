//! Drop-in Lingo.dev compatibility binary for `glossa`.
//!
//! Enables drop-in replacement without modifying existing scripts or CI/CD pipelines.

#[tokio::main]
async fn main() {
    if let Err(e) = glossa::cli::run().await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
