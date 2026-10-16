#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod timing;
mod viewer;

use gpui::{App, AppContext, Application, Bounds, WindowBounds, WindowOptions, px, size};
use timing::Timing;
use viewer::Viewer;

fn main() {
    let timing = Timing::start();
    Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1000.), px(760.)), cx);
        let result = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("MIRA".into()),
                    ..Default::default()
                }),
                window_min_size: Some(size(px(360.), px(240.))),
                ..Default::default()
            },
            move |window, cx| {
                timing.mark("window_created");
                window.on_next_frame(move |_, _| timing.mark("first_frame_callback"));
                cx.new(|_| Viewer)
            },
        );
        if let Err(error) = result {
            eprintln!("Cannot open MIRA window: {error}");
            cx.quit();
        }
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        cx.activate(true);
    });
}
