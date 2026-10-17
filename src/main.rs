#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod file;
mod reader;
mod render_block;
mod render_table;
mod render_text;
mod selection;
mod selection_text;
mod text_background;
mod text_hit_test;
mod text_map;
mod timing;
mod viewer;

use gpui::{App, AppContext, Application, Bounds, WindowBounds, WindowOptions, px, size};
use timing::Timing;
use viewer::Viewer;

fn main() {
    let timing = Timing::start();
    let initial_path = std::env::args_os().nth(1).map(std::path::PathBuf::from);
    Application::new().run(move |cx: &mut App| {
        viewer::bind_keys(cx);
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
                let viewer = cx.new(|cx| Viewer::new(window, cx, timing));
                if let Some(path) = initial_path {
                    let viewer = viewer.clone();
                    window.on_next_frame(move |window, cx| {
                        viewer.update(cx, |viewer, cx| viewer.load(path, window, cx));
                    });
                }
                viewer
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
