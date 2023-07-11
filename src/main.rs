mod crawler;
mod args;
mod utils;

use crate::crawler::setup;
use crate::args::get_args;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    ctrlc::set_handler(move || {
        println!("\nshutting down... received KeyboardInterrupt");
        std::process::exit(0);
    }).expect("Error setting KeyboardInterrupt handler");

    setup(get_args()).await;
}
