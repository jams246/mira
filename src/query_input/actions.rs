use gpui::{
    App, Bounds, ClipboardItem, Context, EntityInputHandler, KeyBinding, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Point, ShapedLine, Window, actions,
};

use super::{QueryInput, coordinates};

actions!(
    query_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        SelectHome,
        SelectEnd,
        Paste,
        Cut,
        Copy
    ]
);

pub(crate) fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, Some("QueryInput")),
        KeyBinding::new("delete", Delete, Some("QueryInput")),
        KeyBinding::new("left", Left, Some("QueryInput")),
        KeyBinding::new("right", Right, Some("QueryInput")),
        KeyBinding::new("shift-left", SelectLeft, Some("QueryInput")),
        KeyBinding::new("shift-right", SelectRight, Some("QueryInput")),
        KeyBinding::new("ctrl-a", SelectAll, Some("QueryInput")),
        KeyBinding::new("ctrl-v", Paste, Some("QueryInput")),
        KeyBinding::new("ctrl-c", Copy, Some("QueryInput")),
        KeyBinding::new("ctrl-x", Cut, Some("QueryInput")),
        KeyBinding::new("home", Home, Some("QueryInput")),
        KeyBinding::new("end", End, Some("QueryInput")),
        KeyBinding::new("ctrl-home", Home, Some("QueryInput")),
        KeyBinding::new("ctrl-end", End, Some("QueryInput")),
        KeyBinding::new("shift-home", SelectHome, Some("QueryInput")),
        KeyBinding::new("shift-end", SelectEnd, Some("QueryInput")),
    ]);
}

impl QueryInput {
    pub(super) fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        let offset = if self.selected.is_empty() {
            coordinates::previous(&self.content, self.cursor())
        } else {
            self.selected.start
        };
        self.move_to(offset, cx);
    }

    pub(super) fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        let offset = if self.selected.is_empty() {
            coordinates::next(&self.content, self.cursor())
        } else {
            self.selected.end
        };
        self.move_to(offset, cx);
    }

    pub(super) fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(coordinates::previous(&self.content, self.cursor()), cx);
    }

    pub(super) fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(coordinates::next(&self.content, self.cursor()), cx);
    }

    pub(super) fn select_everything(
        &mut self,
        _: &SelectAll,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_all(window, cx);
    }

    pub(super) fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
    }

    pub(super) fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), cx);
    }

    pub(super) fn select_home(&mut self, _: &SelectHome, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(0, cx);
    }

    pub(super) fn select_end(&mut self, _: &SelectEnd, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.content.len(), cx);
    }

    pub(super) fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected.is_empty() {
            self.select_to(coordinates::previous(&self.content, self.cursor()), cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub(super) fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected.is_empty() {
            self.select_to(coordinates::next(&self.content, self.cursor()), cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub(super) fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_text_in_range(None, &text, window, cx);
        }
    }

    pub(super) fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected.clone()].to_owned(),
            ));
        }
    }

    pub(super) fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        self.copy(&Copy, window, cx);
        if !self.selected.is_empty() {
            self.replace_text_in_range(None, "", window, cx);
        }
    }

    pub(super) fn cursor(&self) -> usize {
        if self.reversed {
            self.selected.start
        } else {
            self.selected.end
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected = offset..offset;
        self.reversed = false;
        cx.notify();
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let anchor = if self.reversed {
            self.selected.end
        } else {
            self.selected.start
        };
        self.reversed = offset < anchor;
        self.selected = anchor.min(offset)..anchor.max(offset);
        cx.notify();
    }

    fn mouse_index(&self, point: Point<Pixels>) -> usize {
        mouse_index(
            self.content.len(),
            self.last_bounds.as_ref(),
            self.last_layout.as_ref(),
            self.scroll,
            point,
        )
    }

    pub(super) fn mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus);
        self.selecting = true;
        let index = self.mouse_index(event.position);
        if event.click_count > 1 {
            self.select_all(window, cx);
        } else if event.modifiers.shift {
            self.select_to(index, cx);
        } else {
            self.move_to(index, cx);
        }
        cx.stop_propagation();
    }

    pub(super) fn mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        if std::mem::take(&mut self.selecting) {
            cx.stop_propagation();
        }
    }

    pub(super) fn mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selecting {
            self.select_to(self.mouse_index(event.position), cx);
            cx.stop_propagation();
        }
    }
}

pub(super) fn mouse_index(
    content_len: usize,
    bounds: Option<&Bounds<Pixels>>,
    line: Option<&ShapedLine>,
    scroll: Pixels,
    point: Point<Pixels>,
) -> usize {
    match (bounds, line) {
        (Some(bounds), Some(line)) => line
            .closest_index_for_x(point.x - bounds.left() + scroll)
            .min(content_len),
        _ => 0,
    }
}
