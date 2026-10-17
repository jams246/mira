use gpui::{AnyElement, App, IntoElement, Window, div, prelude::*, px, relative, rgb};
use mira::document::{BlockKind, ContextKind, Document};

use crate::{
    render_context::RenderContext,
    render_image::{self, ImageContent},
    render_inline, render_table,
    render_text::TextKind,
    selection::Position,
};

pub(crate) fn block(
    document: &Document,
    index: usize,
    resources: &RenderContext,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let block = &document.blocks[index];
    let position = Position {
        block: index,
        byte: 0,
    };
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
    let width: f32 = window.viewport_size().width.into();
    let gutter = (width * 0.3 / ancestors.len().max(1) as f32).min(28.);
    let mut resources = resources.clone();
    let mut content = div().w_full().min_w_0();
    let size = match &block.kind {
        BlockKind::Heading { level, .. } => {
            [32., 27., 23., 20., 18., 16.][usize::from(*level).saturating_sub(1).min(5)]
        }
        _ => 16.,
    };
    let has_images = match &block.kind {
        BlockKind::Image { .. } => true,
        BlockKind::Paragraph(runs) | BlockKind::RawHtml(runs) | BlockKind::Heading { runs, .. } => {
            runs.iter().any(|run| run.image.is_some())
        }
        _ => false,
    };
    if has_images {
        resources.width = (resources.width - image_inset(&ancestors, gutter, size, window)).max(1.);
    }
    let mut margin = 12.;
    match &block.kind {
        BlockKind::Paragraph(runs) | BlockKind::RawHtml(runs) => {
            content = content.child(render_inline::content(
                document,
                position,
                runs,
                TextKind::Body.into(),
                &resources,
                window,
                cx,
            ));
        }
        BlockKind::Heading { runs, .. } => {
            margin = 16.;
            content = content.pt_3().child(render_inline::content(
                document,
                position,
                runs,
                TextKind::Heading.into(),
                &resources,
                window,
                cx,
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
                    .child(render_inline::content(
                        document,
                        position,
                        runs,
                        TextKind::Code.into(),
                        &resources,
                        window,
                        cx,
                    )),
            );
        }
        BlockKind::TableRow { table, .. } => {
            if document.blocks.get(index + 1).is_some_and(|next| matches!(next.kind, BlockKind::TableRow { table: next_table, .. } if next_table == *table)) {
                margin = 0.;
            }
            content = content.child(render_table::row(document, index, &resources, window, cx));
        }
        BlockKind::Image { target, alt, link } => {
            content = content.child(render_image::image(
                document,
                position,
                ImageContent {
                    target: *target,
                    alt,
                    link: *link,
                    occurrence: 0,
                },
                &resources,
                window,
                cx,
            ));
        }
    }
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
                    let label = marker_label(*number, *checked);
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

fn marker_label(number: Option<u64>, checked: Option<bool>) -> String {
    match checked {
        Some(true) => "☑".into(),
        Some(false) => "☐".into(),
        None => number.map_or_else(|| "•".into(), |number| format!("{number}.")),
    }
}

fn image_inset(ancestors: &[&ContextKind], gutter: f32, size: f32, window: &Window) -> f32 {
    let mut nearest_item = true;
    ancestors
        .iter()
        .map(|ancestor| match ancestor {
            ContextKind::Quote => gutter.min(16.) + 2.,
            ContextKind::ListItem { number, checked } if nearest_item => {
                nearest_item = false;
                let label = marker_label(*number, *checked);
                let run = window.text_style().to_run(label.len());
                let line = window
                    .text_system()
                    .shape_line(label.into(), px(size), &[run], None);
                (f32::from(line.width).ceil() + 8.).max(28.)
            }
            ContextKind::ListItem { .. } => gutter,
        })
        .sum()
}
