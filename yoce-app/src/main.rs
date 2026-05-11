mod agent;
mod components;
mod shell;
mod state;

use gpui::{px, App, AppContext, Bounds, Pixels, Size, WindowBounds, WindowOptions};

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .init();


    gpui_platform::application()
        .with_assets(gpui_component_assets::Assets)
        .run(|cx: &mut App| {
            gpui_component::init(cx);

            let bounds: Bounds<Pixels> =
                Bounds::centered(None, Size::new(px(1200.0), px(820.0)), cx);

            cx.open_window(
                WindowOptions {
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some("Yoce Agent Browser".into()),
                        appears_transparent: false,
                        ..Default::default()
                    }),
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window: &mut gpui::Window, cx: &mut App| {
                    let shell = shell::build_root(window, cx);
                    cx.new(|cx| gpui_component::Root::new(shell, window, cx))
                },
            )
            .expect("open yoce window");

            cx.activate(true);
        });
}
