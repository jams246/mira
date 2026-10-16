use gpui::{Context, Window, div, prelude::*, rgb};

pub(crate) struct Viewer;

impl Render for Viewer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .gap_3()
            .bg(rgb(0xfafafa))
            .text_color(rgb(0x242424))
            .font_family("Segoe UI")
            .child(div().text_3xl().child("MIRA"))
            .child("Open a Markdown file to start reading.")
    }
}
