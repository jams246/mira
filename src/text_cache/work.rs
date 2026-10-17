use super::*;

pub(super) fn schedule(
    entry: &EntryHandle,
    loaded: Arc<LoadedDocument>,
    source: Source,
    owner: EntityId,
    window: &mut Window,
    cx: &mut App,
) {
    // Start during prepaint to avoid an extra frame wait for every background batch.
    let weak = Rc::downgrade(entry);
    let callback = Timing::start();
    let mut state = entry.borrow_mut();
    state.scheduled = false;
    if !state.active || state.complete || (state.index.is_some() && state.rows >= state.target) {
        return;
    }
    state.active = false;
    let Some(index) = state.index.clone() else {
        state.scheduled = true;
        drop(state);
        let task = cx.background_executor().spawn(async move {
            let timing = Timing::start();
            let index = InlineIndex::new(&loaded.document, source.runs(&loaded.document));
            timing.mark("text_source_index");
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
        callback.mark("text_checkpoint_callback");
        return;
    };
    let input = shape::Input {
        index,
        config: state.config,
        cursor: state.cursor,
        context_start: state.context_start,
        row: state.rows,
        target: state.target,
        intrinsic: state.intrinsic,
        natural_width: state.natural_width,
    };
    state.scheduled = true;
    drop(state);
    let system = window.text_system().background_shaper();
    let task = cx.background_executor().spawn(async move {
        let timing = Timing::start();
        let result = shape::batch(&loaded.document, source, input, &system);
        let layouts = Arc::new(system.cached_layouts());
        timing.mark("text_checkpoint_batch");
        (result, layouts)
    });
    cx.spawn(async move |cx| {
        let (result, layouts) = task.await;
        let _ = cx.update(|cx| {
            let timing = Timing::start();
            let Some(entry) = weak.upgrade() else { return };
            let mut state = entry.borrow_mut();
            state.scheduled = false;
            match result {
                Ok(mut result) => {
                    if result.retain_layouts {
                        for chunk in &mut result.chunks {
                            chunk.layouts = Some(Arc::clone(&layouts));
                        }
                    }
                    state.latest_layouts = Some(layouts);
                    state.chunks.extend(result.chunks);
                    state.cursor = result.cursor;
                    state.context_start = result.context_start;
                    state.rows = result.rows;
                    state.len = result.len;
                    state.complete = result.cursor == result.len;
                    state.intrinsic = result.intrinsic;
                    state.natural_width = result.natural_width;
                }
                Err(error) => {
                    state.error = Some(error);
                    state.complete = true;
                    state.intrinsic = IntrinsicWidth::Full;
                }
            }
            if state.complete {
                state.timing.mark("text_layout_ready");
            }
            cx.notify(owner);
            timing.mark("text_checkpoint_publish");
        });
    })
    .detach();
    callback.mark("text_checkpoint_callback");
}
