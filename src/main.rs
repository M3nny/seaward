mod crawler;
mod args;
use crate::crawler::setup;
use crate::args::get_args;

fn main() {
    ctrlc::set_handler(move || {
        println!("\nshutting down... received KeyboardInterrupt");
        std::process::exit(0);
    }).expect("Error setting KeyboardInterrupt handler");

    let args = get_args();

    setup(
        args.get_one::<String>("URL").unwrap(),
        args.get_one::<String>("WORD"),
        args.get_one::<u32>("DEPTH"),
        args.get_one::<u64>("TIMEOUT"),
        args.get_one::<u32>("WARMUP")
    );
}
