#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod appearance;
mod code_cache;
mod code_glyphs;
mod code_panel;
mod code_view;
mod content_paint;
mod controls;
mod file;
mod find_highlight;
mod find_state;
mod flow_cache;
mod flow_view;
mod image_cache;
mod navigation_state;
mod query_input;
mod reader;
mod render_block;
mod render_context;
mod render_image;
mod render_inline;
mod render_table;
mod render_text;
mod reveal;
mod selection;
mod selection_text;
mod shell;
mod table_state;
mod text_background;
mod text_cache;
mod text_hit_test;
mod text_map;
mod timing;
mod viewer;
mod wrapped_text;

use gpui::{App, AppContext, Application, Bounds, WindowBounds, WindowOptions, px, size};
use shell::Shell;
use timing::Timing;

fn main() {
    let timing = Timing::start();
    let initial_paths = std::env::args_os()
        .skip(1)
        .map(std::path::PathBuf::from)
        .collect::<Vec<_>>();
    let application = Application::new();
    timing.mark("platform_created");
    application.run(move |cx: &mut App| {
        timing.mark("event_loop_ready");
        viewer::bind_keys(cx);
        shell::bind_keys(cx);
        controls::bind_keys(cx);
        query_input::bind_keys(cx);
        let bounds = Bounds::centered(None, size(px(1000.), px(760.)), cx);
        timing.mark("window_open_start");
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
                timing.scale(window.scale_factor());
                window.on_next_frame(move |_, _| timing.mark("first_frame_callback"));
                let shell = cx.new(|cx| Shell::new(window, cx, timing));
                if !initial_paths.is_empty() {
                    let shell = shell.clone();
                    window.on_next_frame(move |window, cx| {
                        shell.update(cx, |shell, cx| shell.open_paths(initial_paths, window, cx));
                    });
                }
                shell
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
