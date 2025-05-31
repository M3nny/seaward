//! App configuration and cli args parsing.

use crate::{app::errors::AppError, info};
use clap::{command, crate_version, Parser};
use colored::Colorize;
use reqwest::Client;
use std::time::{Duration, Instant};

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

/// Contains the args specified via cli.
#[derive(Clone, Parser)]
#[command(
    name = "seaward",
    version = crate_version!(),
    about = format!("seaward: {}\n{}", crate_version!(), ABOUT)
)]
pub struct Args {
    #[arg(help = "Base URL used to start crawling.")]
    pub url: String,

    #[arg(short = 'w', long = "word", help = "Case insensitive word to search.")]
    pub word: String,

    #[arg(
        short = 'd',
        long = "depth",
        value_parser = clap::value_parser!(u32),
        default_value_t = 100,
        help = "Set the crawl depth."
    )]
    pub depth: u32,

    #[arg(
        short = 't',
        long = "timeout",
        value_parser = clap::value_parser!(u64),
        default_value_t = 3000,
        help = "Set a request timeout in milliseconds (default: 3000ms).",
        long_help = "Set a request timeout in milliseconds (default: 3000ms)\nlow timeout: ignores long requests thus making the crawling faster\nhigh timeout: higher probabilities of getting a response from every link, but decreasing the crawling speed with long requests."
    )]
    pub timeout: u64,

    #[arg(
        long = "warmup",
        value_parser = clap::value_parser!(u32),
        default_value_t = 0,
        help = "Set how many requests to make to find the best timeout automatically.",
        long_help = "An average of n requests timings is made, this can lead to many connection timeouts! (overrides --timeout option)."
    )]
    pub warmup_requests: u32,

    #[arg(
        short = 's',
        long = "strict",
        action = clap::ArgAction::SetTrue,
        default_value_t = false,
        help = "Crawl the links only if they are subpaths of the base url.",
        long_help = "Crawl the links only if they are subpaths of the base url.\ne.g. base_url: https://example.com/sub/\nhttps://example.com/sub/file/ will be followed\nhttps://example.com/sub2/ will NOT be followed."
    )]
    pub strict: bool,

    #[arg(
        long = "user-agent",
        default_value = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:122.0) Gecko/20100101 Firefox/122.0",
        help = "Set the user agent used for making the requests"
    )]
    pub user_agent: String,

    #[arg(
        long = "link-selectors",
        value_delimiter = ',',
        num_args = 1..,
        default_value = "a[href]",
        help = "Set the html tags to consider for exploring new links (default: a[href])"
    )]
    pub link_tags: Vec<String>,

    #[arg(
        long = "word-selectors",
        value_delimiter = ',',
        num_args = 1..,
        default_value = "title,text,p,h1,h2,h3,h4,h5,h6",
        help = "Set the HTML tags to consider when matching the given word (default: title, text, p, h1...h6)"
    )]
    pub word_tags: Vec<String>,

    #[arg(
        short = 'i',
        long = "ignore-case",
        action = clap::ArgAction::SetTrue,
        default_value_t = false,
        help = "Ignore case sensitivity",
    )]
    pub case_insensitive: bool,

    #[arg(long = "silent", action = clap::ArgAction::SetTrue, default_value_t = false, help = "Display output only.")]
    pub silent: bool,
}

/// Gets a greedy timeout estimation by returning the longest request time summed to an extra
/// delay.
pub async fn get_timeout(args: &Args) -> u64 {
    let warmup_client = Client::builder()
        .user_agent(&args.user_agent)
        .build()
        .expect(&format!(
            "\n[{}] Failed to build reqwest client",
            "FATAL".red()
        ));

    let mut max_elapsed_time = Duration::new(0, 0);

    for i in 0..args.warmup_requests {
        let start_time = Instant::now();

        if let Ok(response) = warmup_client.get(&args.url).send().await {
            if response.status().is_success() {
                max_elapsed_time = if start_time.elapsed() > max_elapsed_time {
                    start_time.elapsed()
                } else {
                    max_elapsed_time
                };
            }

            if args.silent {
                info!(
                    "Request({}/{}): {:?}",
                    i + 1,
                    args.warmup_requests,
                    start_time.elapsed()
                );
            }
        }
    }

    let greedy_timeout = (max_elapsed_time.as_millis() as u64) + 1000;
    info!("Using a timeout of: {}ms", greedy_timeout);

    greedy_timeout
}

/// Prints app ascii logo.
fn print_banner() {
    println!("{} v: {}\n", BANNER, crate_version!());
}

/// Sets up the app config.
///
/// # Returns
/// A struct containing the cli arguments.
pub async fn setup() -> Result<(Args, Client), AppError> {
    let mut args = Args::parse();

    if !args.silent {
        print_banner();
    }

    if args.warmup_requests > 0 {
        args.timeout = get_timeout(&args).await;
    }

    let client = Client::builder()
        .user_agent(&args.user_agent)
        .timeout(Duration::from_millis(args.timeout))
        .build()
        .map_err(|err| AppError::new(err.to_string()))?;

    Ok((args.clone(), client))
}
