//! Utils methods used by core.

use crate::{app_error, structs::error::AppError};
use colored::Colorize;
use rayon::prelude::*;
use regex::Regex;
use reqwest::{Client, Url};
use scraper::{Html, Selector};
use std::collections::HashSet;

/// Check if a given url is a subpath of another one.
///
/// # Parameters
/// - `base_url`: url that may contain the specified subpath.
/// - `url`: subpath to be checked against the base url.
///
/// # Returns
/// A boolean which tells if `url` is a subpath of `base_url`.
pub fn is_subpath(base_url: &Url, url: &Url) -> bool {
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
pub fn find_links_in_document<'a>(
    base_url: &str,
    document: &Html,
    selectors: &[Selector],
    strict: bool,
) -> Result<HashSet<String>, AppError> {
    let parsed_base_url =
        Url::parse(base_url).map_err(|err| app_error!("Error while parsing url: {}", err))?;

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

/// Retrieves an html document by making a get http request to a given url.
///
/// # Parameters
/// - `client`: http client used for making the request.
/// - `url`: url used for making the request.
///
/// # Returns
/// The html document returned by the http request.
pub async fn get_document(client: &Client, url: &str) -> Result<Html, AppError> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|err| app_error!("Error while fetching document: {}", err))?;

    let body = if response.status().is_success() {
        response
            .text()
            .await
            .map_err(|err| app_error!("Failed to read body: {}", err))?
    } else {
        return Err(app_error!(
            "Response failed with status code: {}",
            response.status().to_string()
        ));
    };

    Ok(Html::parse_document(&body))
}

/// Finds the strings that match the given regex inside of the provided selectors.
///
/// # Parameters
/// - `selectors`: html selectors that specifies in which tags to search for the regex.
/// - `regex`: represents the searched word/phrase inside of the website.
/// - `document`: html document to be parsed in search of regex matches.
///
/// # Returns
/// The `regex` matches inside of the `document`.
pub fn find_matches(selectors: &[Selector], regex: &Regex, document: &Html) -> Vec<String> {
    let texts: Vec<String> = selectors
        .iter()
        .flat_map(|selector| document.select(selector).flat_map(|element| element.text()))
        .map(|text| text.to_string())
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
pub fn print_matches(url: &str, regex: &Regex, matches: &Vec<String>) {
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
