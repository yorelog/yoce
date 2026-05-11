use gpui::{App, AppContext, Entity, SharedString};

use yoce_engine::ShellEvent;

// ---------------------------------------------------------------------------
// Message types
// ---------------------------------------------------------------------------

/// Who sent a message.
#[derive(Clone, PartialEq)]
pub enum MessageRole {
    User,
    System,
}

/// A single chat message.
#[derive(Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: SharedString,
}

impl Message {
    pub fn user(text: impl Into<SharedString>) -> Self {
        Self {
            role: MessageRole::User,
            content: text.into(),
        }
    }

    pub fn system(text: impl Into<SharedString>) -> Self {
        Self {
            role: MessageRole::System,
            content: text.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// AgentStore
// ---------------------------------------------------------------------------

/// Centralised conversation store.
///
/// Both `AgentPanel` and the future agent runtime access conversation
/// history through this store.  It lives as a gpui `Entity` so it can
/// be shared between components.
pub struct AgentStore {
    messages: Vec<Message>,
}

impl AgentStore {
    const MAX_MESSAGES: usize = 200;

    pub fn new() -> Self {
        Self {
            messages: vec![Message::system("Agent Panel ready. Use Ctrl+B to toggle.")],
        }
    }

    pub fn create(cx: &mut App) -> Entity<Self> {
        cx.new(|_| Self::new())
    }

    // -- reads ---------------------------------------------------------------

    /// All messages, newest last.
    #[allow(dead_code)]
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Messages for display, newest first, capped at `n`.
    pub fn recent_messages(&self, n: usize) -> impl Iterator<Item = &Message> {
        self.messages.iter().rev().take(n)
    }

    // -- writes --------------------------------------------------------------

    pub fn add_user_message(&mut self, text: impl Into<SharedString>) {
        self.push(Message::user(text));
    }

    #[allow(dead_code)]
    pub fn add_system_message(&mut self, text: impl Into<SharedString>) {
        self.push(Message::system(text));
    }

    /// Log a `ShellEvent` as a system message.
    pub fn push_event(&mut self, event: &ShellEvent) {
        let text = format!("{:?}", event);
        self.push(Message::system(text));
    }

    // -- internal ------------------------------------------------------------

    fn push(&mut self, msg: Message) {
        self.messages.push(msg);
        if self.messages.len() > Self::MAX_MESSAGES {
            self.messages.remove(0);
        }
    }
}
