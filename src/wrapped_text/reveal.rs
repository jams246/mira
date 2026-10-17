use gpui::{App, Bounds, Pixels};

use super::WrappedText;
use crate::text_cache::EntryHandle;

pub(super) fn prepare(
    view: &WrappedText,
    entry: &EntryHandle,
    bounds: Bounds<Pixels>,
    line_height: Pixels,
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
    let byte = target.byte - view.position.byte;
    if view
        .source
        .span
        .is_some_and(|(start, end)| byte < start || byte > end)
    {
        return false;
    }
    let entry = entry.borrow();
    let count = entry
        .chunks
        .partition_point(|chunk| chunk.range.start <= byte);
    let chunk = count
        .checked_sub(1)
        .and_then(|index| entry.chunks.get(index))
        .filter(|chunk| byte <= chunk.range.end);
    if let Some(chunk) = chunk {
        let top = bounds.top() + line_height * chunk.row as f32;
        drop(state);
        crate::reveal::coarse(&view.resources.reveal, view.position.block, top, cx);
        false
    } else {
        !entry.complete
    }
}
