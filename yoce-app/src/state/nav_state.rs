use std::sync::{Arc, Mutex};

/// Cross-thread shared state for navigation updates.
/// wry callbacks write here; the main thread reads and syncs.
pub struct NavState {
    pub pending_url: Option<String>,
    pub pending_title: Option<String>,
}

/// Thread-safe handle to NavState.
/// Reserved for future use by agent panel and command bus.
#[allow(dead_code)]
pub type SharedNavState = Arc<Mutex<NavState>>;
