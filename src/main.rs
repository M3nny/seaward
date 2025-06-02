pub mod app;
pub mod config;
pub mod enums;
pub mod structs;

use app::core::crawl;
use config::setup;
use enums::log;
use structs::error::AppError;

/// Listens for a keyboard interrupt while crawling.
async fn run() -> Result<(), AppError> {
    let shared_state = setup().await?;
    crawl(shared_state).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        error!("{}", err);
    }
}
