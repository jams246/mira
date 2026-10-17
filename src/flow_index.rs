//! Source-backed word and image occurrences for native inline flex layout.

use std::ops::Range;

use crate::{
    document::{Document, Inline},
    inline_index::InlineIndex,
};

/// One native inline flow item's content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceKind {
    /// Text ending at whitespace or the next image or hard break.
    Text,
    /// An image identified by its occurrence in the original inline sequence.
    Image {
        /// Original inline index, distinct even when alternative text is empty.
        inline: usize,
    },
    /// A mandatory line break owning one semantic LF byte.
    Break,
}

/// An ordered occurrence within the normalized inline text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    /// Semantic UTF-8 bytes; empty image occurrences retain a zero-length range.
    pub range: Range<usize>,
    /// Native flow content or original image occurrence.
    pub kind: PieceKind,
}

/// Immutable flow metadata without retained per-word text or inline fragments.
///
/// Use the original document and inline sequence when slicing. Progress through
/// pieces by their ordinal because consecutive empty images can share all byte
/// coordinates. Non-LF Unicode whitespace follows `char::is_whitespace`.
pub struct FlowIndex {
    inlines: InlineIndex,
    pieces: Vec<Piece>,
}

impl FlowIndex {
    /// Scans borrowed text once, keeping words together across style boundaries.
    pub fn new(document: &Document, runs: &[Inline]) -> Self {
        let mut pieces = Vec::new();
        let mut word_start = None;
        let mut byte = 0;
        for (inline, run) in runs.iter().enumerate() {
            let text = run.text.as_str(&document.source);
            let end = byte + text.len();
            if run.image.is_some() {
                flush(&mut pieces, &mut word_start, byte);
                pieces.push(Piece {
                    range: byte..end,
                    kind: PieceKind::Image { inline },
                });
            } else {
                for (offset, ch) in text.char_indices() {
                    let start = byte + offset;
                    if ch == '\n' {
                        flush(&mut pieces, &mut word_start, start);
                        pieces.push(Piece {
                            range: start..start + 1,
                            kind: PieceKind::Break,
                        });
                    } else {
                        word_start.get_or_insert(start);
                        if ch.is_whitespace() {
                            flush(&mut pieces, &mut word_start, start + ch.len_utf8());
                        }
                    }
                }
            }
            byte = end;
        }
        flush(&mut pieces, &mut word_start, byte);
        Self {
            inlines: InlineIndex::new(document, runs),
            pieces,
        }
    }

    /// Returns occurrences in source order, including zero-byte images.
    pub fn pieces(&self) -> &[Piece] {
        &self.pieces
    }

    /// Returns semantic text bytes, including image alt text and hard breaks.
    ///
    /// A zero result does not imply that the flow has no image occurrences.
    pub fn text_len(&self) -> usize {
        self.inlines.len()
    }

    /// Materializes only a requested semantic range through the shared index.
    ///
    /// Text piece ranges preserve style/link transitions and source-backed text.
    /// Normalized literal fragments copy only their requested portion. Use the
    /// original immutable document/runs and UTF-8 boundaries.
    pub fn slice(&self, document: &Document, runs: &[Inline], range: Range<usize>) -> Vec<Inline> {
        self.inlines.slice(document, runs, range)
    }
}

fn flush(pieces: &mut Vec<Piece>, start: &mut Option<usize>, end: usize) {
    if let Some(start) = start.take() {
        pieces.push(Piece {
            range: start..end,
            kind: PieceKind::Text,
        });
    }
}

#[cfg(test)]
mod tests;
