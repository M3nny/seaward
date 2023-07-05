use std::env;
mod crawler;
use crate::crawler::crawl;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            println!("No arguments provided"); // TODO: show possible arguments
            return;
        }
        2 => {
            if args[1] == "-h" {
                println!("help"); // TODO: show help message
                return;
            } else {
                println!("error: word or url missing"); // TODO: improve this message
                return;
            }
        }
        3 => {
            crawl(&args[1], &args[2], -1);
        }
        4 => {
            let depth = args[3].parse().unwrap();
            if depth < 0 {
                println!("error: cannot assign negative depth");
                return;
            }
            crawl(&args[1], &args[2], depth);
        }
        _ => {
            println!("error: Too many arguments");
            return;
        }
    }
}
