use gpui::{Context, Div, Window, div, prelude::*, px};

use super::{Open, Viewer};
use crate::appearance::Palette;

impl Viewer {
    pub(super) fn empty(
        &self,
        root: Div,
        palette: Palette,
        window: &Window,
        _cx: &mut Context<Self>,
    ) -> Div {
        let message = div()
            .w(px(
                (f32::from(window.viewport_size().width) - 64.).clamp(1., 720.)
            ))
            .text_center()
            .child(self.message.clone());
        let message = if self.message.starts_with("Cannot open ") {
            self.content_paint
                .wrap(message.text_color(palette.error).into_any_element())
        } else {
            message.into_any_element()
        };
        root.justify_center()
            .items_center()
            .p_8()
            .gap_3()
            .child(div().text_3xl().child("MIRA"))
            .child(message)
            .child(
                div()
                    .id("open")
                    .cursor_pointer()
                    .px_4()
                    .py_2()
                    .rounded_md()
                    .border_1()
                    .border_color(palette.border)
                    .hover(move |style| style.bg(palette.panel))
                    .on_click(|_, window, cx| window.dispatch_action(Box::new(Open), cx))
                    .child("Open Markdown (Ctrl+O)"),
            )
    }
}
