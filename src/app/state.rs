use crate::config::Args;
use regex::Regex;
use reqwest::Client;
use scraper::Selector;
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub args: Args,
    pub link_selectors: Vec<Selector>,
    pub word_selectors: Vec<Selector>,
    pub regex: Regex,
    pub cancel_token: CancellationToken,
}

impl AppState {
    pub fn new(
        client: Client,
        args: Args,
        link_selectors: Vec<Selector>,
        word_selectors: Vec<Selector>,
        regex: Regex,
        cancel_token: CancellationToken,
    ) -> Self {
        AppState {
            client,
            args,
            link_selectors,
            word_selectors,
            regex,
            cancel_token,
        }
    }
}
