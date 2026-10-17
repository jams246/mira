use gpui::{App, Bounds, Pixels, Window, px};

use super::CodeView;
use crate::code_cache::{CodeCache, MIN_UNICODE_BYTES, Source};

pub(super) fn prepare(
    view: &CodeView,
    bounds: Bounds<Pixels>,
    metrics: (Pixels, Pixels, Pixels),
    window: &mut Window,
    cx: &mut App,
) -> Option<usize> {
    let target = view
        .resources
        .reveal
        .borrow()
        .target
        .filter(|target| target.block == view.block && target.cell.is_none())?;
    let layout = view.resources.loaded.code.get(&view.block)?;
    let row = layout.row_for_byte(target.byte);
    let line = layout.line(row)?;
    let (advance, line_height, font_size) = metrics;
    crate::reveal::coarse(
        &view.resources.reveal,
        view.block,
        bounds.top() + line_height * row as f32,
        cx,
    );
    if line.ascii {
        let x = bounds.left() + advance * layout.column_for_byte(&line, target.byte) as f32;
        crate::reveal::horizontal(&view.resources, view.block, x, cx);
    } else if line.bytes.len() >= MIN_UNICODE_BYTES {
        let source = Source {
            block: view.block,
            row,
        };
        let entry = CodeCache::request(&view.resources.codes, source, &line, font_size.into());
        let state = entry.borrow();
        let first = state
            .chunks
            .partition_point(|chunk| chunk.core.end < target.byte);
        let chunk = state
            .chunks
            .get(first)
            .filter(|chunk| chunk.core.start <= target.byte && target.byte <= chunk.core.end);
        let x = chunk.map(|chunk| chunk.x);
        let pending = x.is_none() && !state.complete;
        drop(state);
        if let Some(x) = x {
            crate::reveal::horizontal(&view.resources, view.block, bounds.left() + px(x), cx);
        } else if pending {
            CodeCache::demand(&view.resources.codes, &entry, f32::INFINITY, window, cx);
            return Some(row);
        }
    }
    None
}
