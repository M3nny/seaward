/// Result of processing a single URL
#[derive(Debug)]
pub struct CrawlResult {
    pub depth: u32,
    pub links: Vec<String>,
}
