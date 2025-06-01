use std::collections::HashSet;

/// Result of processing a single URL
#[derive(Debug)]
pub struct CrawlResult {
    pub url: String,
    pub depth: u32,
    pub links: HashSet<String>,
    pub matches: Vec<String>,
}
