//! yoce-engine — Shared contracts between the gpui shell (yoce-app)
//! and the AI agent runtime.
//!
//! This crate contains **pure data types only**:
//! - `ShellCommand` — every action the shell can perform.
//! - `ShellEvent` — events the shell emits (observed by agents).
//! - Error types and shared identifiers.
//!
//! Zero runtime dependencies beyond `url`.

use std::fmt;

pub use url::Url;

// ---------------------------------------------------------------------------
// Error types — shared across shell and agent
// ---------------------------------------------------------------------------

pub type ShellResult<T> = Result<T, ShellError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellError {
    InvalidInput(String),
    Unsupported(String),
    Runtime(String),
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
            Self::Unsupported(msg) => write!(f, "unsupported: {msg}"),
            Self::Runtime(msg) => write!(f, "runtime error: {msg}"),
        }
    }
}

impl std::error::Error for ShellError {}

// ---------------------------------------------------------------------------
// Identifiers
// ---------------------------------------------------------------------------

/// Opaque identifier for a web view / tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WebViewId(pub u64);

/// Page load lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadStatus {
    Idle,
    Loading,
    Loaded,
    Failed,
}

// ---------------------------------------------------------------------------
// Command layer — shared entry point for UI and agent
// ---------------------------------------------------------------------------

/// Every action the shell can perform.
///
/// Both the UI (button clicks, keyboard shortcuts) and the agent runtime
/// route through this enum via `YoceShell::dispatch()`.
#[derive(Clone, Debug)]
pub enum ShellCommand {
    // -- Navigation --
    Navigate(String),
    Reload,
    Back,

    // -- Tabs --
    NewTab,
    CloseActiveTab,
    SwitchTab(u64),

    // -- Address bar --
    CommitAddress,
    FocusAddress,
    BlurAddress,
}

// ---------------------------------------------------------------------------
// Event layer — observation surface for agents
// ---------------------------------------------------------------------------

/// Events the shell emits after executing a command.
///
/// Agents observe these events to track tab state, navigation, and UI changes.
#[derive(Clone, Debug)]
pub enum ShellEvent {
    TabCreated { id: u64, url: String },
    TabClosed { id: u64 },
    TabSwitched { id: u64 },
    Navigated { url: String },
    Reloaded,
    BackNavigated { result: Result<(), String> },
    AddressFocused,
    AddressBlurred,
    StatusChanged(String),
}
