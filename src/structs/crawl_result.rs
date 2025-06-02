//! Result returned from each crawling task.

use std::collections::HashSet;

/// Result of processing a single URL
pub struct CrawlResult {
    /// Crawled url.
    pub url: String,
    /// Distance relative to the starting url.
    pub depth: u32,
    /// Links found while crawling `url`.
    pub links: HashSet<String>,
    /// Word matches found while crawling `url`.
    pub matches: Vec<String>,
}
