mod actions;
mod coordinates;
mod element;
mod input_handler;
#[cfg(test)]
mod tests;

use std::ops::Range;

use gpui::{
    App, Bounds, Context, CursorStyle, EventEmitter, FocusHandle, Focusable, MouseButton, Pixels,
    ShapedLine, SharedString, Window, div, prelude::*, px,
};

use crate::appearance::Palette;
pub(crate) use actions::bind_keys;

pub(crate) struct QueryChanged;

pub(crate) struct QueryInput {
    focus: FocusHandle,
    content: SharedString,
    selected: Range<usize>,
    reversed: bool,
    marked: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    scroll: Pixels,
    selecting: bool,
}

impl QueryInput {
    pub(crate) fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle().tab_stop(true),
            content: "".into(),
            selected: 0..0,
            reversed: false,
            marked: None,
            last_layout: None,
            last_bounds: None,
            scroll: px(0.),
            selecting: false,
        }
    }

    pub(crate) fn query(&self) -> &str {
        &self.content
    }

    pub(crate) fn select_all(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.selected = 0..self.content.len();
        self.reversed = false;
        cx.notify();
    }

    fn replace(&mut self, range: Range<usize>, text: &str, cx: &mut Context<Self>) {
        if self.content[range.clone()] != *text {
            self.content = format!(
                "{}{}{}",
                &self.content[..range.start],
                text,
                &self.content[range.end..]
            )
            .into();
            self.last_layout = None;
            cx.emit(QueryChanged);
        }
    }
}

impl EventEmitter<QueryChanged> for QueryInput {}

impl Focusable for QueryInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for QueryInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::for_window(window);
        div()
            .id("find-query")
            .key_context("QueryInput")
            .track_focus(&self.focus)
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_everything))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::select_home))
            .on_action(cx.listener(Self::select_end))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::mouse_up))
            .on_mouse_move(cx.listener(Self::mouse_move))
            .flex_1()
            .min_w_0()
            .h(px(28.))
            .px_2()
            .border_1()
            .border_color(palette.border)
            .rounded_sm()
            .bg(palette.background)
            .text_color(palette.text)
            .text_size(px(14.))
            .line_height(px(24.))
            .focus(|style| style.border_color(palette.link))
            .child(element::QueryText { input: cx.entity() })
    }
}
