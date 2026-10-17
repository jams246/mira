use std::time::{Duration, Instant};

use gpui::{App, Window};

use super::*;

pub(super) fn schedule(cache: &CacheHandle, window: &mut Window, _: &mut App) {
    let weak = Rc::downgrade(cache);
    window.on_next_frame(move |window, cx| {
        let Some(cache) = weak.upgrade() else { return };
        let mut cache = cache.borrow_mut();
        cache.scheduled = false;
        let pending = pending(&cache.order, cache.next, &cache.entries);
        let advanced = advance(&cache.entries, &pending, |source, state| {
            shape::step(&cache.loaded, source, state, window.text_system())
        });
        if let Some(index) = advanced {
            cache.next = (index + 1) % cache.order.len();
        }
        for entry in cache.entries.values() {
            entry.borrow_mut().active = false;
        }
        if advanced.is_some() {
            cx.notify(cache.owner);
        }
    });
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
    entries: &HashMap<Source, EntryHandle>,
    pending: &[(usize, Source)],
    mut step: impl FnMut(Source, &mut Entry) -> Result<(), String>,
) -> Option<usize> {
    let start = Instant::now();
    let mut advanced = None;
    for &(index, source) in pending.iter().cycle().take(64) {
        let mut state = entries[&source].borrow_mut();
        if state.complete || state.width >= state.target {
            continue;
        }
        let timing = Timing::start();
        if let Err(error) = step(source, &mut state) {
            state.error = Some(error);
            state.complete = true;
        }
        timing.mark("code_checkpoint_step");
        if state.complete {
            state.timing.mark("code_layout_ready");
        }
        advanced = Some(index);
        if start.elapsed() >= Duration::from_millis(4) {
            break;
        }
    }
    advanced
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_demand_after_sixty_four_historical_rows_actually_advances() {
        let order = (0..129)
            .map(|row| Source { block: 0, row })
            .collect::<Vec<_>>();
        let entries = order
            .iter()
            .map(|source| {
                (
                    *source,
                    Rc::new(RefCell::new(Entry {
                        font_size: 14.,
                        line: 0..1000,
                        chunks: Vec::new(),
                        width: 0.,
                        complete: false,
                        error: None,
                        cursor: 0,
                        context_start: 0,
                        context_column: 0,
                        target: 5.,
                        active: source.row == 100,
                        timing: Timing::start(),
                    })),
                )
            })
            .collect::<HashMap<_, _>>();
        let demand = pending(&order, 0, &entries);
        let advanced = advance(&entries, &demand, |source, state| {
            assert_eq!(source.row, 100);
            state.width += 1.;
            state.cursor += 1;
            Ok(())
        });
        assert_eq!(advanced, Some(100));
        let visible = entries[&order[100]].borrow();
        assert_eq!(visible.width, 5.);
        assert_eq!(visible.cursor, 5);
        assert_eq!(entries[&order[0]].borrow().cursor, 0);
    }
}
