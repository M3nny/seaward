mod app;
mod crawler;
mod utils;

use crate::app::setup;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    ctrlc::set_handler(move || {
        println!("\nshutting down... received KeyboardInterrupt");
        std::process::exit(0);
    }).expect("Error setting KeyboardInterrupt handler");

    setup().await;
}
