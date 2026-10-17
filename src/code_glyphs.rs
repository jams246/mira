mod interaction;

use std::rc::Rc;

use gpui::{Bounds, Hitbox, Pixels, Point, ShapedLine, Window, fill, point, px, rgb, size};

use crate::{
    code_cache::{self, Chunk, Entry, Source},
    render_context::RenderContext,
    text_map::TextMap,
};

struct Piece {
    chunk: Chunk,
    line: ShapedLine,
    mapping: TextMap,
}

pub(crate) struct CodeGlyphs {
    pieces: Rc<Vec<Piece>>,
    source: Source,
    start: usize,
    bounds: Bounds<Pixels>,
    font_size: Pixels,
    hitbox: Hitbox,
    status: Option<ShapedLine>,
    error: bool,
}

impl CodeGlyphs {
    pub(crate) fn prepare(
        resources: &RenderContext,
        source: Source,
        entry: &Entry,
        bounds: Bounds<Pixels>,
        font_size: Pixels,
        window: &mut Window,
    ) -> Self {
        let mask = window.content_mask().bounds;
        let left = f32::from((mask.left() - bounds.left()).max(px(0.))) - 256.;
        let right = f32::from(mask.right() - bounds.left()) + 256.;
        let first = entry
            .chunks
            .partition_point(|chunk| chunk.x + chunk.width < left)
            .saturating_sub(1);
        let pieces: Vec<_> = entry.chunks[first..]
            .iter()
            .take_while(|chunk| chunk.x <= right)
            .filter(|_| entry.complete && entry.error.is_none() || entry.width >= left)
            .map(|chunk| {
                let (text, mapping) = code_cache::prepare(&resources.loaded, source, chunk);
                let line = code_cache::native(window.text_system(), text, font_size.into());
                Piece {
                    chunk: chunk.clone(),
                    line,
                    mapping,
                }
            })
            .collect();
        let status = pieces.is_empty().then(|| {
            code_cache::native(
                window.text_system(),
                entry
                    .error
                    .clone()
                    .unwrap_or_else(|| "Loading code...".into()),
                font_size.into(),
            )
        });
        Self {
            pieces: Rc::new(pieces),
            source,
            start: entry.line.start,
            bounds,
            font_size,
            hitbox: window.insert_hitbox(bounds, Default::default()),
            status,
            error: entry.error.is_some(),
        }
    }

    pub(crate) fn paint(&self, resources: &RenderContext, window: &mut Window) -> bool {
        if let Some(status) = &self.status {
            let origin = point(
                self.bounds.left().max(window.content_mask().bounds.left()),
                self.bounds.top(),
            );
            let baseline =
                (self.bounds.size.height - status.ascent - status.descent) / 2. + status.ascent;
            for run in &status.runs {
                for glyph in &run.glyphs {
                    let _ = window.paint_glyph(
                        origin + glyph.position + point(px(0.), baseline),
                        run.font_id,
                        glyph.id,
                        self.font_size,
                        window.text_style().color,
                    );
                }
            }
        }
        interaction::register(self, resources, window);
        let color = window.text_style().color;
        for piece in self.pieces.iter() {
            let chunk = &piece.chunk;
            let origin = point(
                self.bounds.left() + px(chunk.x - chunk.local_x),
                self.bounds.top(),
            );
            if let Some(selection) = resources.selection.borrow().range_at(
                crate::selection::Position {
                    block: self.source.block,
                    byte: chunk.window.start,
                },
                chunk.window.len(),
            ) {
                let start = (chunk.window.start + selection.start).max(chunk.core.start);
                let end = (chunk.window.start + selection.end).min(chunk.core.end);
                if start < end {
                    let x = piece
                        .line
                        .x_for_index(piece.mapping.to_display(start - chunk.window.start));
                    let end = piece
                        .line
                        .x_for_index(piece.mapping.to_display(end - chunk.window.start));
                    window.paint_quad(fill(
                        Bounds::new(
                            point(origin.x + x, origin.y),
                            size((end - x).max(px(0.)), self.bounds.size.height),
                        ),
                        rgb(0xb6d7ff),
                    ));
                }
            }
            let baseline = (self.bounds.size.height - piece.line.ascent - piece.line.descent) / 2.
                + piece.line.ascent;
            for run in &piece.line.runs {
                for glyph in &run.glyphs {
                    let byte = chunk.window.start + piece.mapping.to_source(glyph.index);
                    if !chunk.core.contains(&byte) {
                        continue;
                    }
                    let position = origin + glyph.position + point(px(0.), baseline);
                    if glyph.is_emoji {
                        let _ = window.paint_emoji(position, run.font_id, glyph.id, self.font_size);
                    } else {
                        let _ = window.paint_glyph(
                            position,
                            run.font_id,
                            glyph.id,
                            self.font_size,
                            color,
                        );
                    }
                }
            }
        }
        let mask = window.content_mask().bounds;
        self.error && self.status.is_some() && self.bounds.intersects(&mask)
            || self.pieces.iter().any(|piece| {
                Bounds::new(
                    point(self.bounds.left() + px(piece.chunk.x), self.bounds.top()),
                    size(px(piece.chunk.width), self.bounds.size.height),
                )
                .intersects(&mask)
            })
    }
}

fn hit(pieces: &[Piece], origin: Point<Pixels>, position: Point<Pixels>) -> Option<usize> {
    let x = f32::from(position.x - origin.x);
    let piece = pieces.iter().min_by(|a, b| {
        let distance = |piece: &Piece| {
            (piece.chunk.x - x).max(0.) + (x - piece.chunk.x - piece.chunk.width).max(0.)
        };
        distance(a).total_cmp(&distance(b))
    })?;
    let chunk = &piece.chunk;
    let byte = piece
        .line
        .closest_index_for_x(px(x - chunk.x + chunk.local_x));
    Some(
        (chunk.window.start + piece.mapping.to_source(byte))
            .clamp(chunk.core.start, chunk.core.end),
    )
}
