use super::*;

pub(crate) fn coarse(handle: &RevealHandle, block: usize, top: Pixels, cx: &mut App) {
    let mut state = handle.borrow_mut();
    let Some(target) = state.target.filter(|target| target.block == block) else {
        return;
    };
    let top = state
        .coarse_request
        .filter(|(old, _)| *old == target)
        .map_or(top, |(_, old_top)| top.max(old_top));
    state.coarse_request = Some((target, top));
    if state.coarse_scheduled {
        return;
    }
    state.coarse_scheduled = true;
    drop(state);
    let weak = Rc::downgrade(handle);
    // GPUI holds ListState through child prepaint; merge nested reveals after the frame.
    cx.defer(move |cx| {
        let Some(handle) = weak.upgrade() else { return };
        let mut state = handle.borrow_mut();
        state.coarse_scheduled = false;
        let Some((target, top)) = state
            .coarse_request
            .take()
            .filter(|(target, _)| state.target == Some(*target))
        else {
            return;
        };
        let Some(bounds) = state.list.bounds_for_item(target.block) else {
            return;
        };
        let offset = (top - bounds.top()).max(px(0.));
        let old = state.list.logical_scroll_top();
        // Flow rows must not undo a deeper wrapped-span checkpoint.
        // Height corrections can move the target upward after image layout changes.
        if old.item_ix != target.block || (old.offset_in_item - offset).abs() > px(1.) {
            state.list.scroll_to(ListOffset {
                item_ix: target.block,
                offset_in_item: offset,
            });
            cx.notify(state.owner);
        }
    });
}

pub(crate) fn horizontal_coarse(resources: &RenderContext, block: usize, x: Pixels, cx: &mut App) {
    let target = resources.reveal.borrow().target;
    let resources = resources.clone();
    // Native paint can resolve a visible core before this coarse scroll is needed.
    cx.defer(move |cx| {
        let state = resources.reveal.borrow();
        let run = target.is_some() && state.target == target && !state.precise;
        drop(state);
        if run {
            horizontal(&resources, block, x, cx);
        }
    });
}
