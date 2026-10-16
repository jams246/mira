use gpui::{AnyElement, App, IntoElement, Window, div, prelude::*, px, relative, rgb};
use mira::document::{BlockKind, ContextKind, Document};

use crate::{
    render_text::{self, LinkHandler, TextKind},
    selection::SelectionHandle,
};

pub(crate) fn block(
    document: &Document,
    index: usize,
    activate_link: LinkHandler,
    selection: SelectionHandle,
    window: &mut Window,
    _cx: &mut App,
) -> AnyElement {
    let block = &document.blocks[index];
    let owner = window.current_view();
    let mut context = block.context;
    let mut ancestors = Vec::new();
    while let Some(index) = context {
        let ancestor = &document.contexts[index];
        ancestors.push(&ancestor.kind);
        context = ancestor.parent;
        // shortcut: Show eight ancestor frames, expand when deeper nesting needs distinct borders.
        if ancestors.len() == 8 {
            break;
        }
    }
    let mut content = div().w_full().min_w_0();
    let mut size = 16.;
    let mut margin = 12.;
    match &block.kind {
        BlockKind::Paragraph(runs) | BlockKind::RawHtml(runs) => {
            content = content.child(render_text::text(
                document,
                index,
                runs,
                TextKind::Body,
                activate_link,
                selection,
                owner,
            ));
        }
        BlockKind::Heading { level, runs } => {
            size = [32., 27., 23., 20., 18., 16.][usize::from(*level).saturating_sub(1).min(5)];
            margin = 16.;
            content = content.pt_3().child(render_text::text(
                document,
                index,
                runs,
                TextKind::Heading,
                activate_link,
                selection,
                owner,
            ));
        }
        BlockKind::Rule => {
            content = content.my_3().h(px(1.)).bg(rgb(0xd8d8d8));
        }
        BlockKind::Code { runs, .. } => {
            content = content.child(
                div()
                    .id(("code", index))
                    .w_full()
                    .flex()
                    .overflow_x_scroll()
                    .p_3()
                    .bg(rgb(0xeeeeee))
                    .rounded_md()
                    .text_size(px(14.))
                    .child(render_text::text(
                        document,
                        index,
                        runs,
                        TextKind::Code,
                        activate_link,
                        selection,
                        owner,
                    )),
            );
        }
        BlockKind::TableRow { .. } => content = content.child("[Table row: rendering pending]"),
        BlockKind::Image { .. } => content = content.child("[Local image: rendering pending]"),
    }
    let width: f32 = window.viewport_size().width.into();
    let gutter = (width * 0.3 / ancestors.len().max(1) as f32).min(28.);
    let mut first_item = true;
    for ancestor in ancestors {
        content = match ancestor {
            ContextKind::Quote => div()
                .w_full()
                .min_w_0()
                .border_l_2()
                .border_color(rgb(0xc4c8cc))
                .pl(px(gutter.min(16.)))
                .child(content),
            ContextKind::ListItem { number, checked } => {
                let marker = if first_item {
                    first_item = false;
                    let label = match checked {
                        Some(true) => "☑".into(),
                        Some(false) => "☐".into(),
                        None => number.map_or_else(|| "•".into(), |number| format!("{number}.")),
                    };
                    div()
                        .min_w(px(28.))
                        .flex_none()
                        .whitespace_nowrap()
                        .pr_2()
                        .child(label)
                        .when(!block.marker, |marker| marker.opacity(0.))
                } else {
                    div().w(px(gutter)).flex_none()
                };
                div()
                    .w_full()
                    .min_w_0()
                    .flex()
                    .items_start()
                    .child(marker)
                    .child(div().min_w_0().flex_1().child(content))
            }
        };
    }
    div()
        .w_full()
        .text_size(px(size))
        .line_height(relative(1.5))
        .pb(px(margin))
        .child(content)
        .into_any_element()
}
