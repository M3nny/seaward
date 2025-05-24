use crate::{
    app::{errors::AppError, queue_item::QueueItem},
    config::Args, warn,
};
use colored::Colorize;
use regex::{Regex, RegexBuilder};
use reqwest::{Client, Url};
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};

fn is_subpath(base_url: &Url, url: &Url) -> bool {
    let base_path = base_url.path();
    let url_path = url.path();

    url_path
        .get(..base_path.len())
        .map(|url_prefix| url_prefix == base_path)
        .unwrap_or(false)
}

fn find_links_in_document(
    base_url: &str,
    document: &Html,
    selectors: &Vec<Selector>,
    strict: bool,
) -> Result<HashSet<String>, AppError> {
    let base_url = Url::parse(base_url).map_err(|err| AppError::new(format!("Error while parsing url: {}", err)))?;

    let is_valid_url = |url: &Url| {
        match (url.domain(), base_url.domain()) {
            (Some(domain), Some(base_domain)) => {
                // if strict is set to true then check if the url is a sub-path
                domain.ends_with(base_domain) && (!strict || is_subpath(&base_url, url))
            }
            _ => false,
        }
    };

    let resolve_href = |href: &str| -> Option<Url> {
        base_url.join(href).ok().map(|mut url| {
            url.set_fragment(None);
            url
        })
    };

    let links: HashSet<String> = selectors
        .iter()
        .flat_map(|selector| document.select(selector))
        .filter_map(|element| element.value().attr("href"))
        .filter_map(resolve_href)
        .filter(is_valid_url)
        .map(|url| url.to_string())
        .collect();

    Ok(links)
}

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

fn find_matches<'a>(
    selectors: &[scraper::Selector],
    regex: &regex::Regex,
    document: &'a scraper::Html,
) -> Vec<&'a str> {
    selectors
        .iter()
        .flat_map(|selector| {
            document
                .select(selector)
                .flat_map(|el| el.text())
                .filter(|text| regex.is_match(text))
                .collect::<Vec<&str>>()
        })
        .collect()
}

fn print_matches(url: &String, regex: &Regex, matches: &Vec<&str>) {
    let dash_line = "-".repeat(url.len());

    if !matches.is_empty() {
        println!("{}\n{}", url.blue(), dash_line);

        matches
            .iter()
            .for_each(|word| {
                println!("{}", regex.replace_all(word, |captures: &regex::Captures| captures[0].cyan().to_string()))
            });

        println!();
    }
}

pub async fn crawl(args: &Args, client: &Client) -> Result<(), AppError> {
    let mut visited = HashSet::<String>::new();
    let mut to_visit = VecDeque::<QueueItem>::from([QueueItem(args.url.clone(), 1)]);

    let link_selectors: Vec<Selector> = args
        .link_tags
        .clone()
        .into_iter()
        .map(|s: String| {
            Selector::parse(&s)
                .map_err(|err| AppError::new(format!("Failed to parse link selector: {}", err)))
        })
        .collect::<Result<Vec<Selector>, AppError>>()?;

    let word_selectors: Vec<Selector> = args
        .word_tags
        .clone()
        .into_iter()
        .map(|s: String| {
            Selector::parse(&s)
                .map_err(|err| AppError::new(format!("Failed to parse word selector: {}", err)))
        })
        .collect::<Result<Vec<Selector>, AppError>>()?;

    let pattern = format!(r"\b{}\b", regex::escape(&args.word));
    let regex = RegexBuilder::new(&pattern)
        .case_insensitive(args.case_insensitive)
        .build()
        .map_err(|err| AppError::new(format!("Failed to create regex: {}", err)))?;

    while let Some(QueueItem(current_url, current_depth)) = to_visit.pop_front() {
        if visited.contains(&current_url) {
            continue;
        }

        visited.insert(current_url.clone());

        if let Ok(document) = get_document(&client, &current_url).await {
            let matches: Vec<&str> = find_matches(&word_selectors, &regex, &document);
            print_matches(&current_url, &regex, &matches);

            if args.depth == 0 || current_depth < args.depth {
                if let Ok(links) = find_links_in_document(&current_url, &document, &link_selectors, args.strict) {
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
