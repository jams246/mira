use gpui::{App, Window};
use mira::flow_index::{FlowIndex, PieceKind};

use super::{Input, Row};
use crate::text_cache::{self, IntrinsicWidth, Source, TextCache};

pub(crate) const MIN_SPAN_BYTES: usize = 32 * 1024;

pub(crate) struct Span {
    pub piece: usize,
    pub source: Source,
    pub text: text_cache::EntryHandle,
    pub row: Option<Row>,
}

impl Span {
    pub(crate) fn height(&self, line_height: f32) -> f32 {
        self.text.borrow().estimated_rows() as f32 * line_height
    }

    pub(super) fn demand(
        &self,
        input: &Input<'_>,
        target: f32,
        line_height: f32,
        window: &mut Window,
        cx: &mut App,
    ) {
        let top = self.row.as_ref().map_or(0., |row| row.top);
        let rows = if !target.is_finite() {
            usize::MAX
        } else if self.row.is_none() {
            2
        } else {
            ((target - top).max(0.) / line_height).ceil() as usize + 8
        };
        TextCache::demand(
            &input.resources.texts,
            &self.text,
            self.source,
            rows,
            window,
            cx,
        );
    }
}

pub(super) fn find(
    index: &FlowIndex,
    range: std::ops::Range<usize>,
    input: &Input<'_>,
    window: &Window,
) -> Option<Span> {
    for ordinal in range {
        let piece = &index.pieces()[ordinal];
        if piece.kind != PieceKind::Text || piece.range.len() < MIN_SPAN_BYTES {
            continue;
        }
        let source = Source {
            span: Some((piece.range.start, piece.range.end)),
            ..Source::whole(input.position.block, input.resources.cell)
        };
        let font_size = window.text_style().font_size.to_pixels(window.rem_size());
        let text = TextCache::request(
            &input.resources.texts,
            source,
            text_cache::Config {
                width: input.width,
                font_size: font_size.into(),
                kind: input.format.kind,
            },
            Some(index.inline_index()),
        );
        if !text.borrow().complete {
            return Some(Span {
                piece: ordinal,
                source,
                text,
                row: None,
            });
        }
    }
    None
}

pub(super) fn known(span: &Span) -> bool {
    span.text.borrow().intrinsic_width() != IntrinsicWidth::Unknown
}
