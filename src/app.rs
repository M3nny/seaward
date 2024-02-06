use clap::{Arg, ArgMatches, ArgAction, command, crate_version, value_parser};
use::colored::Colorize;
use std::time::Duration;
use reqwest::Client;
use reqwest::header::USER_AGENT;
use crate::utils::get_timeout;
use crate::crawler::{crawl_url, crawl_word};

const BANNER: &str = "
                             _
 ___ ___ ___ _ _ _ ___ ___ _| |
|_ -| -_| .'| | | | .'|  _| . |
|___|___|__,|_____|__,|_| |___|";

const ABOUT: &str = "
seaward is a crawler which searches for links or a specified word in a website.
Use -h for short descriptions and --help for more details.

Project home page: https://github.com/M3nny/seaward
";

fn get_args() -> ArgMatches {
    let args = command!()
        .about(format!("seaward: {}\n{}", crate_version!(), ABOUT))
        .arg(Arg::new("URL")
            .required(true)
            .help("Base url to start with.")
        )
        .arg(Arg::new("WORD")
            .short('w')
            .long("word")
            .help("Case insensitive word to search.")
        )
        .arg(Arg::new("DEPTH")
            .short('d')
            .long("depth")
            .value_parser(value_parser!(u32))
            .help("Set how many times a link has to be followed.")
            .long_help(
                "By default the search is performed until there are no more internal links to visit.\n0: only the base url is searched\n1: the base url and its internal links are searched\n..."
            )
        )
        .arg(Arg::new("TIMEOUT")
            .short('t')
            .long("timeout")
            .value_parser(value_parser!(u64))
            .help_heading("Timeout")
            .help("Set a request timeout in milliseconds (default: 3000ms).")
            .long_help(
                "Set a request timeout in milliseconds (default: 3000ms)\nlow timeout: ignores long requests thus making the crawling faster\nhigh timeout: higher probabilities of getting a response from every link, but decreasing the crawling speed with long requests"
            )
        )
        .arg(Arg::new("WARMUP")
            .long("warmup")
            .value_parser(value_parser!(u32))
            .help_heading("Timeout")
            .help("Set how many requests to make to find the best timeout automatically.")
            .long_help("An average of n requests timings is made, this can lead to many connection timeouts! (overrides --timeout option)")
        )
        .arg(Arg::new("SILENT")
            .long("silent")
            .action(ArgAction::SetTrue)
            .help("Display output only.")
        )
        .get_matches();
    args
}

pub async fn setup() {
    let args = get_args();

    if !args.get_flag("SILENT") {
        println!("{} v: {}\n", BANNER, crate_version!());
    }

    let depth: i32;
    let timeout: u64;
    let url = args.get_one::<String>("URL").unwrap();

    match args.get_one::<u32>("DEPTH") {
        Some(d) => {depth = *d as i32},
        None => {depth = -1} // -1 is being used for the "indefinite" crawl
    }

    match args.get_one::<u32>("WARMUP") {
        Some(w) => {timeout = get_timeout(url, *w, args.get_flag("SILENT")).await}
        None => {
            match args.get_one::<u64>("TIMEOUT") {
                Some(t) => {timeout = *t},
                None => {timeout = 3000}
            }
        }
    }

    let client = Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_millis(timeout))
        .build()
        .expect(&"Failed to build reqwest client".red());

    match args.get_one::<String>("WORD") {
        Some(w) => {crawl_word(&client, url, w, depth).await},
        None => {crawl_url(&client, url, depth).await}
    }
}
