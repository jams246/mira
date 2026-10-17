use super::*;
use gpui::{Window, size};

pub(crate) fn text(
    resources: &RenderContext,
    position: Position,
    len: usize,
    mapping: &TextMap,
    layout: &TextLayout,
    window: &mut Window,
) {
    let mut state = resources.reveal.borrow_mut();
    if let Some(target) = state.target
        && inside(target, position, resources.cell, len)
        && layout.bounds().size.height > px(0.)
        && let Some(point) =
            layout.position_for_index(mapping.to_display(target.byte - position.byte))
    {
        align(&mut state, target, point.y, window);
    }
}

pub(crate) fn code(
    resources: &RenderContext,
    position: Position,
    core: &Range<usize>,
    window_end: usize,
    top: Pixels,
    window: &mut Window,
) {
    let mut state = resources.reveal.borrow_mut();
    if let Some(target) = state.target
        && code_inside(target, position, core, window_end)
    {
        align(&mut state, target, top, window);
    }
}

fn align(state: &mut Reveal, target: Location, top: Pixels, window: &mut Window) {
    // Precise geometry supersedes deferred checkpoints from the discarded prepaint.
    state.coarse_request = None;
    // A viewport-sized request aligns the target before GPUI paints the list.
    window.request_autoscroll(Bounds::new(
        point(state.viewport.left(), top - px(target.offset)),
        size(state.viewport.size.width, state.viewport.size.height),
    ));
}
