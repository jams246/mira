use gpui::{App, Window};
use mira::flow_index::{FlowIndex, PieceKind};

use super::{Input, Row};
use crate::{
    image_cache::ImageStatus,
    render_text::TextKind,
    text_cache::{self, IntrinsicWidth, Source, TextCache},
};

pub(crate) const MIN_SPAN_BYTES: usize = text_cache::MIN_ASYNC_BYTES;

pub(crate) struct Span {
    pub piece: usize,
    pub source: Source,
    pub text: text_cache::EntryHandle,
    pub row: Option<Row>,
    extra_height: f32,
}

impl Span {
    pub(crate) fn height(&self, line_height: f32) -> f32 {
        self.text.borrow().estimated_rows() as f32 * line_height + self.extra_height
    }

    pub(super) fn set_row(&mut self, row: Row, line_height: f32) {
        self.extra_height = (row.height - self.height(line_height)).max(0.);
        self.row = Some(row);
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
    let document = &input.resources.loaded.document;
    let runs = Source::whole(input.position.block, input.resources.cell).runs(document);
    for ordinal in range {
        let piece = &index.pieces()[ordinal];
        if piece.range.len() < MIN_SPAN_BYTES {
            continue;
        }
        let kind = match piece.kind {
            PieceKind::Text => input.format.kind,
            PieceKind::Image { inline } => {
                let target = runs[inline].image?;
                if matches!(
                    input.resources.images.borrow().peek(target),
                    ImageStatus::Ready(_)
                ) {
                    continue;
                }
                TextKind::Body
            }
            PieceKind::Break => continue,
        };
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
                kind,
            },
            Some(index.inline_index()),
        );
        if !text.borrow().complete {
            return Some(Span {
                piece: ordinal,
                source,
                text,
                row: None,
                extra_height: 0.,
            });
        }
    }
    None
}

pub(super) fn known(span: &Span) -> bool {
    span.text.borrow().intrinsic_width() != IntrinsicWidth::Unknown
}
