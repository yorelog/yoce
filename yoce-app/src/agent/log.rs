use gpui::{App, AppContext, Entity, SharedString};

// ---------------------------------------------------------------------------
// LogEntry
// ---------------------------------------------------------------------------

/// A single structured log entry, compatible with the `log` crate.
#[derive(Clone)]
#[allow(dead_code)]
pub struct LogEntry {
    pub level: log::Level,
    pub target: SharedString,   // e.g. "shell", "agent", "yoce"
    pub message: SharedString,
}

impl LogEntry {
    pub fn new(level: log::Level, target: impl Into<SharedString>, message: impl Into<SharedString>) -> Self {
        Self {
            level,
            target: target.into(),
            message: message.into(),
        }
    }

    #[allow(dead_code)]
    pub fn format_short(&self) -> SharedString {
        SharedString::from(format!(
            "[{}][{}] {}",
            self.level.as_str(),
            self.target,
            self.message
        ))
    }
}

// ---------------------------------------------------------------------------
// LogStore — in-app log collector
// ---------------------------------------------------------------------------

/// Collects `LogEntry` records for in-app display.
///
/// Use the standard `log::info!`, `log::warn!`, `error!`, `debug!` macros for
/// console logging.  Call `LogStore::push()` to also surface entries in the UI.
///
/// Future: remote log upload reads `entries()` and sends via HTTP.
pub struct LogStore {
    entries: Vec<LogEntry>,
    max_entries: usize,
}

impl LogStore {
    const DEFAULT_MAX: usize = 500;

    pub fn new() -> Self {
        Self {
            entries: Vec::with_capacity(Self::DEFAULT_MAX),
            max_entries: Self::DEFAULT_MAX,
        }
    }

    pub fn create(cx: &mut App) -> Entity<Self> {
        cx.new(|_| Self::new())
    }

    /// All entries, oldest first.
    #[allow(dead_code)]
    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }

    /// Most recent `n` entries, newest first.
    #[allow(dead_code)]
    pub fn recent(&self, n: usize) -> impl Iterator<Item = &LogEntry> {
        self.entries.iter().rev().take(n)
    }

    /// Push a single entry.  Called alongside `info!()` / `warn!()` etc.
    pub fn push(&mut self, entry: LogEntry) {
        self.entries.push(entry);
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    /// Shorthand for `push(LogEntry::new(...))`.
    pub fn log(
        &mut self,
        level: log::Level,
        target: impl Into<SharedString>,
        message: impl Into<SharedString>,
    ) {
        self.push(LogEntry::new(level, target, message));
    }
}
