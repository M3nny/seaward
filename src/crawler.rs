use std::time::{Instant, Duration};
use scraper::{Html, Selector};
use reqwest::blocking::Client;
use reqwest::header::USER_AGENT;
use reqwest::Url;
use colored::Colorize;
use regex::Regex;
use std::collections::{HashSet, VecDeque};

fn get_timeout(base_url: &str, warmup: u32) -> u64 {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect("Failed to build reqwest client");
    let mut total_elapsed_time = Duration::new(0, 0);

    for _ in 0..warmup {
        let start_time = Instant::now();
        let response = client.get(base_url).send();

        if let Ok(response) = response {
            if response.status().is_success() {
                let elapsed_time = start_time.elapsed();
                total_elapsed_time += elapsed_time;
            }
        }
    }

    let average_elapsed_time: u64 = (total_elapsed_time / warmup).as_millis() as u64;
    average_elapsed_time
}

fn find_links(base_url: &str, document: &Html, selectors: &[&str]) -> HashSet<String> {
    let mut links = HashSet::new();
    let base_url = Url::parse(base_url).expect("Failed to parse base URL");

    for selector in selectors {
        let element_selector = Selector::parse(selector).expect("Failed to parse selector");

        for element in document.select(&element_selector) {
            if let Some(href) = element.value().attr("href") {
                if let Ok(url) = base_url.join(href) {
                    if let Some(domain) = url.domain() {
                        if let Some(base_domain) = base_url.domain() {
                            if domain.ends_with(base_domain) {
                                links.insert(url.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    links
}

fn get_document(client: &Client, url: &str) -> Option<Html> {
    let response = client.get(url).send();
    match response {
        Ok(response) => {
            if response.status().is_success() {

                // get the page content as a string and then put it inside an HTML struct
                let body = response.text().expect("Failed to get response body");
                let document = Html::parse_document(&body);
                Some(document)
            } else {
                println!("Request failed at {} with status code: {}", url.purple(), response.status());
                None
            }
        }
        Err(err) => {
            println!("Failed to send request: {}", err);
            None
        }
    }
}

pub fn setup(url: &str, word: Option<&String>, arg_depth: Option<&u32>, arg_timeout: Option<&u64>, arg_warmup: Option<&u32>) {
    let depth: i32;
    let timeout: u64;

    match arg_depth {
        Some(d) => {depth = *d as i32},
        None => {depth = -1} // -1 is being used for the "indefinite" crawl
    }

    match arg_warmup {
        Some(w) => {timeout = get_timeout(url, *w)}
        None => {
            match arg_timeout {
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

    match word {
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
