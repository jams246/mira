use std::time::{Duration, Instant};

use gpui::{WindowTextSystem, px};

use super::*;
use crate::{appearance::Palette, render_text};

pub(super) struct Input {
    pub index: Arc<InlineIndex>,
    pub config: Config,
    pub cursor: usize,
    pub context_start: usize,
    pub row: usize,
    pub target: usize,
    pub intrinsic: IntrinsicWidth,
    pub natural_width: f32,
}

pub(super) struct Batch {
    pub chunks: Vec<Chunk>,
    pub cursor: usize,
    pub context_start: usize,
    pub rows: usize,
    pub len: usize,
    pub intrinsic: IntrinsicWidth,
    pub natural_width: f32,
    pub retain_layouts: bool,
}

pub(super) fn batch(
    document: &Document,
    source: Source,
    input: Input,
    system: &WindowTextSystem,
) -> Result<Batch, String> {
    let runs = source.runs(document);
    let index = input.index;
    let range = source.bounds(index.len())?;
    if index.floor_boundary(document, runs, range.start) != range.start
        || index.floor_boundary(document, runs, range.end) != range.end
    {
        return Err("Native text span splits UTF-8.".into());
    }
    let mut result = Batch {
        len: range.end,
        chunks: Vec::new(),
        cursor: input.cursor,
        context_start: input.context_start,
        rows: input.row,
        intrinsic: input.intrinsic,
        natural_width: input.natural_width,
        retain_layouts: false,
    };
    let start = Instant::now();
    while result.cursor < result.len && result.rows < input.target {
        let mut end = index.floor_boundary(document, runs, (result.cursor + 1024).min(result.len));
        let (end, boundaries) = loop {
            let slice = index.slice(document, runs, result.context_start..end);
            let prepared =
                render_text::prepare(document, &slice, input.config.kind, Palette::light());
            let native_start = Instant::now();
            let native_timing = Timing::start();
            let lines = system
                .shape_text(
                    prepared.text.into(),
                    px(input.config.font_size),
                    &prepared.runs,
                    Some(px(input.config.width)),
                    None,
                )
                .map_err(|error| error.to_string())?;
            native_timing.mark("text_native_shape");
            result.retain_layouts |= native_start.elapsed() >= Duration::from_millis(4);
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
            let natural_width = lines
                .iter()
                .map(|line| f32::from(line.unwrapped_layout.width))
                .fold(0., f32::max);
            result.natural_width = result.natural_width.max(natural_width);
            if lines.iter().any(|line| !line.wrap_boundaries().is_empty())
                || natural_width > input.config.width
            {
                result.intrinsic = IntrinsicWidth::Full;
            }
            if end == result.len
                || boundaries
                    .last()
                    .is_some_and(|byte| *byte > result.cursor - result.context_start)
            {
                break (end, boundaries);
            }
            // Skip unbreakable source instead of repeatedly shaping its growing prefixes.
            let word_end = index.word_end(document, runs, end..result.len);
            end = index.floor_boundary(
                document,
                runs,
                word_end.saturating_add(1024).min(result.len),
            );
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
            layouts: None,
        });
        result.context_start += boundaries.iter().rev().nth(1).copied().unwrap_or(0);
        result.cursor = committed;
        result.rows += rows;
        if final_chunk && result.intrinsic != IntrinsicWidth::Full {
            result.intrinsic = IntrinsicWidth::Exact(result.natural_width);
        }
        // Background batches amortize frame handoffs without enlarging native shaping chunks.
        if start.elapsed() >= Duration::from_millis(16) || result.chunks.len() == 64 {
            break;
        }
    }
    Ok(result)
}
