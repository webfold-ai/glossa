//! Binary entrypoint for `lingo`.

#[tokio::main]
async fn main() {
    if let Err(e) = ptyktos_lingo::cli::run().await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
