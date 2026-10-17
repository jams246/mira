use super::*;

fn entry(active: bool) -> EntryHandle {
    Rc::new(RefCell::new(Entry {
        font_size: 14.,
        line: 0..1000,
        chunks: Vec::new(),
        width: 0.,
        complete: false,
        error: None,
        latest_layouts: None,
        cursor: 0,
        context_start: 0,
        context_column: 0,
        target: 5.,
        active,
        timing: Timing::start(),
    }))
}

fn step(progress: &mut Progress) -> Result<Step, String> {
    let start = progress.cursor;
    progress.cursor += 1;
    progress.width += 1.;
    Ok(Step {
        chunk: Chunk {
            window: start..progress.cursor,
            core: start..progress.cursor,
            column: 0,
            x: progress.width - 1.,
            local_x: 0.,
            width: 1.,
            layouts: None,
        },
        expensive: false,
    })
}

#[test]
fn visible_demand_after_sixty_four_historical_rows_actually_advances() {
    let order = (0..129)
        .map(|row| Source { block: 0, row })
        .collect::<Vec<_>>();
    let entries = order
        .iter()
        .map(|source| (*source, entry(source.row == 100)))
        .collect::<HashMap<_, _>>();
    let demand = pending(&order, 0, &entries);
    assert_eq!(demand, [(100, order[100])]);
    let result = advance(Progress::from(&*entries[&order[100]].borrow()), 5., step);
    assert_eq!(result.progress.width, 5.);
    assert_eq!(result.progress.cursor, 5);
    assert_eq!(result.chunks.len(), 5);
    assert_eq!(entries[&order[0]].borrow().cursor, 0);
    assert_eq!(entries[&order[100]].borrow().cursor, 0);
}

#[test]
fn round_robin_skips_inactive_rows_without_losing_visible_demand() {
    let order = (0..4)
        .map(|row| Source { block: 0, row })
        .collect::<Vec<_>>();
    let entries = order
        .iter()
        .map(|source| (*source, entry(source.row != 1)))
        .collect::<HashMap<_, _>>();
    assert_eq!(
        pending(&order, 2, &entries),
        [(2, order[2]), (3, order[3]), (0, order[0])]
    );
    entries[&order[2]].borrow_mut().complete = true;
    assert_eq!(pending(&order, 3, &entries)[0], (3, order[3]));
    entries[&order[0]].borrow_mut().active = false;
    assert_eq!(pending(&order, 0, &entries), [(3, order[3])]);
}

#[test]
fn changed_font_or_replaced_entry_rejects_stale_publication() {
    let source = Source { block: 2, row: 3 };
    let old = entry(true);
    let expected = Progress::from(&*old.borrow());
    let mut entries = HashMap::from([(source, Rc::clone(&old))]);
    assert!(current(&entries, source, &old, &expected));
    old.borrow_mut().font_size = 16.;
    assert!(!current(&entries, source, &old, &expected));
    old.borrow_mut().font_size = 14.;
    entries.insert(source, entry(true));
    assert!(!current(&entries, source, &old, &expected));
}

#[test]
fn changing_demand_is_allowed_but_changed_progress_is_not() {
    let source = Source { block: 0, row: 0 };
    let value = entry(true);
    let expected = Progress::from(&*value.borrow());
    let entries = HashMap::from([(source, Rc::clone(&value))]);
    value.borrow_mut().target = 1.;
    value.borrow_mut().active = false;
    assert!(current(&entries, source, &value, &expected));
    value.borrow_mut().cursor = 1;
    assert!(!current(&entries, source, &value, &expected));
}

#[test]
fn worker_returns_only_new_chunks_and_stops_at_its_step_limit() {
    let value = entry(true);
    let first = advance(Progress::from(&*value.borrow()), 1000., step);
    assert!(first.chunks.len() <= 64);
    assert!(!first.chunks.is_empty());
    let cursor = first.progress.cursor;
    let next = advance(first.progress, cursor as f32 + 1., step);
    assert_eq!(next.chunks.len(), 1);
    assert_eq!(next.chunks[0].core, cursor..cursor + 1);
    assert_eq!(value.borrow().chunks.len(), 0);
}

#[test]
fn native_error_preserves_message_and_closes_the_pending_row() {
    let value = entry(true);
    let result = advance(Progress::from(&*value.borrow()), 5., |_| {
        Err("native failure".into())
    });
    assert_eq!(result.error.as_deref(), Some("native failure"));
    assert!(result.progress.complete);
    assert!(result.chunks.is_empty());
}
