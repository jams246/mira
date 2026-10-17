use std::time::{Duration, Instant};

use super::*;
use crate::{text_cache::Source, timing::Timing};

pub(super) fn advance(
    handle: &CacheHandle,
    entry: &EntryHandle,
    input: measure::Input<'_>,
    target: f32,
    window: &mut Window,
    cx: &mut App,
) {
    let source = Source::whole(input.position.block, input.resources.cell);
    let mut state = entry.borrow_mut();
    if state.complete || state.height >= target {
        return;
    }
    if let Some(mut span) = state.pending.take() {
        if span.row.is_none() && super::span::known(&span) {
            // Native width is known; remeasure the carried flex row before committing it.
        } else if !span.text.borrow().complete {
            span.demand(&input, target, state.config.line_height, window, cx);
            state.pending = Some(span);
            return;
        } else if let Some(mut row) = span.row.take() {
            debug_assert!(row.pieces.end > state.cursor);
            row.height = span.height(state.config.line_height);
            state.cursor = row.pieces.end;
            state.height = row.top + row.height;
            state.rows.push(row);
        }
    }
    let Some(index) = state.index.clone() else {
        if state.scheduled {
            return;
        }
        state.scheduled = true;
        drop(state);
        let cache = handle.borrow();
        let loaded = Arc::clone(&cache.loaded);
        let owner = cache.owner;
        let weak = Rc::downgrade(entry);
        let task = cx.background_executor().spawn(async move {
            let timing = Timing::start();
            let index = FlowIndex::new(&loaded.document, source.runs(&loaded.document));
            timing.mark("flow_source_index");
            Arc::new(index)
        });
        cx.spawn(async move |cx| {
            let index = task.await;
            let _ = cx.update(|cx| {
                if let Some(entry) = weak.upgrade() {
                    let mut state = entry.borrow_mut();
                    state.index = Some(index);
                    state.scheduled = false;
                    cx.notify(owner);
                }
            });
        })
        .detach();
        return;
    };
    let start = Instant::now();
    let timing = Timing::start();
    let mut iterations = 0;
    while state.cursor < index.pieces().len() && state.height < target {
        let mut batch = measure::next(&index, state.cursor, &input, window, cx);
        let top = state.height;
        if let Some(last) = batch.rows.last() {
            let measured_height = last.top + last.height;
            state.cursor = last.pieces.end;
            state.rows.extend(batch.rows.into_iter().map(|mut row| {
                row.top += top;
                row
            }));
            state.height += measured_height;
        } else if batch.span.is_none() {
            state.cursor = index.pieces().len();
        }
        if let Some(mut span) = batch.span.take() {
            if let Some(row) = &mut span.row {
                row.top += top;
            }
            span.demand(&input, target, state.config.line_height, window, cx);
            state.pending = Some(span);
            break;
        }
        iterations += 1;
        if start.elapsed() >= Duration::from_millis(4) || iterations == 64 {
            break;
        }
    }
    state.complete = state.cursor == index.pieces().len() && state.pending.is_none();
    timing.mark("flow_checkpoint_batch");
    if !state.scheduled {
        state.scheduled = true;
        let weak = Rc::downgrade(entry);
        let owner = handle.borrow().owner;
        window.on_next_frame(move |_, cx| {
            if let Some(entry) = weak.upgrade() {
                entry.borrow_mut().scheduled = false;
                cx.notify(owner);
            }
        });
    }
}
