//! Data type stored inside of the crawling queue.

/// Items pushed inside of the crawling queue.
///
/// # Parameters
/// - `String`: the url to be visited.
/// - `u32`: depth relative to the starting url.
pub struct QueueItem(pub String, pub u32);

impl QueueItem {
    pub fn new(url: String, depth: u32) -> Self {
        Self(url, depth)
    }
}
