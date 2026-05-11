use std::sync::Arc;

use gpui::{App, WeakEntity};

use crate::state::NavState;

use super::yoce_shell::YoceShell;

/// URL/title polling has been removed due to gpui's AppCell borrow rules.
///
/// The recursive `cx.defer()` pattern caused "RefCell already borrowed" panics
/// because gpui's event loop already holds a mutable borrow on `AppCell` when
/// executing deferred callbacks.  Calling `e.update(cx, ...)` inside the
/// callback tried to acquire a second mutable borrow.
///
/// `sync_nav_state()` is called at the top of `render()` instead and only
/// applies pending URL/title data without triggering nested notifications.
#[allow(dead_code)]
pub fn poll_nav_state(
    _entity: WeakEntity<YoceShell>,
    _nav: Arc<std::sync::Mutex<NavState>>,
    _cx: &mut App,
) {
}
