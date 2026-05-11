use gpui::{
    div, prelude::*, px, rgb, App, Context, Entity, FocusHandle, Focusable, IntoElement,
    KeyDownEvent, Render, SharedString, Styled, Window,
};

use crate::components::button;
use crate::agent::store::{AgentStore, MessageRole};

/// Agent panel — provides input UI; renders messages from `AgentStore`.
pub struct AgentPanel {
    focus_handle: FocusHandle,
    input_text: String,
    store: Entity<AgentStore>,
}

impl AgentPanel {
    pub fn new(store: Entity<AgentStore>, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self {
            focus_handle: cx.focus_handle(),
            input_text: String::new(),
            store,
        })
    }

    pub fn on_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();

        if key == "enter" && !self.input_text.trim().is_empty() {
            let text = std::mem::take(&mut self.input_text);
            self.store.update(cx, |store, cx| {
                store.add_user_message(text);
                cx.notify();
            });
            cx.notify();
            return;
        }

        if key == "backspace" {
            self.input_text.pop();
            cx.notify();
            return;
        }

        if event.keystroke.modifiers.control {
            return;
        }

        if let Some(chars) = event.keystroke.key_char.as_ref() {
            if chars.len() == 1 {
                self.input_text.push_str(chars);
                cx.notify();
            }
        }
    }
}

impl Focusable for AgentPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AgentPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let input_display = if self.input_text.is_empty() {
            "Type a message...".to_string()
        } else {
            format!("{}|", self.input_text)
        };

        // Read messages from the shared store.
        let messages: Vec<(SharedString, SharedString)> = self
            .store
            .read(cx)
            .recent_messages(20)
            .map(|msg| {
                let role_label = match msg.role {
                    MessageRole::User => "You",
                    MessageRole::System => "●",
                };
                (SharedString::from(role_label), msg.content.clone())
            })
            .collect();

        div()
            .id(SharedString::from("agent-panel"))
            .w(px(320.0))
            .h_full()
            .flex()
            .flex_col()
            .border_l_1()
            .border_color(rgb(0x203247))
            .bg(rgb(0x0f1926))
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event, _window, cx| {
                this.on_key_down(event, cx);
            }))
            // --- header ---
            .child(
                div()
                    .h(px(40.0)).w_full().px_3()
                    .flex().items_center()
                    .border_b_1().border_color(rgb(0x203247))
                    .child(
                        div()
                            .text_sm().font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(0xd8e3ef))
                            .child("Agent Panel"),
                    ),
            )
            // --- message list ---
            .child(
                div().flex_1().w_full().px_2().py_2()
                    .children(messages.into_iter().map(|(role, content)| {
                        let role_color = if role.as_str() == "You" {
                            rgb(0x6cc7ff)
                        } else {
                            rgb(0x9fb2c9)
                        };
                        div().py_1()
                            .child(
                                div()
                                    .text_xs().text_color(role_color)
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .child(role),
                            )
                            .child(
                                div()
                                    .text_sm().text_color(rgb(0xd8e3ef))
                                    .child(content),
                            )
                    })),
            )
            // --- input area ---
            .child(
                div()
                    .h(px(44.0)).w_full().px_2()
                    .flex().items_center().gap_2()
                    .border_t_1().border_color(rgb(0x203247))
                    .bg(rgb(0x0a111a))
                    .child(
                        div()
                            .id(SharedString::from("agent-input"))
                            .on_click(cx.listener(|this, _evt, window, cx| {
                                window.focus(&this.focus_handle, cx);
                            }))
                            .h(px(28.0)).flex_1().px_2().items_center().text_sm()
                            .bg(rgb(0x101927)).border_1().border_color(rgb(0x31506d))
                            .text_color(if self.input_text.is_empty() {
                                rgb(0x5a7494)
                            } else {
                                rgb(0xd8e3ef)
                            })
                            .rounded_sm()
                            .cursor_pointer()
                            .child(input_display),
                    )
                    .child(button("Send", cx.listener(|this, _evt, _window, cx| {
                        let text = std::mem::take(&mut this.input_text);
                        if !text.trim().is_empty() {
                            this.store.update(cx, |store, cx| {
                                store.add_user_message(text);
                                cx.notify();
                            });
                            cx.notify();
                        }
                    }))),
            )
    }
}
