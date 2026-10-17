use std::ops::Range;

use mira::document::{BlockKind, Document, Inline};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Source {
    pub block: usize,
    pub cell: Option<usize>,
    pub span: Option<(usize, usize)>,
}

impl Source {
    pub(crate) fn whole(block: usize, cell: Option<usize>) -> Self {
        Self {
            block,
            cell,
            span: None,
        }
    }

    pub(crate) fn runs(self, document: &Document) -> &[Inline] {
        match &document.blocks[self.block].kind {
            BlockKind::Paragraph(runs)
            | BlockKind::RawHtml(runs)
            | BlockKind::Heading { runs, .. }
            | BlockKind::Image { alt: runs, .. } => runs,
            BlockKind::TableRow { cells, .. } => self
                .cell
                .and_then(|column| cells.get(column))
                .map_or(&[], Vec::as_slice),
            _ => &[],
        }
    }

    pub(super) fn bounds(self, len: usize) -> Result<Range<usize>, String> {
        match self.span {
            Some((start, end)) if start <= end && end <= len => Ok(start..end),
            Some(_) => Err("Invalid native text span.".into()),
            None => Ok(0..len),
        }
    }
}
