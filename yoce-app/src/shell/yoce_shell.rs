use std::sync::{Arc, Mutex};

use gpui::{
    div, prelude::*, px, rgb, App, ClickEvent, Context, Entity, FocusHandle, Focusable,
    IntoElement, Render, SharedString, Styled, Window,
};

use gpui_wry::WebView;
use raw_window_handle::HasWindowHandle;

use crate::components::button;
use crate::state::{NavState, TabState};

use super::polling::poll_nav_state;
use yoce_engine::{ShellCommand, ShellEvent};

pub struct YoceShell {
    pub focus_handle: FocusHandle,
    pub webview: Option<Entity<WebView>>,
    pub tabs: Vec<TabState>,
    pub active_tab_index: usize,
    pub next_tab_id: u64,
    pub address_input: String,
    pub address_cursor: usize,
    pub address_selection: Option<(usize, usize)>,
    pub address_focused: bool,
    pub status: String,
    pub nav_state: Arc<Mutex<NavState>>,
    pub agent_visible: bool,
    pub agent_panel: Entity<crate::agent::AgentPanel>,
    pub agent_store: Entity<crate::agent::store::AgentStore>,
    pub log_store: Entity<crate::agent::log::LogStore>,
}

impl YoceShell {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let nav_state = Arc::new(Mutex::new(NavState {
            pending_url: None,
            pending_title: None,
        }));

        let initial_url = "https://example.com".to_string();

        let agent_store = crate::agent::store::AgentStore::create(cx);
        let agent_panel = crate::agent::AgentPanel::new(agent_store.clone(), cx);
        let log_store = crate::agent::log::LogStore::create(cx);

        log_store.update(cx, |store, _| {
            store.log(log::Level::Info, "yoce", "Shell initialized");
            store.log(log::Level::Info, "yoce", format!("Loading initial URL: {}", initial_url));
        });

        let shell = cx.new(|cx| Self {
            focus_handle: cx.focus_handle(),
            webview: None,
            agent_panel,
            agent_store,
            log_store,
            agent_visible: false,
            tabs: vec![TabState {
                id: 1,
                title: crate::shell::title_from_url(&initial_url),
                url: initial_url.clone(),
            }],
            active_tab_index: 0,
            next_tab_id: 2,
            address_input: initial_url.clone(),
            address_cursor: initial_url.len(),
            address_selection: None,
            address_focused: false,
            status: "Yoce Browser demo running in-app".to_string(),
            nav_state: nav_state.clone(),
        });

        let _ = (window, nav_state.clone(), initial_url.clone());

        poll_nav_state(shell.downgrade(), nav_state.clone(), cx);

        shell
    }

    pub fn sync_nav_state(&mut self) {
        let (pending_url, pending_title) = {
            let mut nav = self.nav_state.lock().unwrap();
            (nav.pending_url.take(), nav.pending_title.take())
        };

        let mut changed = false;

        if let Some(url) = pending_url {
            if let Some(tab) = self.active_tab_mut() {
                tab.url = url;
                changed = true;
            }
        }
        if let Some(title) = pending_title {
            if let Some(tab) = self.active_tab_mut() {
                tab.title = title;
                changed = true;
            }
        }

        if changed {
            self.sync_address_from_active_tab();
        }
    }

    pub fn active_tab(&self) -> Option<&TabState> {
        self.tabs.get(self.active_tab_index)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut TabState> {
        self.tabs.get_mut(self.active_tab_index)
    }

    pub fn sync_address_from_active_tab(&mut self) {
        let active_url = self
            .active_tab()
            .map(|tab| tab.url.clone())
            .unwrap_or_else(|| "https://example.com".to_string());
        self.address_input = active_url;
        self.address_cursor = self.address_input.len();
        self.address_selection = None;
    }

    pub fn navigate(&mut self, url: &str, cx: &mut Context<Self>) {
        let normalized = crate::shell::normalize_url_input(url);
        self.status = format!("Navigate: {normalized}");
        if let Some(webview) = &self.webview {
            webview.update(cx, |view, _| view.load_url(&normalized));
        } else {
            self.status = "WebView is still initializing".to_string();
        }
        if let Some(tab) = self.active_tab_mut() {
            tab.url = normalized.clone();
            tab.title = crate::shell::title_from_url(&normalized);
        }
    }

    pub fn commit_address_navigation(&mut self, cx: &mut Context<Self>) {
        let url = crate::shell::normalize_url_input(&self.address_input);
        self.address_input = url.clone();
        self.address_cursor = self.address_input.len();
        self.address_selection = None;
        self.address_focused = false;
        self.navigate(&url, cx);
    }

    pub fn reload(&mut self, cx: &mut Context<Self>) {
        let Some(webview) = &self.webview else {
            self.status = "WebView is still initializing".to_string();
            return;
        };
        let result = webview.update(cx, |view, _| view.evaluate_script("location.reload();"));
        self.status = match result {
            Ok(()) => "Reload".to_string(),
            Err(err) => format!("Reload failed: {err}"),
        };
    }

    /// Route a `ShellCommand` to the appropriate handler and emit the
    /// corresponding `ShellEvent`.
    ///
    /// This is the single entry point for both UI and future agent code.
    pub fn dispatch(&mut self, cmd: ShellCommand, cx: &mut Context<Self>) -> Option<ShellEvent> {
        let event = match cmd {
            ShellCommand::Navigate(url) => {
                self.navigate(&url, cx);
                ShellEvent::Navigated { url }
            }
            ShellCommand::Reload => {
                self.reload(cx);
                ShellEvent::Reloaded
            }
            ShellCommand::Back => {
                let Some(webview) = &self.webview else {
                    self.status = "WebView is still initializing".to_string();
                    return None;
                };
                let result = webview.update(cx, |view, _| view.back());
                self.status = match result {
                    Ok(()) => "Back".to_string(),
                    Err(ref err) => format!("Back failed: {err}"),
                };
                ShellEvent::BackNavigated {
                    result: result.map_err(|e| format!("{e}")),
                }
            }
            ShellCommand::NewTab => {
                let id = self.next_tab_id;
                let url = "https://example.com".to_string();
                let tab = crate::state::TabState {
                    id,
                    title: format!("Tab {id}"),
                    url: url.clone(),
                };
                self.next_tab_id += 1;
                self.tabs.push(tab);
                self.active_tab_index = self.tabs.len() - 1;
                self.sync_address_from_active_tab();
                self.navigate(&url, cx);
                self.status = format!("New tab: {id}");
                ShellEvent::TabCreated { id, url }
            }
            ShellCommand::CloseActiveTab => {
                let closed_id = self.active_tab().map(|t| t.id);
                if self.tabs.len() <= 1 {
                    self.status = "At least one tab must remain".to_string();
                    return None;
                }
                self.tabs.remove(self.active_tab_index);
                if self.active_tab_index >= self.tabs.len() {
                    self.active_tab_index = self.tabs.len() - 1;
                }
                let url = self
                    .active_tab()
                    .map(|t| t.url.clone())
                    .unwrap_or_else(|| "https://example.com".to_string());
                self.sync_address_from_active_tab();
                self.navigate(&url, cx);
                self.status = format!("Closed tab: {:?}", closed_id);
                ShellEvent::TabClosed {
                    id: closed_id.unwrap_or(0),
                }
            }
            ShellCommand::SwitchTab(tab_id) => {
                if let Some(index) = self.tabs.iter().position(|tab| tab.id == tab_id) {
                    self.active_tab_index = index;
                    let url = self
                        .active_tab()
                        .map(|t| t.url.clone())
                        .unwrap_or_else(|| "https://example.com".to_string());
                    self.sync_address_from_active_tab();
                    self.navigate(&url, cx);
                    self.status = format!("Switched to tab: {tab_id}");
                    ShellEvent::TabSwitched { id: tab_id }
                } else {
                    self.status = format!("Tab {tab_id} not found");
                    return None;
                }
            }
            ShellCommand::CommitAddress => {
                self.commit_address_navigation(cx);
                ShellEvent::AddressBlurred
            }
            ShellCommand::FocusAddress => {
                self.address_focused = true;
                self.address_cursor = self.address_input.len();
                self.address_selection = None;
                self.status = "Address focused".to_string();
                ShellEvent::AddressFocused
            }
            ShellCommand::BlurAddress => {
                self.address_focused = false;
                self.address_selection = None;
                self.status = "Address unfocused".to_string();
                ShellEvent::AddressBlurred
            }
        };
        cx.notify();

        // Log.
        log::info!("dispatch → {:?}", event);
        self.log_store.update(cx, |store, _| {
            store.log(log::Level::Info, "shell", format!("{:?}", event));
        });

        // Forward event to agent store.
        self.agent_store.update(cx, |store, cx| {
            store.push_event(&event);
            cx.notify();
        });

        Some(event)
    }
}

impl Focusable for YoceShell {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for YoceShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_nav_state();

        let toolbar = div()
            .h(px(48.0))
            .w_full()
            .px_3()
            .flex()
            .items_center()
            .gap_2()
            .bg(rgb(0x132030))
            .border_b_1()
            .border_color(rgb(0x203247))
            .child(button(
                "Back",
                cx.listener(|this, _evt: &ClickEvent, _window, cx| {
                    this.dispatch(yoce_engine::ShellCommand::Back, cx);
                }),
            ))
            .child(button(
                "Reload",
                cx.listener(|this, _evt: &ClickEvent, _window, cx| {
                    this.dispatch(yoce_engine::ShellCommand::Reload, cx);
                }),
            ))
            .child(button(
                "New Tab",
                cx.listener(|this, _evt: &ClickEvent, _window, cx| {
                    this.dispatch(yoce_engine::ShellCommand::NewTab, cx);
                }),
            ))
            .child(button(
                "Close Tab",
                cx.listener(|this, _evt: &ClickEvent, _window, cx| {
                    this.dispatch(yoce_engine::ShellCommand::CloseActiveTab, cx);
                }),
            ))
            .child(button(
                "GPUI Docs",
                cx.listener(|this, _evt: &ClickEvent, _window, cx| {
                    this.dispatch(
                        yoce_engine::ShellCommand::Navigate(
                            "https://longbridge.github.io/gpui-component".into(),
                        ),
                        cx,
                    );
                }),
            ))
            .child(button(
                "GitHub",
                cx.listener(|this, _evt: &ClickEvent, _window, cx| {
                    this.dispatch(
                        yoce_engine::ShellCommand::Navigate("https://github.com".into()),
                        cx,
                    );
                }),
            ))
            .child(button(
                "Example",
                cx.listener(|this, _evt: &ClickEvent, _window, cx| {
                    this.dispatch(
                        yoce_engine::ShellCommand::Navigate("https://example.com".into()),
                        cx,
                    );
                }),
            ))
            .child(button(
                "Agent",
                cx.listener(|this, _evt: &ClickEvent, _window, cx| {
                    this.agent_visible = !this.agent_visible;
                    this.status = format!(
                        "Agent panel: {}",
                        if this.agent_visible {
                            "shown"
                        } else {
                            "hidden"
                        }
                    );
                    cx.notify();
                }),
            ))
            .child(
                div()
                    .ml_3()
                    .text_xs()
                    .text_color(rgb(0x9fb2c9))
                    .child("Ctrl+L/T/W/R/B and Ctrl+A/C/X/V in address"),
            );

        let tab_strip = div()
            .h(px(40.0))
            .w_full()
            .px_3()
            .flex()
            .items_center()
            .gap_2()
            .bg(rgb(0x0f1926))
            .border_b_1()
            .border_color(rgb(0x203247))
            .children(self.tabs.iter().enumerate().map(|(index, tab)| {
                let tab_id = tab.id;
                let is_active = index == self.active_tab_index;
                let bg_color = if is_active {
                    rgb(0x2a4461)
                } else {
                    rgb(0x1a2c40)
                };
                let label = format!("{} {}", tab.id, tab.title);
                div()
                    .id(SharedString::from(format!("tab-{}", tab.id)))
                    .px_2()
                    .py_1()
                    .text_sm()
                    .cursor_pointer()
                    .bg(bg_color)
                    .border_1()
                    .border_color(rgb(0x31506d))
                    .text_color(rgb(0xd8e3ef))
                    .rounded_sm()
                    .child(label)
                    .on_click(cx.listener(move |this, _evt: &ClickEvent, _window, cx| {
                        this.dispatch(yoce_engine::ShellCommand::SwitchTab(tab_id), cx);
                    }))
            }));

        let address_bar = div()
            .h(px(44.0))
            .w_full()
            .px_3()
            .flex()
            .items_center()
            .gap_2()
            .bg(rgb(0x101927))
            .border_b_1()
            .border_color(rgb(0x203247))
            .child(
                div()
                    .id(SharedString::from("address-input"))
                    .h(px(30.0))
                    .flex_1()
                    .px_2()
                    .items_center()
                    .text_sm()
                    .bg(rgb(0x0f1a27))
                    .border_1()
                    .border_color(rgb(0x31506d))
                    .text_color(rgb(0xd8e3ef))
                    .rounded_sm()
                    .child(self.address_display_text())
                    .on_click(cx.listener(|this, evt, window, cx| {
                        this.on_address_click(evt, window, cx);
                    })),
            )
            .child(button(
                "Go",
                cx.listener(|this, _evt: &ClickEvent, _window, cx| {
                    this.dispatch(yoce_engine::ShellCommand::CommitAddress, cx);
                }),
            ));

        let status = div()
            .h(px(30.0))
            .w_full()
            .px_3()
            .flex()
            .items_center()
            .bg(rgb(0x0e1722))
            .border_b_1()
            .border_color(rgb(0x203247))
            .text_sm()
            .text_color(rgb(0xc7d4e5))
            .child(self.status.clone());

        div()
            .id(SharedString::from("yoce-shell"))
            .size_full()
            .flex()
            .flex_col()
            .key_context("yoce-shell")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event, _window, cx| {
                this.on_key_down(event, cx);
            }))
            .bg(rgb(0x0a111a))
            .child(toolbar)
            .child(tab_strip)
            .child(address_bar)
            .child(status)
            .child({
                let webview_area = div().flex_1().w_full().p_3().child(
                    div()
                        .size_full()
                        .border_1()
                        .border_color(rgb(0x2a3f58))
                        .rounded_sm()
                        .child(if let Some(webview) = &self.webview {
                            webview.clone().into_any_element()
                        } else {
                            div()
                                .size_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_sm()
                                .text_color(rgb(0x9fb2c9))
                                .child("Initializing webview...")
                                .into_any_element()
                        }),
                );
                if self.agent_visible {
                    div()
                        .flex_row()
                        .flex_1()
                        .w_full()
                        .child(webview_area)
                        .child(self.agent_panel.clone())
                } else {
                    webview_area
                }
            })
    }
}
