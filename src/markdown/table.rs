use std::ops::Range;

use crate::document::{BlockKind, Inline};

pub(super) struct TableState {
    index: usize,
    header: bool,
    range: Range<usize>,
    cells: Vec<Vec<Inline>>,
    pub cell: Option<Vec<Inline>>,
    row: bool,
}

impl TableState {
    pub(super) fn new(index: usize) -> Self {
        Self {
            index,
            header: false,
            range: 0..0,
            cells: Vec::new(),
            cell: None,
            row: false,
        }
    }

    pub(super) fn begin_row(&mut self, header: bool, range: Range<usize>) {
        self.header = header;
        self.range = range;
        self.cells.clear();
        self.row = true;
    }

    pub(super) fn begin_cell(&mut self) {
        self.cell = Some(Vec::new());
    }

    pub(super) fn finish_cell(&mut self) {
        if let Some(cell) = self.cell.take() {
            self.cells.push(cell);
        }
    }

    pub(super) fn finish_row(&mut self) -> Option<(BlockKind, Range<usize>)> {
        if !self.row {
            return None;
        }
        self.finish_cell();
        self.row = false;
        Some((
            BlockKind::TableRow {
                table: self.index,
                header: self.header,
                cells: std::mem::take(&mut self.cells),
            },
            self.range.clone(),
        ))
    }
}
