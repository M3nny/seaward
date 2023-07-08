use clap::{Arg, ArgMatches, command, value_parser};

pub fn get_args() -> ArgMatches{
    let args = command!()
        .arg(Arg::new("URL")
            .required(true)
            .help("Base url to start with")
        )
        .arg(Arg::new("WORD")
            .short('w')
            .long("word")
            .help("Case insensitive word to search")
        )
        .arg(Arg::new("DEPTH")
            .short('d')
            .long("depth")
            .value_parser(value_parser!(u32))
            .help("Set how many times a link has to be followed")
            .long_help(
                "By default the search is performed until there are no more internal links to visit.
                0: only the base url is searched
                1: the base url and its internal links are searched
                ..."
            )
        )
        .arg(Arg::new("TIMEOUT")
            .short('t')
            .long("timeout")
            .value_parser(value_parser!(u32))
            .help_heading("Timeout")
            .help("Set a request timeout")
            .long_help(
                "Set a request timeout.
                low timeout: ignores long requests thus making the crawling faster
                high timeout: higher probabilities of getting a response from every link, but decreasing the crawling speed with long requests"
            )
        )
        .arg(Arg::new("WARMUP")
            .long("warmup")
            .value_parser(value_parser!(u32))
            .help_heading("Timeout")
            .help("Set how many requests to make to find the best timeout automatically")
            .long_help("An average of n requests timings is made, this can lead to many connection timeouts!")
        )
        .get_matches();
    args
}
