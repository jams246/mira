use std::sync::Arc;

use gpui::{App, IntoElement, ListState, Window, canvas, div, list, prelude::*};

use crate::{file::LoadedDocument, render_block, render_context::RenderContext};

pub(crate) fn viewport(
    loaded: Arc<LoadedDocument>,
    state: ListState,
    resources: RenderContext,
) -> impl IntoElement {
    let reveal = resources.reveal.clone();
    let viewport = canvas(
        |_, _, _| (),
        move |bounds, (), _, _| {
            reveal.borrow_mut().begin_frame(bounds);
        },
    )
    .absolute()
    .inset_0()
    .size_full();
    let list = list(state, move |index, window: &mut Window, cx: &mut App| {
        render_block::block(&loaded.document, index, &resources, window, cx)
    })
    .size_full();
    div()
        .w_full()
        .flex_1()
        .min_h_0()
        .px_8()
        .py_6()
        .child(div().relative().size_full().child(viewport).child(list))
}
