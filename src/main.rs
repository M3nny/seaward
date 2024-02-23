mod app;
mod crawler;
mod utils;

use app::setup;
use tokio::signal::ctrl_c;
use colored::Colorize;

#[tokio::main]
async fn main() {
    tokio::spawn(async {
        setup().await;
        std::process::exit(0);
    });

    match ctrl_c().await {
        Ok(_) => {
            println!("\n[{}] Shutting down: received KeyboardInterrupt", "INFO".green());
            std::process::exit(0);
        }
        Err(err) => {
            eprintln!("\n[{}] Unable to listen for shutdown signal: {}", "FATAL".red(), err);
            std::process::exit(0);
        }
    }
}
