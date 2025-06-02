//! Shared app state.

use crate::config::Args;
use regex::Regex;
use reqwest::Client;
use scraper::Selector;
use tokio_util::sync::CancellationToken;

/// Struct containing the app state.
pub struct AppState {
    /// Shared client.
    pub client: Client,
    /// Arguments passed to the app via cli.
    pub args: Args,
    /// Parsed html link selectors.
    pub link_selectors: Vec<Selector>,
    /// Parsed html word selectors.
    pub word_selectors: Vec<Selector>,
    /// Regex used to find word matches in a html document.
    pub regex: Regex,
    /// Token shared between tasks,
    /// when cancelled the crawling stops.
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
