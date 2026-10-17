mod shape;
mod source;
#[cfg(test)]
mod tests;
mod work;

pub(crate) use source::Source;

use std::{cell::RefCell, collections::HashMap, ops::Range, rc::Rc, sync::Arc};

use gpui::{App, EntityId, Window};
use mira::{
    document::{Document, Inline},
    inline_index::InlineIndex,
};

use crate::{file::LoadedDocument, render_text::TextKind, timing::Timing};

pub(crate) type CacheHandle = Rc<RefCell<TextCache>>;
pub(crate) type EntryHandle = Rc<RefCell<Entry>>;

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Config {
    pub width: f32,
    pub font_size: f32,
    pub kind: TextKind,
}

#[derive(Clone)]
pub(crate) struct Chunk {
    pub range: Range<usize>,
    pub row: usize,
    pub rows: usize,
    pub skip_rows: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum IntrinsicWidth {
    Unknown,
    Full,
    Exact(f32),
}

pub(crate) struct Entry {
    config: Config,
    index: Option<Arc<InlineIndex>>,
    pub chunks: Vec<Chunk>,
    cursor: usize,
    context_start: usize,
    pub rows: usize,
    len: usize,
    pub complete: bool,
    pub error: Option<String>,
    start: usize,
    intrinsic: IntrinsicWidth,
    natural_width: f32,
    timing: Timing,
    target: usize,
    active: bool,
    scheduled: bool,
}

impl Entry {
    pub(crate) fn intrinsic_width(&self) -> IntrinsicWidth {
        self.intrinsic
    }

    pub(crate) fn estimated_rows(&self) -> usize {
        if self.complete {
            return self.rows.max(1);
        }
        let remaining = self.len.saturating_sub(self.cursor);
        let consumed = self.cursor.saturating_sub(self.start);
        let estimate = if consumed > 0 {
            remaining as f64 * self.rows as f64 / consumed as f64
        } else {
            self.len.saturating_sub(self.start) as f64
                / (self.config.width / (self.config.font_size * 0.5)).max(1.) as f64
        };
        self.rows.saturating_add(estimate.ceil() as usize).max(1)
    }

    pub(crate) fn slice(
        &self,
        document: &Document,
        source: Source,
        range: Range<usize>,
    ) -> Vec<Inline> {
        self.index.as_ref().map_or_else(Vec::new, |index| {
            index.slice(
                document,
                source.runs(document),
                range.start.max(self.start)..range.end.min(self.len),
            )
        })
    }
}

pub(crate) struct TextCache {
    loaded: Arc<LoadedDocument>,
    owner: EntityId,
    entries: HashMap<Source, EntryHandle>,
}

impl TextCache {
    pub(crate) fn height_known(&self, block: usize, columns: &Range<usize>) -> bool {
        self.entries
            .iter()
            .filter(|(source, _)| {
                source.block == block && source.cell.is_some_and(|cell| columns.contains(&cell))
            })
            .all(|(_, entry)| entry.borrow().complete)
    }

    pub(crate) fn new(loaded: Arc<LoadedDocument>, owner: EntityId) -> CacheHandle {
        Rc::new(RefCell::new(Self {
            loaded,
            owner,
            entries: HashMap::new(),
        }))
    }

    pub(crate) fn request(
        cache: &CacheHandle,
        source: Source,
        config: Config,
        seed: Option<Arc<InlineIndex>>,
    ) -> EntryHandle {
        let mut cache = cache.borrow_mut();
        if let Some(entry) = cache.entries.get(&source) {
            let mut state = entry.borrow_mut();
            if state.config == config {
                if state.index.is_none() && seed.is_some() {
                    state.index = seed;
                }
                drop(state);
                return Rc::clone(entry);
            }
        }
        let runs = source.runs(&cache.loaded.document);
        let len = seed.as_ref().map_or_else(
            || {
                runs.iter()
                    .map(|run| run.text.as_str(&cache.loaded.document.source).len())
                    .sum()
            },
            |index| index.len(),
        );
        let (range, error) = match source.bounds(len) {
            Ok(range) => (range, None),
            Err(error) => (0..0, Some(error)),
        };
        let complete = range.is_empty();
        let entry = Rc::new(RefCell::new(Entry {
            config,
            index: seed,
            chunks: Vec::new(),
            cursor: range.start,
            context_start: range.start,
            rows: 0,
            len: range.end,
            complete,
            intrinsic: if error.is_some() {
                IntrinsicWidth::Full
            } else if complete {
                IntrinsicWidth::Exact(0.)
            } else {
                IntrinsicWidth::Unknown
            },
            start: range.start,
            error,
            natural_width: 0.,
            timing: Timing::start(),
            target: 0,
            active: false,
            scheduled: false,
        }));
        cache.entries.insert(source, Rc::clone(&entry));
        entry
    }

    pub(crate) fn demand(
        cache: &CacheHandle,
        entry: &EntryHandle,
        source: Source,
        target: usize,
        window: &mut Window,
        cx: &mut App,
    ) {
        let mut state = entry.borrow_mut();
        state.target = target;
        state.active = true;
        if state.scheduled || state.complete || (state.index.is_some() && state.rows >= target) {
            return;
        }
        state.scheduled = true;
        drop(state);
        let cache = cache.borrow();
        work::schedule(
            entry,
            Arc::clone(&cache.loaded),
            source,
            cache.owner,
            window,
            cx,
        );
    }
}
