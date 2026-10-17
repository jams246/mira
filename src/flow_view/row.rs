use std::{ops::Range, rc::Rc};

use gpui::{AnyElement, App, IntoElement, TextAlign, Window, div, prelude::*};
use mira::flow_index::{FlowIndex, PieceKind};

use crate::{
    flow_cache::{
        Input,
        measure::{Measured, MeasuredIds},
    },
    render_image::{self, ImageContent},
    render_inline,
    render_text::TextFormat,
    selection::Position,
    text_cache::Source,
    wrapped_text::WrappedText,
};

pub(crate) fn build(
    index: &FlowIndex,
    pieces: Range<usize>,
    input: &Input<'_>,
    measuring: bool,
    ids: Option<&MeasuredIds>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let resources = input.resources;
    let document = &resources.loaded.document;
    let runs = Source::whole(input.position.block, resources.cell).runs(document);
    let style = window.text_style();
    let font_size = style.font_size.to_pixels(window.rem_size());
    let line_height = style
        .line_height
        .to_pixels(font_size.into(), window.rem_size());
    let mut output = div()
        .id(gpui::SharedString::from(format!(
            "flow-row-{}-{}-{}-{}-{measuring}",
            input.position.block, input.position.byte, pieces.start, pieces.end
        )))
        .w_full()
        .min_w_0()
        .flex()
        .flex_wrap()
        .items_baseline()
        .when(input.format.align == TextAlign::Center, |row| {
            row.justify_center()
        })
        .when(input.format.align == TextAlign::Right, |row| {
            row.justify_end()
        });
    for ordinal in pieces {
        let piece = &index.pieces()[ordinal];
        let position = Position {
            byte: input.position.byte + piece.range.start,
            ..input.position
        };
        let child = match piece.kind {
            PieceKind::Text if piece.range.len() >= crate::flow_cache::span::MIN_SPAN_BYTES => {
                let mut resources = resources.clone();
                resources.width = input.width;
                div()
                    .flex_none()
                    .max_w_full()
                    .child(WrappedText::new_span(
                        input.position,
                        (piece.range.start, piece.range.end),
                        TextFormat {
                            inline: false,
                            ..input.format
                        },
                        &resources,
                        index.inline_index(),
                    ))
                    .into_any_element()
            }
            PieceKind::Text => render_inline::text(
                document,
                position,
                &index.slice(document, runs, piece.range.clone()),
                TextFormat {
                    inline: true,
                    ..input.format
                },
                resources,
            ),
            PieceKind::Image { inline } => {
                let run = &runs[inline];
                let mut resources = resources.clone();
                resources.width = input.width;
                render_image::image(
                    document,
                    position,
                    ImageContent {
                        target: run.image.expect("indexed image"),
                        alt: std::slice::from_ref(run),
                        link: run.link,
                        occurrence: inline,
                        span_source: Some((input.position, index.inline_index())),
                    },
                    &resources,
                    window,
                    cx,
                    !measuring,
                )
            }
            PieceKind::Break => div()
                .w_full()
                .h(
                    if ordinal == 0 || index.pieces()[ordinal - 1].kind == PieceKind::Break {
                        line_height
                    } else {
                        gpui::px(0.)
                    },
                )
                .into_any_element(),
        };
        output = output.child(if let Some(ids) = ids {
            Measured {
                child,
                piece: ordinal,
                ids: Rc::clone(ids),
            }
            .into_any_element()
        } else {
            child
        });
    }
    output.into_any_element()
}
