use crate::utils::{find_links, get_document};
use colored::Colorize;
use regex::Regex;
use reqwest::Client;
use scraper::Selector;
use std::collections::{HashSet, VecDeque};

pub async fn crawl_word(client: &Client, url: &str, word: &str, mut depth: i32, strict: bool) {
    let mut found_in_document: bool;
    let mut visited = HashSet::<String>::new();
    let mut to_visit = VecDeque::new();

    let link_selectors = vec!["a[href]"];
    let regex = Regex::new(&format!("(?i)\\b{}\\b", word)).expect(&format!("[{}] Failed to create regex", "FATAL".red()));
    to_visit.push_back(url.to_string());

    while let Some(current_url) = to_visit.pop_front() {
        if visited.contains(&current_url) {
            continue;
        }
        visited.insert(current_url.clone());

        if let Some(document) = get_document(client, &current_url).await {
            found_in_document = false;
            let links = find_links(&current_url, &document, &link_selectors, strict);
            let selectors = vec!["title", "text", "p", "h1", "h2", "h3", "h4", "h5", "h6"];

            for selector in selectors {
                let element_selector = match Selector::parse(selector) {
                    Ok(parsed_selector) => parsed_selector,
                    Err(err) => {
                        eprintln!("[{}] Failed to parse selector \"{}\": {}", "ERROR".magenta(), selector, err);
                        continue;
                    }
                };

                let matches: Vec<_> = document
                    .select(&element_selector)
                    .flat_map(|element| element.text())
                    .filter(|text| regex.is_match(text))
                    .collect();

                // prints the contents of the tags that contain the selected word
                if !matches.is_empty() {
                    if !found_in_document {
                        println!("{}\n--------------------", current_url.blue());
                        found_in_document = true;
                    }
                    for word_match in matches {
                        let colored_text = regex
                            .replace_all(&word_match, |caps: &regex::Captures| {
                                caps[0].cyan().to_string()
                            });
                        println!("● {}", colored_text);
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

pub async fn crawl_url(client: &Client, url: &str, mut depth: i32, strict: bool) {
    let mut visited = HashSet::<String>::new();
    let mut to_visit = VecDeque::new();

    let link_selectors = vec!["a[href]", "link[href]"];
    to_visit.push_back(url.to_string());

    while let Some(current_url) = to_visit.pop_front() {
        if visited.contains(&current_url) {
            continue;
        }
        visited.insert(current_url.clone());

        if let Some(document) = get_document(client, &current_url).await {
            let links = find_links(&current_url, &document, &link_selectors, strict);

            if depth != 0 {
                depth -= 1;
                for link in links {
                    if !visited.contains(&link) && !to_visit.contains(&link) {
                        println!("{}", link);
                        to_visit.push_back(link);
                    }
                }
            } else {
                for link in links {
                    if !visited.contains(&link) && !to_visit.contains(&link) {
                        println!("{}", link);
                        visited.insert(link.clone());
                    }
                }
            }
        }
    }
}
