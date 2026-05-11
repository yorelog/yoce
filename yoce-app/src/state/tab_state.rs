/// In-memory tab state for the Yoce shell.
#[derive(Clone)]
pub struct TabState {
    pub id: u64,
    pub title: String,
    pub url: String,
}
