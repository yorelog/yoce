use gpui::{ClickEvent, Context, KeyDownEvent, Window};

use super::yoce_shell::YoceShell;

impl YoceShell {
    pub fn on_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();

        if event.keystroke.modifiers.control && key == "l" {
            self.dispatch(yoce_engine::ShellCommand::FocusAddress, cx);
            return;
        }

        if event.keystroke.modifiers.control && key == "t" {
            self.dispatch(yoce_engine::ShellCommand::NewTab, cx);
            return;
        }

        if event.keystroke.modifiers.control && key == "w" {
            self.dispatch(yoce_engine::ShellCommand::CloseActiveTab, cx);
            return;
        }

        if event.keystroke.modifiers.control && key == "r" {
            self.dispatch(yoce_engine::ShellCommand::Reload, cx);
            return;
        }

        if event.keystroke.modifiers.control && key == "b" {
            self.agent_visible = !self.agent_visible;
            self.status = format!(
                "Agent panel: {}",
                if self.agent_visible {
                    "shown"
                } else {
                    "hidden"
                }
            );
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
            if let Some(text) = super::selected_text(&self.address_input, self.address_selection) {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
                self.status = "Address copied".to_string();
                cx.notify();
            }
            return;
        }

        if event.keystroke.modifiers.control && key == "x" {
            if let Some(text) = super::selected_text(&self.address_input, self.address_selection) {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
                let _ = super::delete_selection(
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
                super::replace_selection_or_insert(
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
            self.dispatch(yoce_engine::ShellCommand::CommitAddress, cx);
            return;
        }

        if key == "escape" {
            self.dispatch(yoce_engine::ShellCommand::BlurAddress, cx);
            return;
        }

        if key == "left" {
            if let Some((start, _)) = super::normalized_selection(self.address_selection) {
                self.address_cursor = start;
                self.address_selection = None;
            } else {
                self.address_cursor =
                    super::prev_char_boundary(&self.address_input, self.address_cursor);
            }
            cx.notify();
            return;
        }

        if key == "right" {
            if let Some((_, end)) = super::normalized_selection(self.address_selection) {
                self.address_cursor = end;
                self.address_selection = None;
            } else {
                self.address_cursor =
                    super::next_char_boundary(&self.address_input, self.address_cursor);
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
            let deleted = super::delete_selection(
                &mut self.address_input,
                &mut self.address_cursor,
                &mut self.address_selection,
            );
            if !deleted && self.address_cursor > 0 {
                let prev = super::prev_char_boundary(&self.address_input, self.address_cursor);
                self.address_input
                    .replace_range(prev..self.address_cursor, "");
                self.address_cursor = prev;
            }
            cx.notify();
            return;
        }

        if key == "delete" {
            let deleted = super::delete_selection(
                &mut self.address_input,
                &mut self.address_cursor,
                &mut self.address_selection,
            );
            if !deleted && self.address_cursor < self.address_input.len() {
                let next = super::next_char_boundary(&self.address_input, self.address_cursor);
                self.address_input
                    .replace_range(self.address_cursor..next, "");
            }
            cx.notify();
            return;
        }

        if event.keystroke.modifiers.control {
            return;
        }

        if let Some(chars) = event.keystroke.key_char.as_ref() {
            super::replace_selection_or_insert(
                &mut self.address_input,
                &mut self.address_cursor,
                &mut self.address_selection,
                chars,
            );
            cx.notify();
        }
    }

    pub fn on_address_click(
        &mut self,
        _event: &ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus_handle, cx);
        self.address_focused = true;
        self.address_cursor = self.address_input.len();
        self.address_selection = None;
        self.status = "Address focused".to_string();
        cx.notify();
    }

    pub fn address_display_text(&self) -> String {
        if let Some((start, end)) = super::normalized_selection(self.address_selection) {
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
