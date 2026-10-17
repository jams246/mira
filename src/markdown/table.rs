use crate::document::{BlockKind, Inline};

pub(super) struct TableState {
    index: usize,
    header: bool,
    cells: Vec<Vec<Inline>>,
    pub cell: Option<Vec<Inline>>,
    row: bool,
}

impl TableState {
    pub(super) fn new(index: usize) -> Self {
        Self {
            index,
            header: false,
            cells: Vec::new(),
            cell: None,
            row: false,
        }
    }

    pub(super) fn begin_row(&mut self, header: bool) {
        self.header = header;
        self.cells.clear();
        self.row = true;
    }

    pub(super) fn begin_cell(&mut self) {
        self.cell = Some(Vec::with_capacity(1));
    }

    pub(super) fn finish_cell(&mut self) {
        if let Some(cell) = self.cell.take() {
            self.cells.push(cell);
        }
    }

    pub(super) fn finish_row(&mut self) -> Option<BlockKind> {
        if !self.row {
            return None;
        }
        self.finish_cell();
        self.row = false;
        Some(BlockKind::TableRow {
            table: self.index,
            header: self.header,
            cells: std::mem::take(&mut self.cells),
        })
    }
}
