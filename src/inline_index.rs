//! Source-backed slicing of semantic inline text without scanning preceding runs.

use std::ops::Range;

use crate::document::{Document, Inline, Text};

/// Cumulative UTF-8 byte offsets for one immutable inline sequence.
pub struct InlineIndex {
    starts: Vec<usize>,
}

impl InlineIndex {
    /// Indexes displayed inline text lengths without copying source text.
    pub fn new(document: &Document, runs: &[Inline]) -> Self {
        let mut starts = Vec::with_capacity(runs.len() + 1);
        let mut len = 0;
        for run in runs {
            starts.push(len);
            len += run.text.as_str(&document.source).len();
        }
        starts.push(len);
        Self { starts }
    }

    /// Total displayed text length in UTF-8 bytes.
    pub fn len(&self) -> usize {
        self.starts.last().copied().unwrap_or_default()
    }

    /// Whether the indexed sequence contains no displayed text bytes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clamps a byte offset to the preceding semantic UTF-8 character boundary.
    ///
    /// Use the original immutable document and run sequence indexed by this value.
    pub fn floor_boundary(&self, document: &Document, runs: &[Inline], byte: usize) -> usize {
        let byte = byte.min(self.len());
        let index = self
            .starts
            .partition_point(|start| *start <= byte)
            .saturating_sub(1);
        let Some(run) = runs.get(index) else {
            return byte;
        };
        let start = self.starts[index];
        let text = run.text.as_str(&document.source);
        let mut local = byte - start;
        while !text.is_char_boundary(local) {
            local -= 1;
        }
        start + local
    }

    /// Borrows source text and copies only normalized text within a visible range.
    ///
    /// Use the original immutable run sequence and semantic UTF-8 boundaries.
    /// Invalid character-boundary slices are omitted rather than splitting UTF-8.
    pub fn slice(&self, document: &Document, runs: &[Inline], range: Range<usize>) -> Vec<Inline> {
        let first = self
            .starts
            .partition_point(|start| *start <= range.start)
            .saturating_sub(1);
        let mut output = Vec::new();
        for (index, run) in runs.iter().enumerate().skip(first) {
            let start = self.starts[index];
            if start >= range.end {
                break;
            }
            let text = run.text.as_str(&document.source);
            let a = range.start.saturating_sub(start).min(text.len());
            let b = range.end.saturating_sub(start).min(text.len());
            if a >= b {
                continue;
            }
            let Some(slice) = text.get(a..b) else {
                continue;
            };
            output.push(Inline {
                source_range: run.source_range.clone(),
                text: match &run.text {
                    Text::Source(source) => Text::Source(source.start + a..source.start + b),
                    Text::Literal(_) => Text::Literal(slice.into()),
                },
                style: run.style,
                link: run.link,
                image: run.image,
            });
        }
        output
    }
}

#[cfg(test)]
mod tests;
