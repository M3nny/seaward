use scraper::{Html, Selector};
use colored::Colorize;
use regex::Regex;

fn main() {
    let url = "";
    let word = "";

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

                // selectors to be examined
                let selectors = vec!["title", "text", "p", "h1", "h2", "h3", "h4", "h5", "h6"];

                let pattern = format!("(?i)\\b{}\\b", word);
                let regex = Regex::new(&pattern).expect("Failed to create regex");

                for selector in selectors {
                    let element_selector = Selector::parse(selector).expect("Failed to parse selector");

                    let matches: Vec<_> = document
                        .select(&element_selector)
                        .flat_map(|element| element.text())
                        .filter(|text| regex.is_match(text))
                        .collect();

                    // prints the contents of the tags that contain the selected word
                    for word_match in matches {
                        let colored_text = regex.replace_all(&word_match, |caps: &regex::Captures| {
                            caps[0].red().to_string()
                        });
                        println!("- {}", colored_text);
                    }
                }
            } else {
                println!("Request failed with status code: {}", response.status());
            }
        }
        Err(err) => {
            println!("Failed to send request: {}", err);
        }
    }
}
