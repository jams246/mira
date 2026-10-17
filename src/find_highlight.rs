use std::{cell::RefCell, ops::Range, rc::Rc, sync::Arc};

use gpui::TextRun;
use mira::search::SearchMatch;

use crate::{appearance::Palette, selection::Position, text_map::TextMap};

pub(crate) type HighlightHandle = Rc<RefCell<Highlights>>;

#[derive(Default)]
pub(crate) struct Highlights {
    pub matches: Arc<Vec<SearchMatch>>,
    pub active: Option<usize>,
    pub enabled: bool,
}

impl Highlights {
    pub fn ranges(
        &self,
        position: Position,
        cell: Option<usize>,
        len: usize,
    ) -> Vec<(Range<usize>, bool)> {
        if !self.enabled || len == 0 {
            return Vec::new();
        }
        let end = position.byte.saturating_add(len);
        let first = self.matches.partition_point(|item| {
            item.block < position.block
                || item.block == position.block && item.range.end <= position.byte
        });
        self.matches[first..]
            .iter()
            .enumerate()
            .take_while(|(_, item)| item.block == position.block && item.range.start < end)
            .filter(|(_, item)| item.cell == cell)
            .map(|(index, item)| {
                (
                    item.range.start.max(position.byte) - position.byte
                        ..item.range.end.min(end) - position.byte,
                    self.active == Some(first + index),
                )
            })
            .collect()
    }
}

pub(crate) fn highlight(
    runs: Vec<TextRun>,
    position: Position,
    cell: Option<usize>,
    len: usize,
    mapping: &TextMap,
    highlights: &HighlightHandle,
    palette: Palette,
) -> Vec<TextRun> {
    let ranges = highlights.borrow().ranges(position, cell, len);
    if ranges.is_empty() {
        return runs;
    }
    let mut ranges = ranges
        .into_iter()
        .map(|(range, active)| {
            (
                mapping.to_display(range.start)..mapping.to_display(range.end),
                active,
            )
        })
        .peekable();
    let mut output = Vec::with_capacity(runs.len() + ranges.len() * 2);
    let mut start = 0;
    for run in runs {
        let end = start + run.len;
        let mut byte = start;
        while byte < end {
            while ranges.peek().is_some_and(|(range, _)| range.end <= byte) {
                ranges.next();
            }
            let mut part = run.clone();
            let next = match ranges.peek() {
                Some((range, active)) if range.contains(&byte) => {
                    part.background_color = Some(if *active {
                        palette.find_active
                    } else {
                        palette.find
                    });
                    part.color = palette.text;
                    range.end.min(end)
                }
                Some((range, _)) => range.start.min(end),
                None => end,
            };
            part.len = next - byte;
            output.push(part);
            byte = next;
        }
        start = end;
    }
    output
}

#[cfg(test)]
mod tests;
