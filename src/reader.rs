use std::{rc::Rc, sync::Arc};

use gpui::{App, IntoElement, ListState, Window, div, list, prelude::*};

use crate::{
    file::LoadedDocument, render_block, render_text::LinkHandler, selection::SelectionHandle,
};

pub(crate) fn viewport(
    loaded: Arc<LoadedDocument>,
    state: ListState,
    activate_link: LinkHandler,
    selection: SelectionHandle,
) -> impl IntoElement {
    let list = list(state, move |index, window: &mut Window, cx: &mut App| {
        render_block::block(
            &loaded.document,
            index,
            Rc::clone(&activate_link),
            Rc::clone(&selection),
            window,
            cx,
        )
    })
    .size_full();
    div().w_full().flex_1().min_h_0().px_8().py_6().child(list)
}
