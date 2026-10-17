use std::time::{Duration, Instant};

use super::*;
use shape::{Progress, Step};

struct Batch {
    progress: Progress,
    chunks: Vec<Chunk>,
    error: Option<String>,
    expensive: bool,
}

pub(super) fn schedule(cache: &CacheHandle, window: &mut Window, _: &mut App) {
    let weak_cache = Rc::downgrade(cache);
    window.on_next_frame(move |window, cx| {
        let callback = Timing::start();
        let Some(handle) = weak_cache.upgrade() else {
            return;
        };
        let mut cache = handle.borrow_mut();
        let demand = pending(&cache.order, cache.next, &cache.entries);
        let Some(&(index, source)) = demand.first() else {
            cache.scheduled = false;
            return;
        };
        let entry = Rc::clone(&cache.entries[&source]);
        let weak_entry = Rc::downgrade(&entry);
        let state = entry.borrow();
        let expected = Progress::from(&*state);
        let input = expected.clone();
        let target = state.target;
        drop(state);
        cache.next = (index + 1) % cache.order.len();
        for entry in cache.entries.values() {
            entry.borrow_mut().active = false;
        }
        let loaded = Arc::clone(&cache.loaded);
        drop(cache);
        let system = window.text_system().background_shaper();
        let task = cx.background_executor().spawn(async move {
            let timing = Timing::start();
            let batch = advance(input, target, |state| {
                shape::step(&loaded, source, state, &system)
            });
            let layouts = Arc::new(system.cached_layouts());
            timing.mark("code_checkpoint_batch");
            (batch, layouts)
        });
        cx.spawn(async move |cx| {
            let (batch, layouts) = task.await;
            let _ = cx.update(|cx| {
                let timing = Timing::start();
                let Some(cache) = weak_cache.upgrade() else {
                    return;
                };
                let mut cache = cache.borrow_mut();
                cache.scheduled = false;
                if let Some(entry) = weak_entry.upgrade()
                    && current(&cache.entries, source, &entry, &expected)
                {
                    let mut state = entry.borrow_mut();
                    publish(&mut state, batch, layouts);
                    if state.complete {
                        state.timing.mark("code_layout_ready");
                    }
                }
                let owner = cache.owner;
                drop(cache);
                cx.notify(owner);
                timing.mark("code_checkpoint_publish");
            });
        })
        .detach();
        callback.mark("code_checkpoint_callback");
    });
}

fn current(
    entries: &HashMap<Source, EntryHandle>,
    source: Source,
    entry: &EntryHandle,
    expected: &Progress,
) -> bool {
    entries.get(&source).is_some_and(|current| {
        Rc::ptr_eq(current, entry) && Progress::from(&*current.borrow()) == *expected
    })
}

fn pending(
    order: &[Source],
    next: usize,
    entries: &HashMap<Source, EntryHandle>,
) -> Vec<(usize, Source)> {
    (0..order.len())
        .filter_map(|offset| {
            let index = (next + offset) % order.len();
            let source = order[index];
            let state = entries.get(&source)?.borrow();
            (state.active && !state.complete && state.width < state.target)
                .then_some((index, source))
        })
        .collect()
}

fn advance(
    mut progress: Progress,
    target: f32,
    mut step: impl FnMut(&mut Progress) -> Result<Step, String>,
) -> Batch {
    let start = Instant::now();
    let mut batch = Batch {
        progress: progress.clone(),
        chunks: Vec::new(),
        error: None,
        expensive: false,
    };
    for _ in 0..64 {
        if progress.complete || progress.width >= target {
            break;
        }
        let timing = Timing::start();
        match step(&mut progress) {
            Ok(step) => {
                batch.expensive |= step.expensive;
                batch.chunks.push(step.chunk);
            }
            Err(error) => {
                batch.error = Some(error);
                progress.complete = true;
            }
        }
        timing.mark("code_checkpoint_step");
        // Background batches amortize frame handoffs without enlarging native shaping chunks.
        if batch.error.is_some() || start.elapsed() >= Duration::from_millis(16) {
            break;
        }
    }
    batch.progress = progress;
    batch
}

fn publish(state: &mut Entry, mut batch: Batch, layouts: Arc<gpui::BackgroundTextLayouts>) {
    if batch.expensive {
        for chunk in &mut batch.chunks {
            chunk.layouts = Some(Arc::clone(&layouts));
        }
    }
    state.latest_layouts = Some(layouts);
    state.chunks.extend(batch.chunks);
    state.width = batch.progress.width;
    state.complete = batch.progress.complete;
    state.cursor = batch.progress.cursor;
    state.context_start = batch.progress.context_start;
    state.context_column = batch.progress.context_column;
    state.error = batch.error;
}

#[cfg(test)]
mod tests;
