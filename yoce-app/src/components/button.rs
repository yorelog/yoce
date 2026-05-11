use gpui::{
    div, rgb, App, ClickEvent, InteractiveElement, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement, Styled, Window,
};

/// A styled clickable button used in the shell toolbar and address bar.
pub fn button(
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
