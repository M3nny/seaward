pub mod app;
pub mod config;
pub mod log;

use app::{core::crawl, errors::AppError};
use config::setup;

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
