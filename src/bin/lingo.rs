//! `lingo` binary: the same CLI as `glossa` under a second name.

#[tokio::main]
async fn main() {
    if let Err(e) = glossa::cli::run().await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
