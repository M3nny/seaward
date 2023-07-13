use std::time::{Instant, Duration};
use scraper::{Html, Selector};
use reqwest::Client;
use reqwest::header::USER_AGENT;
use reqwest::Url;
use colored::Colorize;
use std::collections::HashSet;

pub async fn get_timeout(base_url: &str, warmup: u32) -> u64 {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect(&"Failed to build reqwest client".red());
    let mut total_elapsed_time = Duration::new(0, 0);

    for _ in 0..warmup {
        let start_time = Instant::now();
        let response = client.get(base_url).send();

        if let Ok(response) = response.await {
            if response.status().is_success() {
                let elapsed_time = start_time.elapsed();
                total_elapsed_time += elapsed_time;
            }
        }
    }

    let average_elapsed_time: u64 = (total_elapsed_time / warmup).as_millis() as u64;
    average_elapsed_time
}

pub fn find_links(base_url: &str, document: &Html, selectors: &[&str]) -> HashSet<String> {
    let mut links = HashSet::new();
    let base_url = Url::parse(base_url).expect(&"Failed to parse base URL".red());

    for selector in selectors {
        let element_selector = Selector::parse(selector).expect(&"Failed to parse selector".red());

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

pub async fn get_document(client: &Client, url: &str) -> Option<Html> {
    let response = client.get(url).send();
    match response.await {
        Ok(response) => {
            if response.status().is_success() {

                // get the page content as a string and then put it inside an HTML struct
                let body = response.text().await.expect(&"Failed to get response body".red());
                let document = Html::parse_document(&body);
                Some(document)
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
