//! Cancellable literal search over normalized display text and semantic offsets.

mod lowercase;

use std::{
    borrow::Cow,
    ops::Range,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::document::{BlockKind, Document, Inline};

use self::lowercase::Lowercase;

/// One non-overlapping displayed-text match in document order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchMatch {
    /// Semantic block identity within the searched document.
    pub block: usize,
    /// Table cell identity, or `None` for other blocks.
    pub cell: Option<usize>,
    /// Block-global semantic UTF-8 bytes, including preceding table separators.
    pub range: Range<usize>,
}

/// Searches literal displayed text without crossing images, blocks, or cells.
///
/// Case-insensitive matching uses scalar lowercase mapping, without Unicode
/// normalization or case folding. A match cannot split a lowercase expansion.
/// Cancelled searches return no results; callers must also reject stale tasks.
pub fn search(
    document: &Document,
    query: &str,
    match_case: bool,
    cancel: &AtomicBool,
) -> Vec<SearchMatch> {
    if query.is_empty() || cancel.load(Ordering::Relaxed) {
        return Vec::new();
    }
    let query = if match_case {
        Cow::Borrowed(query)
    } else {
        Cow::Owned(query.chars().flat_map(char::to_lowercase).collect())
    };
    let mut scanner = Scanner {
        document,
        query: &query,
        match_case,
        cancel,
        matches: Vec::new(),
    };
    for (block, item) in document.blocks.iter().enumerate() {
        match &item.kind {
            BlockKind::Paragraph(runs)
            | BlockKind::RawHtml(runs)
            | BlockKind::Heading { runs, .. }
            | BlockKind::Code { runs, .. } => scanner.runs(runs, block, None, 0),
            BlockKind::TableRow { cells, .. } => {
                let mut base = 0;
                for (cell, runs) in cells.iter().enumerate() {
                    scanner.runs(runs, block, Some(cell), base);
                    base += runs
                        .iter()
                        .map(|run| run.text.as_str(&document.source).len())
                        .sum::<usize>()
                        + 1;
                }
            }
            BlockKind::Image { .. } | BlockKind::Rule => {}
        }
        if cancel.load(Ordering::Relaxed) {
            return Vec::new();
        }
    }
    scanner.matches
}

/// Copies complete parser-preserved code text without fences or language info.
pub fn code_text(document: &Document, block: usize) -> Option<String> {
    let BlockKind::Code { runs, .. } = &document.blocks.get(block)?.kind else {
        return None;
    };
    Some(
        runs.iter()
            .map(|run| run.text.as_str(&document.source))
            .collect(),
    )
}

struct Scanner<'a> {
    document: &'a Document,
    query: &'a str,
    match_case: bool,
    cancel: &'a AtomicBool,
    matches: Vec<SearchMatch>,
}

impl Scanner<'_> {
    fn runs(&mut self, runs: &[Inline], block: usize, cell: Option<usize>, base: usize) {
        let mut span = String::new();
        let mut byte = base;
        let mut start = base;
        for run in runs {
            if self.cancel.load(Ordering::Relaxed) {
                return;
            }
            let text = run.text.as_str(&self.document.source);
            if run.image.is_some() {
                self.span(std::mem::take(&mut span), block, cell, start);
                start = byte + text.len();
            } else {
                span.push_str(text);
            }
            byte += text.len();
        }
        self.span(span, block, cell, start);
    }

    fn span(&mut self, text: String, block: usize, cell: Option<usize>, base: usize) {
        if text.is_empty() || self.cancel.load(Ordering::Relaxed) {
            return;
        }
        let Some(text) = Lowercase::new(text, self.match_case, self.cancel) else {
            return;
        };
        let mut cursor = 0;
        while let Some(found) = text.as_str()[cursor..].find(self.query) {
            if self.cancel.load(Ordering::Relaxed) {
                return;
            }
            let start = cursor + found;
            let end = start + self.query.len();
            if let (Some(a), Some(b)) = (text.source_boundary(start), text.source_boundary(end)) {
                self.matches.push(SearchMatch {
                    block,
                    cell,
                    range: base + a..base + b,
                });
                cursor = end;
            } else {
                cursor = start
                    + text.as_str()[start..]
                        .chars()
                        .next()
                        .map_or(0, char::len_utf8);
            }
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod code_tests;
