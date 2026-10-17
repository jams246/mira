use crate::document::{Block, BlockContext, BlockKind, ContextKind, Inline, Table};

use super::{inline::InlineState, table::TableState};

#[derive(Default)]
pub(super) struct State {
    pub blocks: Vec<Block>,
    pub contexts: Vec<BlockContext>,
    pub targets: Vec<String>,
    pub tables: Vec<Table>,
    pub lists: Vec<Option<u64>>,
    pub inline: InlineState,
    pub table: Option<TableState>,
    ancestry: Vec<usize>,
    items: Vec<(usize, bool)>,
    leaf: Option<BlockKind>,
}

impl State {
    pub(super) fn begin(&mut self, kind: BlockKind) {
        self.finish();
        self.leaf = Some(kind);
    }

    pub(super) fn push_context(&mut self, kind: ContextKind) {
        let index = self.contexts.len();
        self.contexts.push(BlockContext {
            parent: self.ancestry.last().copied(),
            kind,
        });
        self.ancestry.push(index);
    }

    pub(super) fn pop_context(&mut self) {
        self.ancestry.pop();
    }

    pub(super) fn begin_item(&mut self) {
        self.finish();
        let number = self.lists.last().copied().flatten();
        if let Some(Some(next)) = self.lists.last_mut() {
            *next = next.saturating_add(1);
        }
        self.push_context(ContextKind::ListItem {
            number,
            checked: None,
        });
        if let Some(&index) = self.ancestry.last() {
            self.items.push((index, true));
        }
    }

    pub(super) fn finish_pending_item(&mut self) {
        self.finish();
        if self.items.last().is_some_and(|(_, pending)| *pending) {
            self.emit(BlockKind::Paragraph(Vec::new()));
        }
    }

    pub(super) fn end_item(&mut self) {
        self.finish_pending_item();
        self.items.pop();
        self.pop_context();
    }

    pub(super) fn task(&mut self, checked: bool) {
        if let Some(&(index, _)) = self.items.last()
            && let ContextKind::ListItem { checked: state, .. } = &mut self.contexts[index].kind
        {
            *state = Some(checked);
        }
    }

    pub(super) fn push_run(&mut self, run: Inline) {
        if let Some(cell) = self.table.as_mut().and_then(|table| table.cell.as_mut()) {
            cell.push(run);
            return;
        }
        let kind = self
            .leaf
            .get_or_insert_with(|| BlockKind::Paragraph(Vec::with_capacity(1)));
        match kind {
            BlockKind::Paragraph(runs)
            | BlockKind::Heading { runs, .. }
            | BlockKind::Code { runs, .. } => runs.push(run),
            _ => {}
        }
    }

    pub(super) fn finish(&mut self) {
        if let Some(mut kind) = self.leaf.take() {
            if let BlockKind::Paragraph(runs) = &mut kind
                && runs.len() == 1
                && let Some(target) = runs[0].image
            {
                kind = BlockKind::Image {
                    target,
                    link: runs[0].link,
                    alt: std::mem::take(runs),
                };
            }
            self.emit(kind);
        }
    }

    pub(super) fn emit(&mut self, kind: BlockKind) {
        let marker = self
            .items
            .last_mut()
            .is_some_and(|(_, pending)| std::mem::take(pending));
        self.blocks.push(Block {
            context: self.ancestry.last().copied(),
            marker,
            kind,
        });
    }

    pub(super) fn finish_row(&mut self) {
        if let Some(kind) = self.table.as_mut().and_then(TableState::finish_row) {
            self.emit(kind);
        }
    }
}
