use std::ops::Range;

use gpui::{
    AnyElement, Bounds, IntoElement, SharedString, StyledText, TextAlign, TextLayout, TextRun,
    canvas, div, fill, point, prelude::*, px, size,
};

use crate::text_hit_test;

pub(crate) fn text(
    text: SharedString,
    mut runs: Vec<TextRun>,
    align: TextAlign,
) -> (AnyElement, TextLayout) {
    let mut backgrounds = Vec::new();
    let mut byte = 0;
    if align != TextAlign::Left {
        // GPUI's aligned run backgrounds use an unaligned clip and final endpoint.
        for run in &mut runs {
            if let Some(color) = run.background_color.take() {
                backgrounds.push((byte..byte + run.len, color));
            }
            byte += run.len;
        }
    }
    let styled = StyledText::new(text.clone()).with_runs(runs);
    let layout = styled.layout().clone();
    if backgrounds.is_empty() {
        return (styled.into_any_element(), layout);
    }
    let paint_layout = layout.clone();
    let layer = canvas(
        |_, _, _| (),
        move |_, (), window, _| {
            let bounds = paint_layout.bounds();
            let line_height = paint_layout.line_height();
            let mut byte = 0;
            let mut top = bounds.top();
            for line_text in text.split('\n') {
                let Some(line) = paint_layout.line_layout_for_index(byte) else {
                    break;
                };
                let mut row_start = 0;
                let ends = line
                    .wrap_boundaries
                    .iter()
                    .map(|boundary| {
                        line.unwrapped_layout.runs[boundary.run_ix].glyphs[boundary.glyph_ix].index
                    })
                    .chain([line_text.len()]);
                for row_end in ends {
                    let start_x = line.unwrapped_layout.x_for_index(row_start);
                    let end_x = line.unwrapped_layout.x_for_index(row_end);
                    let left = bounds.left()
                        + text_hit_test::offset(bounds.size.width, end_x - start_x, align);
                    for (range, color) in &backgrounds {
                        if let Some(range) =
                            intersection(range, &(byte + row_start..byte + row_end))
                        {
                            let x = line.unwrapped_layout.x_for_index(range.start - byte);
                            let end = line.unwrapped_layout.x_for_index(range.end - byte);
                            window.paint_quad(fill(
                                Bounds::new(
                                    point(left + x - start_x, top),
                                    size((end - x).max(px(0.)), line_height),
                                ),
                                *color,
                            ));
                        }
                    }
                    row_start = row_end;
                    top += line_height;
                }
                byte += line_text.len() + 1;
            }
        },
    )
    .absolute()
    .size_full();
    (
        div()
            .relative()
            .child(layer)
            .child(styled)
            .into_any_element(),
        layout,
    )
}

fn intersection(a: &Range<usize>, b: &Range<usize>) -> Option<Range<usize>> {
    let first = a.start.max(b.start);
    let last = a.end.min(b.end);
    (first < last).then_some(first..last)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn background_ranges_split_at_visual_line_boundaries() {
        assert_eq!(intersection(&(3..15), &(0..8)), Some(3..8));
        assert_eq!(intersection(&(3..15), &(8..20)), Some(8..15));
        assert_eq!(intersection(&(3..15), &(15..20)), None);
    }
}
