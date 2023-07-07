mod crawler;
mod args;
use crate::crawler::crawl;
use crate::args::get_args;

fn main() {
    let args = get_args();
    crawl(
        args.get_one::<String>("URL").unwrap(),
        args.get_one::<String>("WORD"),
        *args.get_one::<i32>("DEPTH").unwrap()
    );
}
