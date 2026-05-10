mod shell;

use gpui::{
    App, Application, Bounds, Pixels, Size, WindowBounds, WindowOptions, px,
};

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
            |window, cx| shell::build_root(window, cx),
        )
        .expect("open yoce window");

        cx.activate(true);
    });
}
