mod shell;

use gpui::{px, App, Application, Bounds, Pixels, Size, WindowBounds, WindowOptions};

fn main() {
    #[cfg(target_os = "windows")]
    {
        // Required by gpui + child webview composition on Windows.
        std::env::set_var("GPUI_DISABLE_DIRECT_COMPOSITION", "true");
    }

    Application::new().run(|cx: &mut App| {
        let bounds: Bounds<Pixels> = Bounds::centered(None, Size::new(px(1200.0), px(820.0)), cx);

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
            shell::build_root,
        )
        .expect("open yoce window");

        cx.activate(true);
    });
}
