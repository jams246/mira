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
    let source = Source {
        block: input.position.block,
        cell: input.resources.cell,
    };
    let mut state = entry.borrow_mut();
    if state.complete || state.height >= target {
        return;
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
        let rows = measure::next(&index, state.cursor, &input, window, cx);
        if let Some(last) = rows.last() {
            let measured_height = last.top + last.height;
            state.cursor = last.pieces.end;
            let top = state.height;
            state.rows.extend(rows.into_iter().map(|mut row| {
                row.top += top;
                row
            }));
            state.height += measured_height;
        } else {
            state.cursor = index.pieces().len();
        }
        iterations += 1;
        if start.elapsed() >= Duration::from_millis(4) || iterations == 64 {
            break;
        }
    }
    state.complete = state.cursor == index.pieces().len();
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
