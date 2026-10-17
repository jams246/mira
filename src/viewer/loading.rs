use super::*;
use mira::{headings, reading_state::Location};

impl Viewer {
    pub(crate) fn load(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.load_with_anchor(path, None, window, cx);
    }

    pub(crate) fn load_with_anchor(
        &mut self,
        path: PathBuf,
        anchor: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let source = path.clone();
        let task = cx
            .background_executor()
            .spawn(async move { crate::file::load(&source) });
        self.load_with_task(path, anchor, task, window, cx);
    }

    pub(crate) fn load_with_task(
        &mut self,
        path: PathBuf,
        anchor: Option<String>,
        task: Task<std::io::Result<LoadedDocument>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        self.loading = true;
        self.requested = Some(path.clone());
        self.pending_anchor = anchor;
        self.link_error = None;
        self.find.cancel();
        self.message = format!("Opening {}...", path.display());
        let previous = self.loaded.clone();
        let old_headings = Arc::clone(&self.headings);
        let location = self.location();
        let task = cx.background_executor().spawn(async move {
            task.await.map(|loaded| {
                let headings = headings::index(&loaded.document);
                let unchanged = previous
                    .as_ref()
                    .is_some_and(|old| old.document.source == loaded.document.source);
                let restored = restore_location(
                    location,
                    &old_headings,
                    &headings,
                    &loaded.document,
                    unchanged,
                );
                (loaded, headings, unchanged, restored)
            })
        });
        self.load_task = Some(cx.spawn_in(window, async move |viewer, cx| {
            let result = task.await;
            let _ = viewer.update_in(cx, |viewer, window, cx| {
                if viewer.generation != generation {
                    return;
                }
                viewer.loading = false;
                let timing = Timing::start();
                match result {
                    Ok((loaded, headings, unchanged, restored)) => {
                        viewer.timing.duration("file_read", loaded.read_time);
                        viewer.timing.duration("parse_model", loaded.parse_time);
                        viewer.timing.duration("code_index", loaded.code_index_time);
                        viewer.timing.mark("document_loaded");
                        viewer.headings = Arc::new(headings);
                        viewer.contents &= !viewer.headings.is_empty();
                        viewer.list.reset(loaded.document.blocks.len());
                        viewer.loaded = Some(Arc::new(loaded));
                        viewer.message.clear();
                        if !unchanged {
                            viewer.selection = Default::default();
                            viewer.history.clear();
                            viewer.table_scrolls = Default::default();
                            viewer.code_scrolls = Default::default();
                        }
                        viewer.reveal.borrow_mut().reading = restored;
                        viewer.find.highlights.borrow_mut().matches = Arc::new(Vec::new());
                        viewer.find.highlights.borrow_mut().active = None;
                        viewer.find.dirty = true;
                        viewer.finish_load(window, cx);
                    }
                    Err(error) => {
                        let message = format!(
                            "Cannot open {}: {error}",
                            viewer
                                .requested
                                .as_deref()
                                .unwrap_or(Path::new(""))
                                .display()
                        );
                        if viewer.loaded.is_some() {
                            viewer.link_error = Some(message);
                        } else {
                            viewer.message = message;
                        }
                    }
                }
                cx.notify();
                timing.mark("reload_publish_callback");
            });
        }));
        cx.notify();
    }

    pub(crate) fn reload(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(path) = self.requested.clone() {
            self.load(path, window, cx);
        }
    }

    pub(super) fn finish_load(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.release_caches(window, cx);
        self.content_paint = ContentPaint::new(self.timing);
        let location = self.reveal.borrow().reading;
        if let Some(anchor) = self.pending_anchor.take() {
            self.navigate_fragment(&anchor, true, window, cx);
        } else {
            self.jump(location, false, cx);
        }
        self.search(false, window, cx);
    }
}

fn restore_location(
    mut location: Location,
    old: &[Heading],
    new: &[Heading],
    document: &mira::document::Document,
    unchanged: bool,
) -> Location {
    if !unchanged
        && let Some((old_heading, new_heading)) = old
            .iter()
            .rev()
            .filter(|h| h.block <= location.block)
            .find_map(|old| {
                new.iter()
                    .find(|h| h.fragment == old.fragment)
                    .map(|new| (old, new))
            })
    {
        location.block = new_heading.block + location.block.saturating_sub(old_heading.block);
    }
    location.block = location.block.min(document.blocks.len().saturating_sub(1));
    if let Some(block) = document.blocks.get(location.block) {
        use mira::document::BlockKind;
        if let BlockKind::TableRow { cells, .. } = &block.kind {
            location.cell = Some(
                location
                    .cell
                    .unwrap_or(0)
                    .min(cells.len().saturating_sub(1)),
            );
            let cell = location.cell.unwrap_or(0);
            let start: usize = cells
                .iter()
                .take(cell)
                .map(|runs| {
                    runs.iter()
                        .map(|r| r.text.as_str(&document.source).len())
                        .sum::<usize>()
                        + 1
                })
                .sum();
            if let Some(runs) = cells.get(cell) {
                let index = mira::inline_index::InlineIndex::new(document, runs);
                location.byte = start
                    + index.floor_boundary(document, runs, location.byte.saturating_sub(start));
            }
        } else {
            location.cell = None;
        }
        let runs = match &block.kind {
            BlockKind::Paragraph(runs)
            | BlockKind::RawHtml(runs)
            | BlockKind::Heading { runs, .. }
            | BlockKind::Code { runs, .. } => Some(runs),
            _ => None,
        };
        if let Some(runs) = runs {
            let index = mira::inline_index::InlineIndex::new(document, runs);
            location.byte = index.floor_boundary(document, runs, location.byte);
        }
    }
    location
}

#[cfg(test)]
mod tests;
