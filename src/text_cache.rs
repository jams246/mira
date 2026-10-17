mod shape;
mod work;

use std::{cell::RefCell, collections::HashMap, ops::Range, rc::Rc, sync::Arc};

use gpui::{App, EntityId, Window};
use mira::{
    document::{BlockKind, Document, Inline},
    inline_index::InlineIndex,
};

use crate::{file::LoadedDocument, render_text::TextKind, timing::Timing};

pub(crate) type CacheHandle = Rc<RefCell<TextCache>>;
pub(crate) type EntryHandle = Rc<RefCell<Entry>>;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Source {
    pub block: usize,
    pub cell: Option<usize>,
}

impl Source {
    pub(crate) fn runs(self, document: &Document) -> &[Inline] {
        match &document.blocks[self.block].kind {
            BlockKind::Paragraph(runs)
            | BlockKind::RawHtml(runs)
            | BlockKind::Heading { runs, .. } => runs,
            BlockKind::TableRow { cells, .. } => self
                .cell
                .and_then(|column| cells.get(column))
                .map_or(&[], Vec::as_slice),
            _ => &[],
        }
    }
}

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
    timing: Timing,
    target: usize,
    active: bool,
    scheduled: bool,
}

impl Entry {
    pub(crate) fn estimated_rows(&self) -> usize {
        if self.complete {
            return self.rows.max(1);
        }
        let remaining = self.len.saturating_sub(self.cursor);
        let estimate = if self.cursor > 0 {
            remaining as f64 * self.rows as f64 / self.cursor as f64
        } else {
            self.len as f64 / (self.config.width / (self.config.font_size * 0.5)).max(1.) as f64
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
            index.slice(document, source.runs(document), range)
        })
    }
}

pub(crate) struct TextCache {
    loaded: Arc<LoadedDocument>,
    owner: EntityId,
    entries: HashMap<Source, EntryHandle>,
}

impl TextCache {
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
        _window: &mut Window,
        _cx: &mut App,
    ) -> EntryHandle {
        let mut cache = cache.borrow_mut();
        if let Some(entry) = cache.entries.get(&source)
            && entry.borrow().config == config
        {
            return Rc::clone(entry);
        }
        let runs = source.runs(&cache.loaded.document);
        let len = runs
            .iter()
            .map(|run| run.text.as_str(&cache.loaded.document.source).len())
            .sum();
        let entry = Rc::new(RefCell::new(Entry {
            config,
            index: None,
            chunks: Vec::new(),
            cursor: 0,
            context_start: 0,
            rows: 0,
            len,
            complete: false,
            error: None,
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
