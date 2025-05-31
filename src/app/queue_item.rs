//! Data type store inside of the crawling queue.

/// Contains a string representing thr url to be visited and the depth reltive to the root url.
pub struct QueueItem(pub String, pub u32);

impl QueueItem {
    pub fn new(url: String, depth: u32) -> Self {
        Self(url, depth)
    }
}
