use std::{path::PathBuf, sync::Arc};

use gpui::{Context, PathPromptOptions, Window};

use super::{Open, Viewer};
use crate::file;

impl Viewer {
    pub(crate) fn load(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        self.loaded = None;
        self.link_error = None;
        self.pending_anchor = None;
        self.content_marked = false;
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
                        viewer.list.reset(loaded.document.blocks.len());
                        viewer.loaded = Some(Arc::new(loaded));
                        if let Some(anchor) = viewer.pending_anchor.take() {
                            viewer.jump_to_anchor(&anchor);
                        }
                    }
                    Err(error) => {
                        viewer.message = format!("Cannot open {}: {error}", path.display());
                    }
                }
                cx.notify();
            });
        }));
    }

    pub(super) fn open(&mut self, _: &Open, window: &mut Window, cx: &mut Context<Self>) {
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
