use std::{cell::RefCell, ops::Range, rc::Rc};

use mira::document::{BlockKind, Document, Inline};

pub(crate) type SelectionHandle = Rc<RefCell<Selection>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Position {
    pub block: usize,
    pub byte: usize,
}

#[derive(Default)]
pub(crate) struct Selection {
    anchor: Option<Position>,
    focus: Option<Position>,
    dragging: bool,
    link_hit: bool,
    drag_origin: Option<Position>,
}

impl Selection {
    pub(crate) fn start(&mut self, position: Position, link_hit: bool) {
        self.anchor = Some(position);
        self.focus = Some(position);
        self.dragging = true;
        self.link_hit = link_hit;
        self.drag_origin = Some(position);
    }

    pub(crate) fn start_text(&mut self, position: Position, origin: Position, link_hit: bool) {
        self.start(position, link_hit);
        self.drag_origin = Some(origin);
    }

    pub(crate) fn extend(&mut self, position: Position) -> bool {
        if !self.dragging || self.focus == Some(position) {
            return false;
        }
        self.focus = Some(position);
        true
    }

    pub(crate) fn finish(&mut self) {
        self.dragging = false;
    }

    pub(crate) fn dragging_from(&self, position: Position) -> bool {
        self.dragging && self.drag_origin == Some(position)
    }

    fn bounds(&self) -> Option<(Position, Position)> {
        let anchor = self.anchor?;
        let focus = self.focus?;
        (anchor != focus).then_some((anchor.min(focus), anchor.max(focus)))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.bounds().is_none()
    }

    pub(crate) fn link_anchor(&self) -> Option<Position> {
        self.anchor.filter(|_| self.link_hit && self.is_empty())
    }

    pub(crate) fn range_for(&self, block: usize, len: usize) -> Option<Range<usize>> {
        let (start, end) = self.bounds()?;
        if block < start.block || block > end.block {
            return None;
        }
        let first = if block == start.block {
            start.byte.min(len)
        } else {
            0
        };
        let last = if block == end.block {
            end.byte.min(len)
        } else {
            len
        };
        (first < last).then_some(first..last)
    }

    pub(crate) fn range_at(&self, position: Position, len: usize) -> Option<Range<usize>> {
        let range = self.range_for(position.block, position.byte + len)?;
        let first = range.start.max(position.byte);
        (first < range.end).then(|| first - position.byte..range.end - position.byte)
    }

    pub(crate) fn select_all(&mut self, document: &Document) {
        let Some(last) = document.blocks.len().checked_sub(1) else {
            return;
        };
        self.anchor = Some(Position { block: 0, byte: 0 });
        self.focus = Some(Position {
            block: last,
            byte: block_text(document, last).len(),
        });
        self.dragging = false;
    }

    pub(crate) fn text(&self, document: &Document) -> String {
        let Some((start, end)) = self.bounds() else {
            return String::new();
        };
        let mut output = String::new();
        for index in start.block..=end.block.min(document.blocks.len().saturating_sub(1)) {
            let text = block_text(document, index);
            if index > start.block {
                output.push_str("\n\n");
            }
            if let Some(range) = self.range_for(index, text.len()) {
                let start = boundary(&text, range.start);
                let end = boundary(&text, range.end);
                output.push_str(&text[start..end]);
            }
        }
        output
    }
}

pub(crate) fn boundary(text: &str, byte: usize) -> usize {
    let mut byte = byte.min(text.len());
    while !text.is_char_boundary(byte) {
        byte -= 1;
    }
    byte
}

fn block_text(document: &Document, index: usize) -> String {
    let Some(block) = document.blocks.get(index) else {
        return String::new();
    };
    let plain = |runs: &[Inline]| -> String {
        runs.iter()
            .map(|run| run.text.as_str(&document.source))
            .collect()
    };
    match &block.kind {
        BlockKind::Paragraph(runs)
        | BlockKind::Heading { runs, .. }
        | BlockKind::Code { runs, .. } => plain(runs),
        BlockKind::Image { alt, .. } => plain(alt),
        BlockKind::TableRow { cells, .. } => cells
            .iter()
            .map(|cell| plain(cell))
            .collect::<Vec<_>>()
            .join("\t"),
        BlockKind::Rule => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mira::markdown;

    #[test]
    fn backwards_selection_copies_unmounted_blocks_and_unicode() {
        let document = markdown::parse("Alpha 東京\n\nMiddle **bold**\n\nOmega".into());
        let mut selection = Selection::default();
        selection.start(Position { block: 2, byte: 3 }, true);
        selection.extend(Position { block: 0, byte: 6 });
        assert_eq!(selection.text(&document), "東京\n\nMiddle bold\n\nOme");
        assert_eq!(selection.range_for(1, 11), Some(0..11));
    }

    #[test]
    fn select_all_copies_plain_text_without_markdown_syntax() {
        let document = markdown::parse("# Heading\n\nA *styled* sentence.".into());
        let mut selection = Selection::default();
        selection.select_all(&document);
        assert_eq!(selection.text(&document), "Heading\n\nA styled sentence.");
    }

    #[test]
    fn selection_stops_after_mouse_up_and_boundaries_remain_valid() {
        let mut selection = Selection::default();
        selection.start(Position { block: 0, byte: 0 }, true);
        selection.finish();
        assert!(!selection.extend(Position { block: 1, byte: 3 }));
        assert!(selection.is_empty());
        assert_eq!(boundary("a😀b", 3), 1);
    }

    #[test]
    fn cell_selection_offsets_intersect_without_repeating_prefixes() {
        let mut selection = Selection::default();
        selection.start(Position { block: 2, byte: 6 }, true);
        selection.extend(Position { block: 2, byte: 14 });
        assert_eq!(
            selection.range_at(Position { block: 2, byte: 4 }, 5),
            Some(2..5)
        );
        assert_eq!(
            selection.range_at(Position { block: 2, byte: 10 }, 4),
            Some(0..4)
        );
        assert_eq!(selection.range_at(Position { block: 2, byte: 0 }, 4), None);
        assert_eq!(selection.range_at(Position { block: 2, byte: 16 }, 4), None);
    }

    #[test]
    fn adjacent_text_fragments_capture_only_the_originating_fragment() {
        let mut selection = Selection::default();
        let origin = Position { block: 1, byte: 10 };
        selection.start_text(Position { byte: 12, ..origin }, origin, true);
        assert!(selection.dragging_from(origin));
        assert!(!selection.dragging_from(Position { byte: 0, ..origin }));
    }
}
