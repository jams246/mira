use gpui::{App, Bounds, Pixels, px};

use super::FlowView;
use crate::flow_cache::EntryHandle;

pub(super) fn prepare(
    view: &FlowView,
    entry: &EntryHandle,
    bounds: Bounds<Pixels>,
    cx: &mut App,
) -> bool {
    let state = view.resources.reveal.borrow();
    let Some(target) = state.target.filter(|target| {
        target.block == view.position.block
            && target.cell == view.resources.cell
            && target.byte >= view.position.byte
    }) else {
        return false;
    };
    let entry = entry.borrow();
    let Some(index) = entry.index.as_ref() else {
        return true;
    };
    let byte = target.byte - view.position.byte;
    let ordinal = index
        .pieces()
        .partition_point(|piece| piece.range.end <= byte)
        .min(index.pieces().len().saturating_sub(1));
    let first = entry.rows.partition_point(|row| row.pieces.end <= ordinal);
    let row = entry
        .rows
        .get(first)
        .filter(|row| row.pieces.contains(&ordinal))
        .cloned()
        .or_else(|| {
            entry
                .pending_row()
                .filter(|row| row.pieces.contains(&ordinal))
        });
    if let Some(row) = row {
        drop(state);
        crate::reveal::coarse(
            &view.resources.reveal,
            view.position.block,
            bounds.top() + px(row.top),
            cx,
        );
        false
    } else {
        !entry.complete
    }
}
