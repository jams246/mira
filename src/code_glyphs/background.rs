use std::ops::Range;

use gpui::{Bounds, Hsla, Pixels, Point, Window, fill, point, px, size};

use crate::{render_context::RenderContext, selection::Position};

use super::Piece;

pub(super) fn paint(
    piece: &Piece,
    resources: &RenderContext,
    block: usize,
    origin: Point<Pixels>,
    height: Pixels,
    window: &mut Window,
) {
    let chunk = &piece.chunk;
    let position = Position {
        block,
        byte: chunk.window.start,
    };
    for (range, active) in
        resources
            .highlights
            .borrow()
            .ranges(position, resources.cell, chunk.window.len())
    {
        rectangle(
            piece,
            range,
            origin,
            height,
            if active {
                resources.palette.find_active
            } else {
                resources.palette.find
            },
            window,
        );
    }
    if let Some(range) = resources
        .selection
        .borrow()
        .range_at(position, chunk.window.len())
    {
        rectangle(
            piece,
            range,
            origin,
            height,
            resources.palette.selection,
            window,
        );
    }
}

fn rectangle(
    piece: &Piece,
    range: Range<usize>,
    origin: Point<Pixels>,
    height: Pixels,
    color: Hsla,
    window: &mut Window,
) {
    let chunk = &piece.chunk;
    let start = (chunk.window.start + range.start).max(chunk.core.start);
    let end = (chunk.window.start + range.end).min(chunk.core.end);
    if start >= end {
        return;
    }
    let x = piece
        .line
        .x_for_index(piece.mapping.to_display(start - chunk.window.start));
    let end = piece
        .line
        .x_for_index(piece.mapping.to_display(end - chunk.window.start));
    window.paint_quad(fill(
        Bounds::new(
            point(origin.x + x, origin.y),
            size((end - x).max(px(0.)), height),
        ),
        color,
    ));
}
