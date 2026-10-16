use std::{path::PathBuf, sync::Arc};

use crate::{
    file::{self, LoadedDocument},
    timing::Timing,
};
use gpui::{
    App, Context, FocusHandle, KeyBinding, PathPromptOptions, Task, Window, actions, div,
    prelude::*, rgb,
};

actions!(mira, [Open]);

pub(crate) fn bind_keys(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("ctrl-o", Open, Some("Mira"))]);
}

pub(crate) struct Viewer {
    loaded: Option<Arc<LoadedDocument>>,
    message: String,
    generation: u64,
    load_task: Option<Task<()>>,
    focus: FocusHandle,
    timing: Timing,
}

impl Viewer {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>, timing: Timing) -> Self {
        let focus = cx.focus_handle();
        window.focus(&focus);
        Self {
            loaded: None,
            message: "Open a Markdown file to start reading.".into(),
            generation: 0,
            load_task: None,
            focus,
            timing,
        }
    }

    pub(crate) fn load(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        self.loaded = None;
        self.message = format!("Opening {}...", path.display());
        window.set_window_title("MIRA");
        cx.notify();
        let task = cx.background_executor().spawn(async move {
            let result = file::load(&path);
            (path, result)
        });
        self.load_task = Some(cx.spawn_in(window, async move |viewer, cx| {
            let (path, result) = task.await;
            let _ = viewer.update_in(cx, |viewer, window, cx| {
                if generation != viewer.generation {
                    return;
                }
                match result {
                    Ok(loaded) => {
                        viewer.timing.duration("file_read", loaded.read_time);
                        viewer.timing.duration("parse_model", loaded.parse_time);
                        viewer.timing.mark("document_loaded");
                        viewer.message = format!(
                            "Loaded {} blocks from {}.",
                            loaded.document.blocks.len(),
                            path.display()
                        );
                        window.set_window_title(&format!(
                            "{} | MIRA",
                            loaded
                                .path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                        ));
                        viewer.loaded = Some(Arc::new(loaded));
                    }
                    Err(error) => {
                        viewer.message = format!("Cannot open {}: {error}", path.display());
                    }
                }
                cx.notify();
            });
        }));
    }

    fn open(&mut self, _: &Open, window: &mut Window, cx: &mut Context<Self>) {
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Open Markdown".into()),
        });
        cx.spawn_in(window, async move |viewer, cx| match paths.await {
            Ok(Ok(Some(paths))) => {
                if let Some(path) = paths.into_iter().next() {
                    let _ = viewer.update_in(cx, |viewer, window, cx| {
                        viewer.load(path, window, cx);
                    });
                }
            }
            Ok(Ok(None)) => {}
            result => {
                let _ = viewer.update(cx, |viewer, cx| {
                    viewer.message = format!("Cannot open file dialog: {result:?}");
                    cx.notify();
                });
            }
        })
        .detach();
    }
}

impl Render for Viewer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .key_context("Mira")
            .track_focus(&self.focus)
            .on_action(cx.listener(Self::open))
            .size_full()
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .p_8()
            .gap_3()
            .bg(rgb(0xfafafa))
            .text_color(rgb(0x242424))
            .font_family("Segoe UI")
            .child(div().text_3xl().child("MIRA"))
            .child(self.message.clone())
            .child(
                div()
                    .id("open")
                    .cursor_pointer()
                    .px_4()
                    .py_2()
                    .rounded_md()
                    .border_1()
                    .border_color(rgb(0xcccccc))
                    .hover(|style| style.bg(rgb(0xeeeeee)))
                    .on_click(cx.listener(|viewer, _, window, cx| viewer.open(&Open, window, cx)))
                    .child("Open Markdown (Ctrl+O)"),
            )
    }
}
