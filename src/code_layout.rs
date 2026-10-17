//! Source-backed physical-line and column indexes for oversized code blocks.

use std::{collections::HashMap, ops::Range};

use crate::document::{BlockKind, Document, Inline};
use crate::inline_index::InlineIndex;

#[cfg(test)]
use crate::document::Text;

/// Small code blocks retain ordinary native text layout.
pub const MIN_INDEXED_BYTES: usize = 2048;

/// One physical code line, excluding its terminating newline.
#[derive(Clone, Debug)]
pub struct CodeLine {
    /// Semantic display-text byte range within the original code block.
    pub bytes: Range<usize>,
    /// Columns after four-column TAB expansion.
    pub columns: usize,
    /// Whether fixed-width native ASCII column slicing is valid.
    pub ascii: bool,
}

struct LineIndex {
    start: usize,
    columns: usize,
    ascii: bool,
}

struct Tab {
    source: usize,
    display_end: usize,
}

/// Layout-independent indexes built once off the UI thread without copying code text.
pub struct CodeLayout {
    runs: InlineIndex,
    lines: Vec<LineIndex>,
    tabs: Vec<Tab>,
    len: usize,
    max_columns: usize,
}

/// Indexes only large code blocks, keyed by the unchanged semantic block index.
pub fn index(document: &Document) -> HashMap<usize, CodeLayout> {
    document
        .blocks
        .iter()
        .enumerate()
        .filter_map(|(index, block)| {
            let BlockKind::Code { runs, .. } = &block.kind else {
                return None;
            };
            let len = runs
                .iter()
                .map(|run| run.text.as_str(&document.source).len())
                .sum::<usize>();
            (len >= MIN_INDEXED_BYTES).then(|| (index, CodeLayout::build(document, runs)))
        })
        .collect()
}

impl CodeLayout {
    fn build(document: &Document, runs: &[Inline]) -> Self {
        let mut result = Self {
            runs: InlineIndex::new(document, runs),
            lines: Vec::new(),
            tabs: Vec::new(),
            len: 0,
            max_columns: 0,
        };
        let (mut start, mut columns, mut extra, mut ascii) = (0, 0, 0, true);
        for run in runs {
            for ch in run.text.as_str(&document.source).chars() {
                match ch {
                    '\n' => {
                        result.push_line(start, columns, ascii);
                        start = result.len + 1;
                        columns = 0;
                        ascii = true;
                    }
                    '\t' => {
                        let spaces = 4 - columns % 4;
                        extra += spaces - 1;
                        result.tabs.push(Tab {
                            source: result.len,
                            display_end: result.len + extra + 1,
                        });
                        columns += spaces;
                    }
                    _ => {
                        columns += 1;
                        ascii &= matches!(ch, ' '..='~');
                    }
                }
                result.len += ch.len_utf8();
            }
        }
        result.push_line(start, columns, ascii);
        result
    }

    fn push_line(&mut self, start: usize, columns: usize, ascii: bool) {
        self.max_columns = self
            .max_columns
            .max(if ascii { columns } else { columns * 4 });
        self.lines.push(LineIndex {
            start,
            columns,
            ascii,
        });
    }

    /// Number of physical lines, including a final blank line after a newline.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Fixed-width extent, with conservative extra width for native Unicode fallback.
    pub fn max_columns(&self) -> usize {
        self.max_columns
    }

    /// Retrieves physical-line metadata without allocating.
    pub fn line(&self, index: usize) -> Option<CodeLine> {
        let line = self.lines.get(index)?;
        Some(CodeLine {
            bytes: line.start
                ..self
                    .lines
                    .get(index + 1)
                    .map_or(self.len, |next| next.start.saturating_sub(1)),
            columns: line.columns,
            ascii: line.ascii,
        })
    }

    fn extra(&self, byte: usize) -> usize {
        let count = self.tabs.partition_point(|tab| tab.source < byte);
        count.checked_sub(1).map_or(0, |index| {
            let tab = &self.tabs[index];
            tab.display_end - tab.source - 1
        })
    }

    /// Maps an ASCII line's semantic byte boundary to its TAB-expanded column.
    pub fn column_for_byte(&self, line: &CodeLine, byte: usize) -> usize {
        let byte = byte.clamp(line.bytes.start, line.bytes.end);
        byte - line.bytes.start + self.extra(byte) - self.extra(line.bytes.start)
    }

    /// Maps an ASCII line column to a source byte, retaining the TAB if inside it.
    pub fn byte_for_column(&self, line: &CodeLine, column: usize) -> usize {
        let display = line.bytes.start + self.extra(line.bytes.start) + column.min(line.columns);
        let completed = self.tabs.partition_point(|tab| tab.display_end <= display);
        let extra = completed.checked_sub(1).map_or(0, |index| {
            let tab = &self.tabs[index];
            tab.display_end - tab.source - 1
        });
        let byte = self
            .tabs
            .get(completed)
            .filter(|tab| display >= tab.source + extra)
            .map_or(display - extra, |tab| tab.source);
        byte.clamp(line.bytes.start, line.bytes.end)
    }

    /// Borrows source ranges and copies only bounded normalized text in a visible slice.
    pub fn slice(&self, document: &Document, runs: &[Inline], range: Range<usize>) -> Vec<Inline> {
        self.runs.slice(document, runs, range)
    }

    /// Clamps a semantic code byte to the preceding UTF-8 character boundary.
    pub fn floor_boundary(&self, document: &Document, runs: &[Inline], byte: usize) -> usize {
        self.runs.floor_boundary(document, runs, byte)
    }

    /// Scans bounded source windows through the next whitespace or range end.
    pub fn word_end(&self, document: &Document, runs: &[Inline], range: Range<usize>) -> usize {
        self.runs.word_end(document, runs, range)
    }
}

#[cfg(test)]
mod tests;

#[test]
fn two_kib_unicode_code_is_indexed_before_native_viewport_layout() {
    let raw = "é".repeat(1024);
    let document = crate::markdown::parse(format!("```\n{raw}\n```"));
    let layouts = index(&document);
    let line = layouts[&0].line(0).expect("Indexed Unicode line");
    assert_eq!(line.bytes.len(), 2048);
    assert!(!line.ascii);
    let smaller = crate::markdown::parse(format!("```\n{}\n```", "é".repeat(1023)));
    assert!(index(&smaller).is_empty());
}

#[test]
fn code_word_scan_skips_a_complete_combining_sequence_and_preserves_space() {
    let raw = format!("a{} after", "\u{301}".repeat(16_384));
    let document = crate::markdown::parse(format!("```\n{raw}\n```"));
    let layouts = index(&document);
    let BlockKind::Code { runs, .. } = &document.blocks[0].kind else {
        panic!("code")
    };
    assert_eq!(layouts[&0].word_end(&document, runs, 895..raw.len()), 32770);
    assert_eq!(layouts[&0].word_end(&document, runs, 895..896), 895);
    assert_eq!(
        layouts[&0].word_end(&document, runs, raw.len() - 2..raw.len()),
        raw.len()
    );
}
