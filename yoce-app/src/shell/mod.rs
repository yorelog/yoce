mod keyboard;
mod polling;
pub mod yoce_shell;

pub use yoce_shell::YoceShell;

// ---------------------------------------------------------------------------
// Re-export build_root dispatcher (used by main.rs)
// ---------------------------------------------------------------------------

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn build_root(window: &mut gpui::Window, cx: &mut gpui::App) -> gpui::Entity<YoceShell> {
    YoceShell::new(window, cx)
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn build_root(_: &mut gpui::Window, cx: &mut gpui::App) -> gpui::Entity<UnsupportedShell> {
    cx.new(|cx| UnsupportedShell {
        focus_handle: cx.focus_handle(),
    })
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub struct UnsupportedShell {
    focus_handle: gpui::FocusHandle,
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
impl gpui::Focusable for UnsupportedShell {
    fn focus_handle(&self, _cx: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
impl gpui::Render for UnsupportedShell {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        gpui::div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child("Yoce currently supports Windows and macOS.")
    }
}

// ---------------------------------------------------------------------------
// Address-bar utility functions
// ---------------------------------------------------------------------------

/// Derive a short display title from a URL.
pub fn title_from_url(url: &str) -> String {
    let trimmed = url.trim();
    let no_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .unwrap_or(trimmed);
    no_scheme.split('/').next().unwrap_or("Tab").to_string()
}

/// Prepend `https://` if no scheme is present; fallback to example.com on empty input.
pub fn normalize_url_input(input: &str) -> String {
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

/// Return the selection range with `start <= end`, or `None` if empty.
pub fn normalized_selection(selection: Option<(usize, usize)>) -> Option<(usize, usize)> {
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

/// Return the text within the selection range, if any.
pub fn selected_text(input: &str, selection: Option<(usize, usize)>) -> Option<String> {
    let (start, end) = normalized_selection(selection)?;
    Some(input[start..end].to_string())
}

/// Replace the current selection with `text`, or insert `text` at the cursor.
pub fn replace_selection_or_insert(
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

/// Delete the selected range. Returns `true` if something was actually deleted.
pub fn delete_selection(
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

/// Move one UTF-8 char boundary left from `idx`.
pub fn prev_char_boundary(s: &str, idx: usize) -> usize {
    if idx == 0 {
        return 0;
    }

    let mut i = idx - 1;
    while !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// Move one UTF-8 char boundary right from `idx`.
pub fn next_char_boundary(s: &str, idx: usize) -> usize {
    if idx >= s.len() {
        return s.len();
    }

    let mut i = idx + 1;
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}
