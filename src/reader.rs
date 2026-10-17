use std::sync::Arc;

use gpui::{App, IntoElement, ListState, Window, div, list, prelude::*};

use crate::{file::LoadedDocument, render_block, render_context::RenderContext};

pub(crate) fn viewport(
    loaded: Arc<LoadedDocument>,
    state: ListState,
    resources: RenderContext,
) -> impl IntoElement {
    let list = list(state, move |index, window: &mut Window, cx: &mut App| {
        render_block::block(&loaded.document, index, &resources, window, cx)
    })
    .size_full();
    div().w_full().flex_1().min_h_0().px_8().py_6().child(list)
}
