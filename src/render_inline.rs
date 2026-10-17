use std::rc::Rc;

use gpui::{AnyElement, App, IntoElement, TextAlign, Window, div, prelude::*};
use mira::document::{Document, Inline};

use crate::{
    inline_flow::{self, Piece},
    render_context::RenderContext,
    render_image::{self, ImageContent},
    render_text::{self, TextFormat},
    selection::Position,
};

pub(crate) fn content(
    document: &Document,
    position: Position,
    inlines: &[Inline],
    format: TextFormat,
    resources: &RenderContext,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    if !inlines.iter().any(|inline| inline.image.is_some()) {
        return text(document, position, inlines, format, resources);
    }
    let mut output = div()
        .w_full()
        .min_w_0()
        .flex()
        .flex_wrap()
        .items_baseline()
        .when(format.align == TextAlign::Center, |row| {
            row.justify_center()
        })
        .when(format.align == TextAlign::Right, |row| row.justify_end());
    for piece in inline_flow::pieces(document, inlines) {
        output = match piece {
            Piece::Text { byte, runs } => output.child(text(
                document,
                Position {
                    byte: position.byte + byte,
                    ..position
                },
                &runs,
                TextFormat {
                    inline: true,
                    ..format
                },
                resources,
            )),
            Piece::Image { byte, index } => {
                let inline = &inlines[index];
                if let Some(target) = inline.image {
                    output.child(render_image::image(
                        document,
                        Position {
                            byte: position.byte + byte,
                            ..position
                        },
                        ImageContent {
                            target,
                            alt: std::slice::from_ref(inline),
                            link: inline.link,
                            occurrence: index,
                        },
                        resources,
                        window,
                        cx,
                    ))
                } else {
                    output
                }
            }
            Piece::Break => output.child(div().w_full().h_0()),
        };
    }
    output.into_any_element()
}

fn text(
    document: &Document,
    position: Position,
    runs: &[Inline],
    format: TextFormat,
    resources: &RenderContext,
) -> AnyElement {
    render_text::text(
        document,
        position,
        runs,
        format,
        Rc::clone(&resources.links),
        Rc::clone(&resources.selection),
        resources.owner,
    )
}
