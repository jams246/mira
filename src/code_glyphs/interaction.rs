use gpui::{CursorStyle, DispatchPhase, MouseButton, MouseDownEvent, MouseMoveEvent};

use super::*;
use crate::selection::Position;

pub(super) fn register(glyphs: &CodeGlyphs, resources: &RenderContext, window: &mut Window) {
    window.set_cursor_style(CursorStyle::IBeam, &glyphs.hitbox);
    let (bounds, hitbox, block) = (glyphs.bounds, glyphs.hitbox.id, glyphs.source.block);
    let origin = Position {
        block,
        byte: glyphs.start,
    };
    let pieces = Rc::clone(&glyphs.pieces);
    let selection = Rc::clone(&resources.selection);
    let owner = resources.owner;
    window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
        if phase == DispatchPhase::Bubble
            && event.button == MouseButton::Left
            && hitbox.is_hovered(window)
            && let Some(byte) = hit(&pieces, bounds.origin, event.position)
        {
            selection
                .borrow_mut()
                .start_text(Position { block, byte }, origin, false);
            cx.notify(owner);
        }
    });
    let pieces = Rc::clone(&glyphs.pieces);
    let selection = Rc::clone(&resources.selection);
    window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
        if event.pressed_button != Some(MouseButton::Left) {
            return;
        }
        let captured = phase == DispatchPhase::Capture
            && selection.borrow().dragging_from(origin)
            && event.position.y >= bounds.top()
            && event.position.y < bounds.bottom();
        if (captured || (phase == DispatchPhase::Bubble && hitbox.is_hovered(window)))
            && let Some(byte) = hit(&pieces, bounds.origin, event.position)
            && selection.borrow_mut().extend(Position { block, byte })
        {
            cx.notify(owner);
        }
    });
}
