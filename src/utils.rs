use std::time::{Instant, Duration};
use scraper::{Html, Selector};
use reqwest::Client;
use reqwest::Url;
use colored::Colorize;
use std::collections::HashSet;

pub async fn get_timeout(base_url: &str, warmup: u32, silent: bool) -> u64 {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:122.0) Gecko/20100101 Firefox/122.0")
        .build()
        .expect(&"Failed to build reqwest client".red());

    let mut start_time: Instant;
    let mut elapsed_time = Duration::new(0, 0);
    let mut max_elapsed_time = Duration::new(0, 0);
    for i in 0..warmup {
        start_time = Instant::now();
        let response = client.get(base_url).send();

        if let Ok(response) = response.await {
            if response.status().is_success() {
                elapsed_time = start_time.elapsed();
                max_elapsed_time = if elapsed_time > max_elapsed_time {elapsed_time} else {max_elapsed_time};
            }
            if !silent {
                println!("- Request({}/{}): {:?}", i+1, warmup, elapsed_time);
            }
        }

    }

    if !silent {
        println!("Using a timeout of: {:?}ms\n", (max_elapsed_time.as_millis() as u64) + 100);
    }
    max_elapsed_time.as_millis() as u64 + 100
}

fn is_subfolder(base_url: &Url, url: &Url) -> bool {
    let base_path = base_url.path();
    let url_path = url.path();

    url_path
        .get(..base_path.len())
        .map(|url_prefix| url_prefix == base_path)
        .unwrap_or(false)
}

pub fn find_links(base_url: &str, document: &Html, selectors: &[&str], strict: bool) -> HashSet<String> {
    let mut links = HashSet::new();
    let base_url = Url::parse(base_url).expect(&"Failed to parse base URL".red());

    for selector in selectors {
        let element_selector = match Selector::parse(selector) {
            Ok(selector) => selector,
            Err(err) => {
                println!("Failed to parse selector {}: {}", selector, err.to_string().red());
                continue;
            }
        };

        for element in document.select(&element_selector) {
            if let Some(href) = element.value().attr("href") { // check whether the element contains the "href" attribute
                if let Ok(mut url) = base_url.join(href) { // merge the link to the base url
                    url.set_fragment(None); // remove the link fragment "#" if present
                    if let Some(domain) = url.domain() {
                        if let Some(base_domain) = base_url.domain() {
                            if domain.ends_with(base_domain) { // check whether the link is internal to the website
                                if strict {
                                    if is_subfolder(&base_url, &url) {
                                        links.insert(url.to_string());
                                    }
                                } else {
                                    links.insert(url.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    links
}

pub async fn get_document(client: &Client, url: &str) -> Option<Html> {
    let response = client.get(url).send();
    match response.await {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await { // get the page content as a string and then put it inside an HTML struct
                    Ok(body) => {
                        let document = Html::parse_document(&body);
                        Some(document)
                    }
                    Err(err) => {
                        println!("Failed to get response body: {}", err.to_string().red());
                        None
                    }
                }
            } else {
                println!("Request failed at {} with status code: {}", url, response.status().to_string().red());
                None
            }
        }
        Err(err) => {
            println!("Failed to send request: {}", err.to_string().red());
            None
        }
    }
}
