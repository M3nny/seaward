use scraper::{Html, Selector};
use colored::Colorize;
use regex::Regex;
use std::collections::{HashSet, VecDeque};

fn find_links(domain: &str, document: &Html) -> HashSet<String> {
    let link_selector = Selector::parse("a[href]").expect("Failed to parse link selector");
    let mut links = HashSet::new();

    for element in document.select(&link_selector) {
        if let Some(href) = element.value().attr("href") {
            if href.starts_with('/') {
                let absolute_link = format!("{}{}", domain, href);
                links.insert(absolute_link);
            } else if !href.starts_with("http") {
                let absolute_link = format!("{}/{}", domain, href);
                links.insert(absolute_link);
            } else if href.starts_with(domain) {
                links.insert(href.to_string());
            }
        }
    }
    links
}

fn get_document(url: &str) -> Option<Html> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/113.0")
        .build()
        .expect("Failed to build reqwest client");
    let response = client.get(url).send();
    match response {
        Ok(response) => {
            if response.status().is_success() {

                // get the page content as a string and then put it inside an HTML struct
                let body = response.text().expect("Failed to get response body");
                let document = Html::parse_document(&body);
                Some(document)
            } else {
                println!("Request failed with status code: {}", response.status());
                None
            }
        }
        Err(err) => {
            println!("Failed to send request: {}", err);
            None
        }
    }
}

pub fn crawl(url: &str, word: &str, visited: &mut HashSet<String>) {
    let mut found_occurence: bool;
    let mut to_visit = VecDeque::new();
    to_visit.push_back(url.to_string());

    let pattern = format!("(?i)\\b{}\\b", word);
    let regex = Regex::new(&pattern).expect("Failed to create regex");

    while let Some(current_url) = to_visit.pop_front() {
        found_occurence = false;
        if visited.contains(&current_url) {
            continue;
        }
        visited.insert(current_url.clone());

        if let Some(document) = get_document(&current_url) {
            let links = find_links(&current_url, &document);

            // selectors to be examined
            let selectors = vec!["title", "text", "p", "h1", "h2", "h3", "h4", "h5", "h6"];


            let colored_url = current_url.green();
            println!("{}\n--------------------", colored_url);
            for selector in selectors {
                let element_selector = Selector::parse(selector).expect("Failed to parse selector");

                let matches: Vec<_> = document
                    .select(&element_selector)
                    .flat_map(|element| element.text())
                    .filter(|text| regex.is_match(text))
                    .collect();
                if !matches.is_empty() {found_occurence = true};

                // prints the contents of the tags that contain the selected word
                for word_match in matches {
                    let colored_text = regex.replace_all(&word_match, |caps: &regex::Captures| {
                        caps[0].red().to_string()
                    });
                    println!("- {}", colored_text);
                }
            }
            if !found_occurence {
                println!("No occurences of {} found", word);
            }
            println!();
            for link in links {
                if !visited.contains(&link) {
                    to_visit.push_back(link);
                }
            }
        }
    }
}
