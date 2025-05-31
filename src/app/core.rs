//! Core methods.

use crate::{
    app::{errors::AppError, queue_item::QueueItem},
    config::Args,
    warn,
};
use colored::Colorize;
use fastbloom::BloomFilter;
use rayon::prelude::*;
use regex::{Regex, RegexBuilder};
use reqwest::{Client, Url};
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};

/// Check if a given url is a subpath of  another one.
///
/// # Parameters
/// - `base_url`: url that may contain the specified subpath.
/// - `url`: subpath to be checked against the base url.
///
/// # Returns
/// A boolean which tells if `url` is a subpath of `base_url`.
fn is_subpath(base_url: &Url, url: &Url) -> bool {
    let base_path = base_url.path();
    let url_path = url.path();

    url_path
        .get(..base_path.len())
        .map(|url_prefix| url_prefix == base_path)
        .unwrap_or(false)
}

/// Find internal links in the current document looking inside the specified html selectors.
///
/// # Parameters
/// - `base_url`: url used to check if the link is internal to the crawled website.
/// - `document`: html document to be parsed in search of links.
/// - `selectors`: html selectors that specifies in which tags to search for links.
/// - `strict`: if set to true, the collected links must be a subpath of the base url.
///
/// # Returns
/// Internal links defined inside of the provided `selectors` in the given html `document`.
fn find_links_in_document(
    base_url: &str,
    document: &Html,
    selectors: &Vec<Selector>,
    strict: bool,
) -> Result<HashSet<String>, AppError> {
    let parsed_base_url = Url::parse(base_url)
        .map_err(|err| AppError::new(format!("Error while parsing url: {}", err)))?;

    let is_url_valid = |url: &Url| -> bool {
        match (url.domain(), parsed_base_url.domain()) {
            (Some(domain), Some(base_domain)) => {
                // if strict is set to true then check if the url is a subpath
                domain.ends_with(base_domain) && (!strict || is_subpath(&parsed_base_url, url))
            }
            _ => false,
        }
    };

    let resolve_href = |href: &str| -> Option<Url> {
        parsed_base_url.join(href).ok().map(|mut url| {
            url.set_fragment(None);
            url
        })
    };

    let links: HashSet<String> = selectors
        .iter()
        .flat_map(|selector| document.select(selector))
        .filter_map(|element| element.value().attr("href"))
        .collect::<Vec<&str>>()
        .into_par_iter()
        .filter_map(resolve_href)
        .filter(is_url_valid)
        .map(|url| url.to_string())
        .collect::<HashSet<String>>();

    Ok(links)
}

/// Retrieves an html document by makes a get http request to a given url.
///
/// # Parameters
/// - `client`: http client used for making the request.
/// - `url`: url used for making the request.
///
/// # Returns
/// The html document returned by the http request.
async fn get_document(client: &Client, url: &str) -> Result<Html, AppError> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|err| AppError::new(format!("Error while fetching document: {}", err)))?;

    let body = if response.status().is_success() {
        response
            .text()
            .await
            .map_err(|err| AppError::new(format!("Failed to read body: {}", err)))?
    } else {
        return Err(AppError::new(format!(
            "Response failed with status code: {}",
            response.status().to_string()
        )));
    };

    Ok(Html::parse_document(&body))
}

/// Finds the strings that match the given regex inside of the provided selectors.
///
/// # Parameters
/// - `selectors`: html selectors that specifies in which tags to search for the regex.
/// - `regex`: represents the searched word/phrase inside of the website.
/// - `document`: html document to be parsed in search of regex matchesk.
///
/// # Returns
/// The `regex` matches inside of the `document`.
fn find_matches<'a>(
    selectors: &[Selector],
    regex: &Regex,
    document: &'a Html,
) -> Vec<&'a str> {
    let texts: Vec<&'a str> = selectors
        .iter()
        .flat_map(|selector| document.select(selector).flat_map(|element| element.text()))
        .collect();

    texts
        .into_par_iter()
        .filter(|text| regex.is_match(text))
        .collect()
}

/// Pretty prints the regex matches.
///
/// # Parameters
/// - `url`: the url which contains the printed matches.
/// - `regex`: used to highlight the searched word/phrase.
/// - `matches`: vector of strings that include the searched word/phrase.
fn print_matches(url: &String, regex: &Regex, matches: &Vec<&str>) {
    if !matches.is_empty() {
        println!("{}", url.blue());

        matches.iter().for_each(|word| {
            println!(
                "{}",
                regex.replace_all(word, |captures: &regex::Captures| captures[0]
                    .cyan()
                    .bold()
                    .to_string())
            )
        });

        println!();
    }
}

/// Parses the given string representation of html selectors and returns the correspective
/// selectors.
///
/// # Parameters
/// - `tags`: string repsentation of the html selectors.
///
/// # Returns
/// Selectors that corresponds to the given string representation.
fn parse_selectors(tags: &Vec<String>) -> Result<Vec<Selector>, AppError> {
    tags.into_iter()
        .map(|s| {
            Selector::parse(&s)
                .map_err(|err| AppError::new(format!("Failed to parse selector: {}", err)))
        })
        .collect()
}

/// Crawl entry point.
///
/// # Parameters
/// `args`: provided cli args.
/// `client`: http client that does all the requests.
pub async fn crawl(args: &Args, client: &Client) -> Result<(), AppError> {
    let mut visited = BloomFilter::with_false_pos(0.001).expected_items(1000);
    let mut to_visit = VecDeque::<QueueItem>::from([QueueItem(args.url.clone(), 1)]);

    let link_selectors: Vec<Selector> = parse_selectors(&args.link_tags)?;
    let word_selectors: Vec<Selector> = parse_selectors(&args.word_tags)?;

    let pattern = format!(r"\b{}\b", regex::escape(&args.word));
    let regex = RegexBuilder::new(&pattern)
        .case_insensitive(args.case_insensitive)
        .build()
        .map_err(|err| AppError::new(format!("Failed to create regex: {}", err)))?;

    while let Some(QueueItem(current_url, current_depth)) = to_visit.pop_front() {
        if visited.contains(&current_url) {
            continue;
        }

        visited.insert(&current_url);

        if let Ok(document) = get_document(&client, &current_url).await {
            let matches: Vec<&str> = find_matches(&word_selectors, &regex, &document);
            print_matches(&current_url, &regex, &matches);

            if args.depth == 0 || current_depth < args.depth {
                if let Ok(links) =
                    find_links_in_document(&args.url, &document, &link_selectors, args.strict)
                {
                    links
                        .into_iter()
                        .filter(|link| !visited.contains(link))
                        .for_each(|link| to_visit.push_back(QueueItem(link, current_depth + 1)));
                } else {
                    warn!("Failed to extract links from {}", current_url);
                }
            }
        }
    }

    Ok(())
}
