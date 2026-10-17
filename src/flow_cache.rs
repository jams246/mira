pub(crate) mod measure;
#[cfg(test)]
pub(crate) mod native_tests;
pub(crate) mod span;
#[cfg(test)]
mod tests;
mod work;

use std::{cell::RefCell, collections::HashMap, ops::Range, rc::Rc, sync::Arc};

use gpui::{App, EntityId, Window};
use mira::flow_index::FlowIndex;

use crate::{file::LoadedDocument, render_text::TextFormat, text_cache::Source};

pub(crate) type CacheHandle = Rc<RefCell<FlowCache>>;
pub(crate) type EntryHandle = Rc<RefCell<Entry>>;

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Config {
    pub width: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub format: TextFormat,
    pub images: u64,
}

#[derive(Clone)]
pub(crate) struct Row {
    pub pieces: Range<usize>,
    pub top: f32,
    pub height: f32,
}

pub(crate) struct Entry {
    config: Config,
    pub index: Option<Arc<FlowIndex>>,
    pub rows: Vec<Row>,
    cursor: usize,
    height: f32,
    len: usize,
    images: usize,
    pub complete: bool,
    scheduled: bool,
    pub pending: Option<span::Span>,
}

impl Entry {
    pub(crate) fn estimated_height(&self) -> f32 {
        if self.complete {
            return self.height.max(self.config.line_height);
        }
        let consumed = self.index.as_ref().map_or(0, |index| {
            index
                .pieces()
                .get(self.cursor)
                .map_or(self.len, |piece| piece.range.start)
        });
        let remaining = self.len.saturating_sub(consumed);
        let byte_estimate = if consumed > 0 {
            self.height * remaining as f32 / consumed as f32
        } else {
            (self.len as f32 * 0.5 + self.images as f32 * 10.) * self.config.font_size
                / self.config.width.max(1.)
                * self.config.line_height
        };
        let piece_estimate = self
            .index
            .as_ref()
            .filter(|_| self.cursor > 0)
            .map_or(0., |index| {
                self.height * index.pieces().len().saturating_sub(self.cursor) as f32
                    / self.cursor as f32
            });
        let extra = byte_estimate.max(piece_estimate);
        let pending = self.pending_row().map_or(0., |row| row.top + row.height);
        (self.height + extra)
            .max(pending)
            .max(self.config.line_height)
    }

    pub(crate) fn pending_row(&self) -> Option<Row> {
        let span = self.pending.as_ref()?;
        let mut row = span.row.clone()?;
        row.height = span.height(self.config.line_height);
        Some(row)
    }
}

pub(crate) struct FlowCache {
    loaded: Arc<LoadedDocument>,
    owner: EntityId,
    entries: HashMap<Source, EntryHandle>,
}

impl FlowCache {
    pub(crate) fn new(loaded: Arc<LoadedDocument>, owner: EntityId) -> CacheHandle {
        Rc::new(RefCell::new(Self {
            loaded,
            owner,
            entries: HashMap::new(),
        }))
    }

    pub(crate) fn request(handle: &CacheHandle, source: Source, config: Config) -> EntryHandle {
        let mut cache = handle.borrow_mut();
        let previous = cache.entries.get(&source);
        if let Some(entry) = previous
            && entry.borrow().config == config
        {
            return Rc::clone(entry);
        }
        let index = previous.and_then(|entry| entry.borrow().index.clone());
        let runs = source.runs(&cache.loaded.document);
        let len = runs
            .iter()
            .map(|run| run.text.as_str(&cache.loaded.document.source).len())
            .sum();
        let entry = Rc::new(RefCell::new(Entry {
            config,
            index,
            rows: Vec::new(),
            cursor: 0,
            height: 0.,
            len,
            images: runs.iter().filter(|run| run.image.is_some()).count(),
            complete: false,
            scheduled: false,
            pending: None,
        }));
        cache.entries.insert(source, Rc::clone(&entry));
        entry
    }

    pub(crate) fn height_known(&self, block: usize, columns: &Range<usize>) -> bool {
        self.entries.iter().all(|(source, entry)| {
            source.block != block
                || source.cell.is_none_or(|cell| !columns.contains(&cell))
                || entry.borrow().complete
        })
    }

    pub(crate) fn advance(
        handle: &CacheHandle,
        entry: &EntryHandle,
        input: measure::Input<'_>,
        target: f32,
        window: &mut Window,
        cx: &mut App,
    ) {
        work::advance(handle, entry, input, target, window, cx);
    }
}

pub(crate) use measure::Input;
