use std::time::Duration;
use scraper::Selector;
use reqwest::blocking::Client;
use clap::ArgMatches;
use reqwest::header::USER_AGENT;
use colored::Colorize;
use regex::Regex;
use std::collections::{HashSet, VecDeque};
use crate::utils::{get_timeout, find_links, get_document};

pub fn setup(args: ArgMatches) {
    let depth: i32;
    let timeout: u64;
    let url = args.get_one::<String>("URL").unwrap();

    match args.get_one::<u32>("DEPTH") {
        Some(d) => {depth = *d as i32},
        None => {depth = -1} // -1 is being used for the "indefinite" crawl
    }

    match args.get_one::<u32>("WARMUP") {
        Some(w) => {timeout = get_timeout(url, *w)}
        None => {
            match args.get_one::<u64>("TIMEOUT") {
                Some(t) => {timeout = *t * 1000},
                None => {timeout = 3000}
            }
        }
    }

    let client = Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_millis(timeout))
        .build()
        .expect("Failed to build reqwest client");

    match args.get_one::<String>("WORD") {
        Some(w) => {crawl_word(&client, url, w, depth)},
        None => {crawl_url(&client, url, depth)}
    }
}

fn crawl_word(client: &Client, url: &str, word: &str, mut depth: i32) {
    let mut found_in_document: bool;
    let mut visited = HashSet::<String>::new();
    let mut to_visit = VecDeque::new();

    let link_selectors = vec!["a[href]"];
    let regex = Regex::new(&format!("(?i)\\b{}\\b", word)).expect("Failed to create regex");
    to_visit.push_back(url.to_string());

    while let Some(current_url) = to_visit.pop_front() {
        if visited.contains(&current_url) {
            continue;
        }
        visited.insert(current_url.clone());

        if let Some(document) = get_document(client, &current_url) {
            found_in_document = false;
            let links = find_links(&current_url, &document, &link_selectors);
            let selectors = vec!["title", "text", "p", "h1", "h2", "h3", "h4", "h5", "h6"];

            for selector in selectors {
                let element_selector = Selector::parse(selector).expect("Failed to parse selector");

                let matches: Vec<_> = document
                    .select(&element_selector)
                    .flat_map(|element| element.text())
                    .filter(|text| regex.is_match(text))
                    .collect();

                // prints the contents of the tags that contain the selected word
                if !matches.is_empty() {
                    if !found_in_document {
                        println!("{}\n--------------------", current_url.green());
                        found_in_document = true;
                    }
                    for word_match in matches {
                        let colored_text = regex.replace_all(&word_match, |caps: &regex::Captures| {
                            caps[0].red().to_string()
                        });
                        println!("- {}", colored_text);
                    }
                    println!();
                }
            }
            if depth != 0 {
                depth -= 1;
                for link in links {
                    if !visited.contains(&link) {
                        to_visit.push_back(link);
                    }
                }
            }
        }
    }
}

fn crawl_url(client: &Client, url: &str, mut depth: i32) {
    let mut visited = HashSet::<String>::new();
    let mut to_visit = VecDeque::new();

    let link_selectors = vec!["a[href]", "link[href]"];
    to_visit.push_back(url.to_string());

    while let Some(current_url) = to_visit.pop_front() {
        if visited.contains(&current_url) {
            continue;
        }
        visited.insert(current_url.clone());

        if let Some(document) = get_document(client, &current_url) {
            let links = find_links(&current_url, &document, &link_selectors);

            if depth != 0 {
                depth -= 1;
                for link in links {
                    if !visited.contains(&link) {
                        println!("{}", link);
                        to_visit.push_back(link);
                    }
                }
            } else {
                for link in links {
                    if !visited.contains(&link) {
                        visited.insert(link.clone());
                        println!("{}", link);
                    }
                }
            }
        }
    }
}
