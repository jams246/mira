use std::rc::Rc;

use gpui::{
    AnyView, App, AppContext, Context, Div, ElementId, KeyBinding, SharedString, Stateful, Window,
    actions, div, prelude::*, px,
};

use crate::appearance::Palette;

actions!(mira_controls, [Activate]);

pub(crate) fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("enter", Activate, Some("MiraButton")),
        KeyBinding::new("space", Activate, Some("MiraButton")),
    ]);
}

pub(crate) fn button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    description: impl Into<SharedString>,
    enabled: bool,
    palette: Palette,
    activate: impl Fn(&mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let activate = Rc::new(activate);
    let click = Rc::clone(&activate);
    let description = description.into();
    div()
        .id(id)
        .key_context("MiraButton")
        .h(px(28.))
        .px_2()
        .flex()
        .items_center()
        .justify_center()
        .flex_shrink_0()
        .rounded_sm()
        .border_1()
        .border_color(palette.background)
        .text_size(px(13.))
        .text_color(if enabled { palette.text } else { palette.muted })
        .when(enabled, |button| {
            button
                .cursor_pointer()
                .tab_index(0)
                .hover(move |style| style.bg(palette.panel))
                .focus(move |style| style.border_color(palette.link))
        })
        .tooltip(move |_, cx| tip(description.clone(), palette, cx))
        .on_click(move |_, window, cx| {
            cx.stop_propagation();
            if enabled {
                click(window, cx);
            }
        })
        .on_action(move |_: &Activate, window, cx| {
            if enabled {
                activate(window, cx);
            }
        })
        .child(label.into())
}

pub(crate) fn tip(text: SharedString, palette: Palette, cx: &mut App) -> AnyView {
    cx.new(|_| Tip { text, palette }).into()
}

struct Tip {
    text: SharedString,
    palette: Palette,
}

impl Render for Tip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_2()
            .py_1()
            .rounded_sm()
            .border_1()
            .border_color(self.palette.border)
            .bg(self.palette.panel)
            .text_color(self.palette.text)
            .text_size(px(12.))
            .child(self.text.clone())
    }
}
