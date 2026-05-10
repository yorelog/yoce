use gpui::{
    App, ClickEvent, Context, Entity, FocusHandle, Focusable, IntoElement, KeyDownEvent, Render,
    SharedString, Styled, Window, div, prelude::*, px, rgb,
};

#[cfg(target_os = "windows")]
use gpui_wry::WebView;
#[cfg(target_os = "windows")]
use raw_window_handle::HasWindowHandle;

#[cfg(target_os = "windows")]
pub fn build_root(window: &mut Window, cx: &mut App) -> Entity<YoceShell> {
    YoceShell::new(window, cx)
}

#[cfg(not(target_os = "windows"))]
pub fn build_root(_: &mut Window, cx: &mut App) -> Entity<UnsupportedShell> {
    cx.new(|cx| UnsupportedShell {
        focus_handle: cx.focus_handle(),
    })
}

#[cfg(target_os = "windows")]
#[derive(Clone)]
struct TabState {
    id: u64,
    title: String,
    url: String,
}

#[cfg(target_os = "windows")]
pub struct YoceShell {
    focus_handle: FocusHandle,
    webview: Entity<WebView>,
    tabs: Vec<TabState>,
    active_tab_index: usize,
    next_tab_id: u64,
    address_input: String,
    address_cursor: usize,
    address_selection: Option<(usize, usize)>,
    address_focused: bool,
    status: String,
}

#[cfg(target_os = "windows")]
impl YoceShell {
    pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let webview = cx.new(|cx| {
            let builder = wry::WebViewBuilder::new();
            #[cfg(any(debug_assertions, feature = "inspector"))]
            let builder = builder.with_devtools(true);

            let window_handle = window.window_handle().expect("window handle");
            let raw = builder
                .build_as_child(&window_handle)
                .expect("create child webview");

            WebView::new(raw, window, cx)
        });

        let initial_url = "https://example.com".to_string();
        webview.update(cx, |view, _| view.load_url(&initial_url));

        cx.new(|cx| Self {
            focus_handle: cx.focus_handle(),
            webview,
            tabs: vec![TabState {
                id: 1,
                title: title_from_url(&initial_url),
                url: initial_url.clone(),
            }],
            active_tab_index: 0,
            next_tab_id: 2,
            address_input: initial_url.clone(),
            address_cursor: initial_url.len(),
            address_selection: None,
            address_focused: false,
            status: "WebView2 demo running in-app".to_string(),
        })
    }

    fn active_tab(&self) -> Option<&TabState> {
        self.tabs.get(self.active_tab_index)
    }

    fn active_tab_mut(&mut self) -> Option<&mut TabState> {
        self.tabs.get_mut(self.active_tab_index)
    }

    fn sync_address_from_active_tab(&mut self) {
        let active_url = self
            .active_tab()
            .map(|tab| tab.url.clone())
            .unwrap_or_else(|| "https://example.com".to_string());

        self.address_input = active_url;
        self.address_cursor = self.address_input.len();
        self.address_selection = None;
    }

    fn navigate(&mut self, url: &str, cx: &mut Context<Self>) {
        let normalized = normalize_url_input(url);
        self.status = format!("Navigate: {normalized}");
        self.webview.update(cx, |view, _| view.load_url(&normalized));

        if let Some(tab) = self.active_tab_mut() {
            tab.url = normalized.clone();
            tab.title = title_from_url(&normalized);
        }
    }

    fn commit_address_navigation(&mut self, cx: &mut Context<Self>) {
        let url = normalize_url_input(&self.address_input);
        self.address_input = url.clone();
        self.address_cursor = self.address_input.len();
        self.address_selection = None;
        self.address_focused = false;
        self.navigate(&url, cx);
    }

    fn new_tab(&mut self, cx: &mut Context<Self>) {
        let url = "https://example.com".to_string();
        let tab = TabState {
            id: self.next_tab_id,
            title: format!("Tab {}", self.next_tab_id),
            url: url.clone(),
        };
        self.next_tab_id += 1;
        self.tabs.push(tab);
        self.active_tab_index = self.tabs.len() - 1;
        self.sync_address_from_active_tab();
        self.navigate(&url, cx);
        self.status = format!("New tab: {}", self.active_tab().map(|t| t.id).unwrap_or(0));
    }

    fn close_active_tab(&mut self, cx: &mut Context<Self>) {
        if self.tabs.len() <= 1 {
            self.status = "At least one tab must remain".to_string();
            return;
        }

        let closed_id = self.active_tab().map(|t| t.id).unwrap_or(0);
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
        self.status = format!("Closed tab: {closed_id}");
    }

    fn switch_tab(&mut self, tab_id: u64, cx: &mut Context<Self>) {
        if let Some(index) = self.tabs.iter().position(|tab| tab.id == tab_id) {
            self.active_tab_index = index;
            let url = self
                .active_tab()
                .map(|t| t.url.clone())
                .unwrap_or_else(|| "https://example.com".to_string());
            self.sync_address_from_active_tab();
            self.navigate(&url, cx);
            self.status = format!("Switched to tab: {tab_id}");
        }
    }

    fn back(&mut self, cx: &mut Context<Self>) {
        let result = self.webview.update(cx, |view, _| view.back());
        self.status = match result {
            Ok(()) => "Back".to_string(),
            Err(err) => format!("Back failed: {err}"),
        };
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        let result = self.webview.update(cx, |view, _| view.evaluate_script("location.reload();"));
        self.status = match result {
            Ok(()) => "Reload".to_string(),
            Err(err) => format!("Reload failed: {err}"),
        };
    }

    fn open_docs(&mut self, cx: &mut Context<Self>) {
        self.address_input = "https://longbridge.github.io/gpui-component".to_string();
        self.address_cursor = self.address_input.len();
        self.commit_address_navigation(cx);
    }

    fn open_github(&mut self, cx: &mut Context<Self>) {
        self.address_input = "https://github.com".to_string();
        self.address_cursor = self.address_input.len();
        self.commit_address_navigation(cx);
    }

    fn open_example(&mut self, cx: &mut Context<Self>) {
        self.address_input = "https://example.com".to_string();
        self.address_cursor = self.address_input.len();
        self.commit_address_navigation(cx);
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();

        if event.keystroke.modifiers.control && key == "l" {
            self.address_focused = true;
            self.address_cursor = self.address_input.len();
            self.address_selection = None;
            self.status = "Address focused".to_string();
            cx.notify();
            return;
        }

        if event.keystroke.modifiers.control && key == "t" {
            self.new_tab(cx);
            cx.notify();
            return;
        }

        if event.keystroke.modifiers.control && key == "w" {
            self.close_active_tab(cx);
            cx.notify();
            return;
        }

        if event.keystroke.modifiers.control && key == "r" {
            self.reload(cx);
            cx.notify();
            return;
        }

        if !self.address_focused {
            return;
        }

        if event.keystroke.modifiers.control && key == "a" {
            if !self.address_input.is_empty() {
                self.address_selection = Some((0, self.address_input.len()));
                self.address_cursor = self.address_input.len();
                cx.notify();
            }
            return;
        }

        if event.keystroke.modifiers.control && key == "c" {
            if let Some(text) = selected_text(&self.address_input, self.address_selection) {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
                self.status = "Address copied".to_string();
                cx.notify();
            }
            return;
        }

        if event.keystroke.modifiers.control && key == "x" {
            if let Some(text) = selected_text(&self.address_input, self.address_selection) {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
                let _ = delete_selection(
                    &mut self.address_input,
                    &mut self.address_cursor,
                    &mut self.address_selection,
                );
                self.status = "Address cut".to_string();
                cx.notify();
            }
            return;
        }

        if event.keystroke.modifiers.control && key == "v" {
            if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                replace_selection_or_insert(
                    &mut self.address_input,
                    &mut self.address_cursor,
                    &mut self.address_selection,
                    &text.replace('\n', " "),
                );
                self.status = "Address pasted".to_string();
                cx.notify();
            }
            return;
        }

        if key == "enter" {
            self.commit_address_navigation(cx);
            cx.notify();
            return;
        }

        if key == "escape" {
            self.address_focused = false;
            self.address_selection = None;
            self.status = "Address unfocused".to_string();
            cx.notify();
            return;
        }

        if key == "left" {
            if let Some((start, _)) = normalized_selection(self.address_selection) {
                self.address_cursor = start;
                self.address_selection = None;
            } else {
                self.address_cursor = prev_char_boundary(&self.address_input, self.address_cursor);
            }
            cx.notify();
            return;
        }

        if key == "right" {
            if let Some((_, end)) = normalized_selection(self.address_selection) {
                self.address_cursor = end;
                self.address_selection = None;
            } else {
                self.address_cursor = next_char_boundary(&self.address_input, self.address_cursor);
            }
            cx.notify();
            return;
        }

        if key == "home" {
            self.address_cursor = 0;
            self.address_selection = None;
            cx.notify();
            return;
        }

        if key == "end" {
            self.address_cursor = self.address_input.len();
            self.address_selection = None;
            cx.notify();
            return;
        }

        if key == "backspace" {
            let deleted = delete_selection(
                &mut self.address_input,
                &mut self.address_cursor,
                &mut self.address_selection,
            );
            if !deleted && self.address_cursor > 0 {
                let prev = prev_char_boundary(&self.address_input, self.address_cursor);
                self.address_input.replace_range(prev..self.address_cursor, "");
                self.address_cursor = prev;
            }
            cx.notify();
            return;
        }

        if key == "delete" {
            let deleted = delete_selection(
                &mut self.address_input,
                &mut self.address_cursor,
                &mut self.address_selection,
            );
            if !deleted && self.address_cursor < self.address_input.len() {
                let next = next_char_boundary(&self.address_input, self.address_cursor);
                self.address_input.replace_range(self.address_cursor..next, "");
            }
            cx.notify();
            return;
        }

        if event.keystroke.modifiers.control {
            return;
        }

        if let Some(chars) = event.keystroke.key_char.as_ref() {
            replace_selection_or_insert(
                &mut self.address_input,
                &mut self.address_cursor,
                &mut self.address_selection,
                chars,
            );
            cx.notify();
        }
    }

    fn on_address_click(
        &mut self,
        _event: &ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus_handle);
        self.address_focused = true;
        self.address_cursor = self.address_input.len();
        self.address_selection = None;
        self.status = "Address focused".to_string();
        cx.notify();
    }

    fn address_display_text(&self) -> String {
        if let Some((start, end)) = normalized_selection(self.address_selection) {
            let (left, rest) = self.address_input.split_at(start);
            let (selected, right) = rest.split_at(end - start);
            return format!("{}[{}]{}", left, selected, right);
        }

        if !self.address_focused {
            return self.address_input.clone();
        }

        let cursor = self.address_cursor.min(self.address_input.len());
        let (left, right) = self.address_input.split_at(cursor);
        format!("{}|{}", left, right)
    }
}

#[cfg(target_os = "windows")]
impl Focusable for YoceShell {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

#[cfg(target_os = "windows")]
impl Render for YoceShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
            .child(button("Back", cx.listener(|this, _evt: &ClickEvent, _window, cx| {
                this.back(cx);
                cx.notify();
            })))
            .child(button("Reload", cx.listener(|this, _evt: &ClickEvent, _window, cx| {
                this.reload(cx);
                cx.notify();
            })))
            .child(button("New Tab", cx.listener(|this, _evt: &ClickEvent, _window, cx| {
                this.new_tab(cx);
                cx.notify();
            })))
            .child(button("Close Tab", cx.listener(|this, _evt: &ClickEvent, _window, cx| {
                this.close_active_tab(cx);
                cx.notify();
            })))
            .child(button("GPUI Docs", cx.listener(|this, _evt: &ClickEvent, _window, cx| {
                this.open_docs(cx);
                cx.notify();
            })))
            .child(button("GitHub", cx.listener(|this, _evt: &ClickEvent, _window, cx| {
                this.open_github(cx);
                cx.notify();
            })))
            .child(button("Example", cx.listener(|this, _evt: &ClickEvent, _window, cx| {
                this.open_example(cx);
                cx.notify();
            })))
            .child(
                div()
                    .ml_3()
                    .text_xs()
                    .text_color(rgb(0x9fb2c9))
                    .child("Ctrl+L/T/W/R and Ctrl+A/C/X/V in address"),
            );

        let tab_strip = div().h(px(40.0)).w_full().px_3().flex().items_center().gap_2().bg(rgb(0x0f1926)).border_b_1().border_color(rgb(0x203247)).children(
            self.tabs.iter().enumerate().map(|(index, tab)| {
                let tab_id = tab.id;
                let is_active = index == self.active_tab_index;
                let bg_color = if is_active { rgb(0x2a4461) } else { rgb(0x1a2c40) };
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
                        this.switch_tab(tab_id, cx);
                        cx.notify();
                    }))
            }),
        );

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
            .child(button("Go", cx.listener(|this, _evt: &ClickEvent, _window, cx| {
                this.commit_address_navigation(cx);
                cx.notify();
            })));

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
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .p_3()
                    .child(
                        div()
                            .size_full()
                            .border_1()
                            .border_color(rgb(0x2a3f58))
                            .rounded_sm()
                            .child(self.webview.clone()),
                    ),
            )
    }
}

fn title_from_url(url: &str) -> String {
    let trimmed = url.trim();
    let no_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .unwrap_or(trimmed);
    no_scheme
        .split('/')
        .next()
        .unwrap_or("Tab")
        .to_string()
}

fn normalize_url_input(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return "https://example.com".to_string();
    }

    if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    }
}

fn normalized_selection(selection: Option<(usize, usize)>) -> Option<(usize, usize)> {
    selection.and_then(|(a, b)| {
        if a == b {
            None
        } else if a < b {
            Some((a, b))
        } else {
            Some((b, a))
        }
    })
}

fn selected_text(input: &str, selection: Option<(usize, usize)>) -> Option<String> {
    let (start, end) = normalized_selection(selection)?;
    Some(input[start..end].to_string())
}

fn replace_selection_or_insert(
    input: &mut String,
    cursor: &mut usize,
    selection: &mut Option<(usize, usize)>,
    text: &str,
) {
    if let Some((start, end)) = normalized_selection(*selection) {
        input.replace_range(start..end, text);
        *cursor = start + text.len();
    } else {
        input.insert_str(*cursor, text);
        *cursor += text.len();
    }
    *selection = None;
}

fn delete_selection(
    input: &mut String,
    cursor: &mut usize,
    selection: &mut Option<(usize, usize)>,
) -> bool {
    if let Some((start, end)) = normalized_selection(*selection) {
        input.replace_range(start..end, "");
        *cursor = start;
        *selection = None;
        return true;
    }
    false
}

fn prev_char_boundary(s: &str, idx: usize) -> usize {
    if idx == 0 {
        return 0;
    }

    let mut i = idx - 1;
    while !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn next_char_boundary(s: &str, idx: usize) -> usize {
    if idx >= s.len() {
        return s.len();
    }

    let mut i = idx + 1;
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

fn button(
    label: &'static str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(SharedString::from(format!("btn-{label}")))
        .px_2()
        .py_1()
        .text_sm()
        .cursor_pointer()
        .bg(rgb(0x1d2f42))
        .border_1()
        .border_color(rgb(0x2e4864))
        .text_color(rgb(0xd8e3ef))
        .rounded_sm()
        .child(label)
        .on_click(on_click)
}

#[cfg(not(target_os = "windows"))]
pub struct UnsupportedShell {
    focus_handle: FocusHandle,
}

#[cfg(not(target_os = "windows"))]
impl Focusable for UnsupportedShell {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

#[cfg(not(target_os = "windows"))]
impl Render for UnsupportedShell {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child("Yoce demo currently supports embedded webview on Windows.")
    }
}
