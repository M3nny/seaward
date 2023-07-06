use clap::{Arg, ArgMatches, command, value_parser};

pub fn get_args() -> ArgMatches{
    let args = command!()
        .arg(Arg::new("WORD")
            .required(true)
            .help("Case insensitive word to search")
        )
        .arg(Arg::new("URL")
            .required(true)
            .help("Base url to start with")
        )
        .arg(Arg::new("DEPTH")
            .short('d')
            .long("depth")
            .value_parser(value_parser!(i32))
            .default_value("-1").hide_default_value(true)
            .help("Set how many times a link has to be followed")
            .long_help("By default the search is performed until there are no more internal links to visit.\n0: only the base url is searched\n1: the base url and its internal links are searched\n...")
        )
        .get_matches();
    args
}
