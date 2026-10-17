use super::*;

pub(crate) fn horizontal(resources: &RenderContext, block: usize, x: Pixels, cx: &mut App) -> bool {
    let scrolls = resources.code_scrolls.borrow();
    let Some(scroll) = scrolls.get(&block) else {
        return false;
    };
    let bounds = scroll.bounds();
    if bounds.size.width <= px(0.) {
        return false;
    }
    let old = scroll.offset();
    let shift = if x < bounds.left() + px(8.) || x > bounds.right() - px(8.) {
        bounds.left() + px(8.) - x
    } else {
        return false;
    };
    let next = point((old.x + shift).min(px(0.)), old.y);
    if next == old {
        return false;
    }
    scroll.set_offset(next);
    cx.notify(resources.owner);
    true
}
