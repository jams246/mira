use crate::{controls, render_context::RenderContext};
use gpui::{AnyElement, ClipboardItem, IntoElement, div, prelude::*, px};
use mira::document::BlockKind;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

static COPY_REVISION: AtomicU64 = AtomicU64::new(0);

pub(crate) fn panel(block: usize, text: AnyElement, resources: &RenderContext) -> AnyElement {
    let loaded = Arc::clone(&resources.loaded);
    let enabled = matches!(&loaded.document.blocks[block].kind, BlockKind::Code { runs, .. }
        if runs.iter().any(|run| !run.text.as_str(&loaded.document.source).is_empty()));
    let copy = move |_: &mut gpui::Window, cx: &mut gpui::App| {
        let loaded = Arc::clone(&loaded);
        let revision = COPY_REVISION.fetch_add(1, Ordering::Relaxed) + 1;
        let task = cx
            .background_executor()
            .spawn(async move { mira::search::code_text(&loaded.document, block) });
        cx.spawn(async move |cx| {
            if let Some(text) = task.await {
                let _ = cx.update(|cx| {
                    if COPY_REVISION.load(Ordering::Relaxed) == revision {
                        let timing = crate::timing::Timing::start();
                        cx.write_to_clipboard(ClipboardItem::new_string(text));
                        timing.mark("copy_callback");
                    }
                });
            }
        })
        .detach();
    };
    let scroll = resources
        .code_scrolls
        .borrow_mut()
        .entry(block)
        .or_default()
        .clone();
    let palette = resources.palette;
    div()
        .w_full()
        .min_w_0()
        .bg(palette.panel)
        .rounded_md()
        .child(
            div()
                .h(px(28.))
                .flex()
                .justify_end()
                .text_size(px(12.))
                .child(controls::button(
                    ("copy-code", block),
                    "Copy",
                    "Copy code block",
                    enabled,
                    palette,
                    copy,
                )),
        )
        .child(
            div()
                .id(("code", block))
                .w_full()
                .flex()
                .overflow_x_scroll()
                .track_scroll(&scroll)
                .map(|mut scroller| {
                    scroller.style().restrict_scroll_to_axis = Some(true);
                    scroller
                })
                .p_3()
                .text_size(px(14. * resources.scale))
                .child(text),
        )
        .into_any_element()
}
