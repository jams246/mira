use std::time::{Duration, Instant};

use gpui::{WindowTextSystem, px};

use super::*;
use crate::render_text;

pub(super) struct Input {
    pub index: Arc<InlineIndex>,
    pub config: Config,
    pub cursor: usize,
    pub context_start: usize,
    pub row: usize,
    pub target: usize,
}

pub(super) struct Batch {
    pub index: Arc<InlineIndex>,
    pub chunks: Vec<Chunk>,
    pub cursor: usize,
    pub context_start: usize,
    pub rows: usize,
    pub len: usize,
}

pub(super) fn batch(
    document: &Document,
    source: Source,
    input: Input,
    system: &WindowTextSystem,
) -> Result<Batch, String> {
    let runs = source.runs(document);
    let index = input.index;
    let mut result = Batch {
        len: index.len(),
        index,
        chunks: Vec::new(),
        cursor: input.cursor,
        context_start: input.context_start,
        rows: input.row,
    };
    let start = Instant::now();
    while result.cursor < result.len && result.rows < input.target {
        let mut budget = 1024;
        let (end, boundaries) = loop {
            let end = result
                .index
                .floor_boundary(document, runs, result.cursor + budget);
            let slice = result
                .index
                .slice(document, runs, result.context_start..end);
            let prepared = render_text::prepare(document, &slice, input.config.kind);
            let lines = system
                .shape_text(
                    prepared.text.into(),
                    px(input.config.font_size),
                    &prepared.runs,
                    Some(px(input.config.width)),
                    None,
                )
                .map_err(|error| error.to_string())?;
            let mut boundaries = Vec::new();
            let mut base = 0;
            for line in &lines {
                for boundary in line.wrap_boundaries() {
                    boundaries.push(
                        base + line.unwrapped_layout.runs[boundary.run_ix].glyphs
                            [boundary.glyph_ix]
                            .index,
                    );
                }
                base += line.len() + 1;
                if base <= end - result.context_start {
                    boundaries.push(base);
                }
            }
            if end == result.len
                || boundaries
                    .last()
                    .is_some_and(|byte| *byte > result.cursor - result.context_start)
            {
                break (end, boundaries);
            }
            // One indivisible cluster may require a larger native call to preserve its glyphs.
            budget = budget.saturating_mul(2).min(result.len - result.cursor);
        };
        let final_chunk = end == result.len;
        let committed = if final_chunk {
            end
        } else {
            result.context_start + *boundaries.last().ok_or("Cannot wrap text.")?
        };
        if committed <= result.cursor {
            return Err("Cannot advance native text layout.".into());
        }
        let skip_rows =
            boundaries.partition_point(|byte| *byte <= result.cursor - result.context_start);
        let rows = boundaries.len() + usize::from(final_chunk) - skip_rows;
        result.chunks.push(Chunk {
            range: result.context_start..end,
            row: result.rows,
            rows,
            skip_rows,
        });
        result.context_start += boundaries.iter().rev().nth(1).copied().unwrap_or(0);
        result.cursor = committed;
        result.rows += rows;
        if start.elapsed() >= Duration::from_millis(4) || result.chunks.len() == 64 {
            break;
        }
    }
    Ok(result)
}
