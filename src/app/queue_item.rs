pub struct QueueItem(pub String, pub u32);

impl QueueItem {
    pub fn new(url: String, depth: u32) -> Self {
        Self(url, depth)
    }
}
