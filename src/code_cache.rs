mod shape;
mod work;

pub(crate) use shape::{native, prepare};

use std::{cell::RefCell, collections::HashMap, ops::Range, rc::Rc, sync::Arc};

use gpui::{BackgroundTextLayouts, EntityId, Window};
use mira::code_layout::{CodeLayout, CodeLine};

use crate::{file::LoadedDocument, timing::Timing};

pub(crate) type CacheHandle = Rc<RefCell<CodeCache>>;
pub(crate) type EntryHandle = Rc<RefCell<Entry>>;
pub(crate) const MIN_UNICODE_BYTES: usize = 2048;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Source {
    pub block: usize,
    pub row: usize,
}

#[derive(Clone)]
pub(crate) struct Chunk {
    pub window: Range<usize>,
    pub core: Range<usize>,
    pub column: usize,
    pub x: f32,
    pub local_x: f32,
    pub width: f32,
    pub layouts: Option<Arc<BackgroundTextLayouts>>,
}

pub(crate) struct Entry {
    pub font_size: f32,
    pub line: Range<usize>,
    pub chunks: Vec<Chunk>,
    pub width: f32,
    pub complete: bool,
    pub error: Option<String>,
    pub latest_layouts: Option<Arc<BackgroundTextLayouts>>,
    cursor: usize,
    context_start: usize,
    context_column: usize,
    target: f32,
    active: bool,
    timing: Timing,
}

struct Widths {
    other_columns: usize,
    lines: Vec<(usize, usize)>,
}

pub(crate) struct CodeCache {
    loaded: Arc<LoadedDocument>,
    owner: EntityId,
    entries: HashMap<Source, EntryHandle>,
    widths: HashMap<usize, Widths>,
    order: Vec<Source>,
    next: usize,
    scheduled: bool,
}

impl CodeCache {
    pub(crate) fn new(loaded: Arc<LoadedDocument>, owner: EntityId) -> CacheHandle {
        Rc::new(RefCell::new(Self {
            loaded,
            owner,
            entries: HashMap::new(),
            widths: HashMap::new(),
            order: Vec::new(),
            next: 0,
            scheduled: false,
        }))
    }

    pub(crate) fn request(
        cache: &CacheHandle,
        source: Source,
        line: &CodeLine,
        font_size: f32,
    ) -> EntryHandle {
        let mut cache = cache.borrow_mut();
        if let Some(entry) = cache.entries.get(&source)
            && entry.borrow().font_size == font_size
        {
            return Rc::clone(entry);
        }
        if !cache.widths.contains_key(&source.block)
            && let Some(layout) = cache.loaded.code.get(&source.block)
        {
            let widths = Widths::new(layout);
            cache.widths.insert(source.block, widths);
        }
        let entry = Rc::new(RefCell::new(Entry {
            font_size,
            line: line.bytes.clone(),
            chunks: Vec::new(),
            width: 0.,
            complete: line.bytes.is_empty(),
            error: None,
            latest_layouts: None,
            cursor: line.bytes.start,
            context_start: line.bytes.start,
            context_column: 0,
            target: 0.,
            active: false,
            timing: Timing::start(),
        }));
        if !cache.entries.contains_key(&source) {
            cache.order.push(source);
        }
        cache.entries.insert(source, Rc::clone(&entry));
        entry
    }

    pub(crate) fn width(cache: &CacheHandle, block: usize, advance: f32, fallback: f32) -> f32 {
        let cache = cache.borrow();
        let Some(widths) = cache.widths.get(&block) else {
            return fallback;
        };
        widths.lines.iter().fold(
            widths.other_columns as f32 * advance,
            |width, (row, columns)| {
                let source = Source { block, row: *row };
                let line_width = cache.entries.get(&source).map_or_else(
                    || *columns as f32 * advance * 4.,
                    |entry| {
                        let entry = entry.borrow();
                        if entry.complete && entry.error.is_none() {
                            entry.width
                        } else {
                            entry.width.max(*columns as f32 * advance * 4.)
                        }
                    },
                );
                width.max(line_width)
            },
        )
    }

    pub(crate) fn demand(
        cache: &CacheHandle,
        entry: &EntryHandle,
        target: f32,
        window: &mut Window,
    ) {
        let mut state = entry.borrow_mut();
        state.target = target.max(0.);
        state.active = true;
        if state.complete || state.width >= state.target {
            return;
        }
        drop(state);
        if cache.borrow().scheduled {
            return;
        }
        cache.borrow_mut().scheduled = true;
        work::schedule(cache, window);
    }
}

impl Widths {
    fn new(layout: &CodeLayout) -> Self {
        let mut result = Self {
            other_columns: 0,
            lines: Vec::new(),
        };
        for row in 0..layout.line_count() {
            let Some(line) = layout.line(row) else {
                continue;
            };
            if !line.ascii && line.bytes.len() >= MIN_UNICODE_BYTES {
                result.lines.push((row, line.columns));
            } else {
                result.other_columns = result
                    .other_columns
                    .max(line.columns * if line.ascii { 1 } else { 4 });
            }
        }
        result
    }
}
