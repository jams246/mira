use gpui::{AnyElement, App, IntoElement, Window, div, prelude::*, px, relative, rgb};
use mira::document::{BlockKind, ContextKind, Document};

use crate::render_text::{self, LinkHandler};

pub(crate) fn block(
    document: &Document,
    index: usize,
    activate_link: LinkHandler,
    window: &mut Window,
    _cx: &mut App,
) -> AnyElement {
    let block = &document.blocks[index];
    let mut context = block.context;
    let mut list_depth = 0usize;
    let mut quotes = 0usize;
    let mut marker = None;
    while let Some(index) = context {
        let ancestor = &document.contexts[index];
        match ancestor.kind {
            ContextKind::Quote => quotes += 1,
            ContextKind::ListItem { number, checked } => {
                if marker.is_none() && block.marker {
                    marker = Some(match checked {
                        Some(true) => "☑".into(),
                        Some(false) => "☐".into(),
                        None => number.map_or_else(|| "•".into(), |number| format!("{number}.")),
                    });
                }
                list_depth += 1;
            }
        }
        context = ancestor.parent;
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
                false,
                activate_link,
            ));
        }
        BlockKind::Heading { level, runs } => {
            size = [32., 27., 23., 20., 18., 16.][usize::from(*level).saturating_sub(1).min(5)];
            margin = 16.;
            content = content.pt_3().child(render_text::text(
                document,
                index,
                runs,
                true,
                activate_link,
            ));
        }
        BlockKind::Rule => {
            content = content.my_3().h(px(1.)).bg(rgb(0xd8d8d8));
        }
        BlockKind::Code { .. } => content = content.child("[Code block: rendering pending]"),
        BlockKind::TableRow { .. } => content = content.child("[Table row: rendering pending]"),
        BlockKind::Image { .. } => content = content.child("[Local image: rendering pending]"),
    }
    let width: f32 = window.viewport_size().width.into();
    let indent = (list_depth.saturating_sub(1).min(7) as f32 * 26.
        + quotes.saturating_sub(1).min(7) as f32 * 16.)
        .min(width * 0.3);
    div()
        .w_full()
        .flex()
        .items_start()
        .text_size(px(size))
        .line_height(relative(1.5))
        .pb(px(margin))
        .pl(px(indent))
        .when(quotes > 0, |row| {
            row.border_l_2()
                .border_color(rgb(0xc4c8cc))
                .pl(px(indent + 16.))
        })
        .when(list_depth > 0, |row| {
            row.child(
                div()
                    .w(px(28.))
                    .flex_none()
                    .child(marker.unwrap_or_default()),
            )
        })
        .child(content)
        .into_any_element()
}
