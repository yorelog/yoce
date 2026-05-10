//! Shared browser engine contracts for Yoce rebuild.

use std::{error::Error, fmt, rc::Rc};

pub use url::Url;

pub type EngineResult<T> = Result<T, EngineError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineError {
    InvalidInput(String),
    Unsupported(String),
    Runtime(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
            Self::Unsupported(msg) => write!(f, "unsupported: {msg}"),
            Self::Runtime(msg) => write!(f, "runtime error: {msg}"),
        }
    }
}

impl Error for EngineError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WebViewId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadStatus {
    Idle,
    Loading,
    Loaded,
    Failed,
}

#[derive(Debug, Clone)]
pub enum EngineEvent {
    TitleChanged {
        id: WebViewId,
        title: String,
    },
    UrlChanged {
        id: WebViewId,
        url: Url,
    },
}

pub trait Engine {
    fn name(&self) -> &'static str;
}

pub trait BrowserEngine: Engine {
    fn create_webview(&self, url: Url, hidpi_scale: f32) -> EngineResult<Rc<dyn WebViewHandle>>;
    fn spin_event_loop(&self);
    fn drain_events(&self) -> Vec<EngineEvent>;
}

pub trait WebViewHandle {
    fn id(&self) -> WebViewId;
    fn load(&self, url: Url) -> EngineResult<()>;
    fn reload(&self) -> EngineResult<()>;
    fn go_back(&self) -> EngineResult<()>;
    fn go_forward(&self) -> EngineResult<()>;
    fn url(&self) -> Option<Url>;
    fn page_title(&self) -> Option<String>;
    fn load_status(&self) -> LoadStatus;
    fn can_go_back(&self) -> bool;
    fn can_go_forward(&self) -> bool;
    fn resize(&self, width: u32, height: u32) -> EngineResult<()>;
    fn set_bounds(&self, x: i32, y: i32, width: u32, height: u32) -> EngineResult<()>;
    fn show(&self) -> EngineResult<()>;
    fn hide(&self) -> EngineResult<()>;
    fn focus(&self) -> EngineResult<()>;
    fn blur(&self) -> EngineResult<()>;
}
