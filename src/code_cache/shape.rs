use std::time::{Duration, Instant};

use gpui::{ShapedLine, TextRun, WindowTextSystem, font, px, rgb};
use mira::document::BlockKind;

use super::{Chunk, Entry, Source};
use crate::{file::LoadedDocument, text_map::TextMap, timing::Timing};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Progress {
    pub font_size: f32,
    pub line: std::ops::Range<usize>,
    pub width: f32,
    pub complete: bool,
    pub cursor: usize,
    pub context_start: usize,
    pub context_column: usize,
}

impl From<&Entry> for Progress {
    fn from(state: &Entry) -> Self {
        Self {
            font_size: state.font_size,
            line: state.line.clone(),
            width: state.width,
            complete: state.complete,
            cursor: state.cursor,
            context_start: state.context_start,
            context_column: state.context_column,
        }
    }
}

pub(super) struct Step {
    pub chunk: Chunk,
    pub expensive: bool,
}

pub(crate) fn column_after(mut column: usize, text: &str) -> usize {
    for ch in text.chars() {
        column = match ch {
            '\t' => column + 4 - column % 4,
            '\n' => 0,
            _ => column + 1,
        };
    }
    column
}

pub(crate) fn prepare(loaded: &LoadedDocument, source: Source, chunk: &Chunk) -> (String, TextMap) {
    let document = &loaded.document;
    let BlockKind::Code { runs, .. } = &document.blocks[source.block].kind else {
        return TextMap::expand("");
    };
    let Some(index) = loaded.code.get(&source.block) else {
        return TextMap::expand("");
    };
    let text = index
        .slice(document, runs, chunk.window.clone())
        .iter()
        .map(|run| run.text.as_str(&document.source))
        .collect::<String>();
    TextMap::expand_from(&text, chunk.column)
}

pub(crate) fn native(system: &WindowTextSystem, text: String, font_size: f32) -> ShapedLine {
    let run = TextRun {
        len: text.len(),
        font: font("Consolas"),
        color: rgb(0x242424).into(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    system.shape_line(text.into(), px(font_size), &[run], None)
}

pub(super) fn step(
    loaded: &LoadedDocument,
    source: Source,
    state: &mut Progress,
    system: &WindowTextSystem,
) -> Result<Step, String> {
    let layout = loaded
        .code
        .get(&source.block)
        .ok_or("Missing code index.")?;
    let document = &loaded.document;
    let BlockKind::Code { runs, .. } = &document.blocks[source.block].kind else {
        return Err("Missing code block.".into());
    };
    // Keep bounded lines identical to native whole-line glyphs and bidi caret positions.
    let mut budget = if state.line.len() <= 32 * 1024 {
        state.line.len()
    } else {
        896
    };
    let mut expensive = false;
    loop {
        let end =
            layout.floor_boundary(document, runs, (state.cursor + budget).min(state.line.end));
        let raw = layout
            .slice(document, runs, state.context_start..end)
            .iter()
            .map(|run| run.text.as_str(&document.source))
            .collect::<String>();
        let (text, mapping) = TextMap::expand_from(&raw, state.context_column);
        let timing = Timing::start();
        let native_start = Instant::now();
        let line = native(system, text, state.font_size);
        expensive |= native_start.elapsed() >= Duration::from_millis(4);
        timing.mark("code_native_shape");
        let mut boundaries = line
            .runs
            .iter()
            .flat_map(|run| &run.glyphs)
            .map(|glyph| state.context_start + mapping.to_source(glyph.index))
            .filter(|byte| *byte > state.cursor)
            .collect::<Vec<_>>();
        boundaries.sort_unstable();
        boundaries.dedup();
        let commit = if end == state.line.end {
            Some(end)
        } else {
            boundaries
                .iter()
                .copied()
                .rfind(|byte| *byte <= state.cursor + 768)
                .or_else(|| boundaries.first().copied())
        };
        if let Some(commit) = commit.filter(|commit| *commit > state.cursor) {
            let local_start = mapping.to_display(state.cursor - state.context_start);
            let local_end = mapping.to_display(commit - state.context_start);
            let whole_line = state.cursor == state.line.start
                && state.context_start == state.line.start
                && end == state.line.end;
            let local_x = if whole_line {
                0.
            } else {
                f32::from(line.x_for_index(local_start))
            };
            let width = if whole_line {
                f32::from(line.width)
            } else {
                (f32::from(line.x_for_index(local_end)) - local_x).max(0.)
            };
            let chunk = Chunk {
                window: state.context_start..end,
                core: state.cursor..commit,
                column: state.context_column,
                x: state.width,
                local_x,
                width,
                layouts: None,
            };
            state.width += width;
            let context = layout.floor_boundary(
                document,
                runs,
                commit.saturating_sub(128).max(state.line.start),
            );
            state.context_column =
                column_after(state.context_column, &raw[..context - state.context_start]);
            state.context_start = context;
            state.cursor = commit;
            state.complete = commit == state.line.end;
            return Ok(Step { chunk, expensive });
        }
        if end == state.line.end {
            return Err("Cannot advance native code layout.".into());
        }
        let word_end = layout.word_end(document, runs, end..state.line.end);
        budget = word_end.saturating_add(896).min(state.line.end) - state.cursor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_columns_preserve_unicode_scalar_tab_phase_and_physical_resets() {
        assert_eq!(column_after(1, "\t東京é\t"), 8);
        assert_eq!(column_after(3, "é\n\t東京"), 6);
        assert_eq!(column_after(4, ""), 4);
    }

    #[test]
    fn bounded_source_windows_expand_like_the_original_complete_code_line() {
        let raw = "é\t東京 😀\tabc ".repeat(2048);
        let document = mira::markdown::parse(format!("```\n{raw}\n```"));
        let indexes = mira::code_layout::index(&document);
        let index = &indexes[&0];
        let BlockKind::Code { runs, .. } = &document.blocks[0].kind else {
            panic!("code")
        };
        let (display, full_map) = TextMap::expand(&raw);
        for requested in [0, 3, 7, 16, raw.len() / 2, raw.len() - 64] {
            let start = index.floor_boundary(&document, runs, requested);
            let end = index.floor_boundary(&document, runs, start + 48);
            let slice = index.slice(&document, runs, start..end);
            let text = slice
                .iter()
                .map(|run| run.text.as_str(&document.source))
                .collect::<String>();
            let column = column_after(0, &raw[..start]);
            let (actual, map) = TextMap::expand_from(&text, column);
            assert_eq!(
                actual,
                display[full_map.to_display(start)..full_map.to_display(end)]
            );
            for byte in text
                .char_indices()
                .map(|(byte, _)| byte)
                .chain([text.len()])
            {
                assert_eq!(map.to_source(map.to_display(byte)), byte);
            }
            assert_eq!(text, raw[start..end]);
            assert!(text.len() <= 48);
        }
    }
}
