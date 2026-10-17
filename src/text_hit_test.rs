use gpui::{Pixels, Point, TextAlign, TextLayout, px};

pub(crate) fn index(
    layout: &TextLayout,
    text: &str,
    point: Point<Pixels>,
    align: TextAlign,
) -> Result<usize, usize> {
    let mut point = point;
    // GPUI paints aligned glyphs but hit-tests from each line's unaligned origin.
    if align != TextAlign::Left {
        let bounds = layout.bounds();
        let line_height = layout.line_height();
        let mut top = bounds.top();
        let mut byte = 0;
        for line in text.split('\n') {
            let Some(wrapped) = layout.line_layout_for_index(byte) else {
                break;
            };
            let height = wrapped.size(line_height).height;
            if point.y < top + height {
                let row = ((point.y - top).max(px(0.)) / line_height) as usize;
                let glyph_x = |boundary: &gpui::WrapBoundary| {
                    wrapped.unwrapped_layout.runs[boundary.run_ix].glyphs[boundary.glyph_ix]
                        .position
                        .x
                };
                let start = row
                    .checked_sub(1)
                    .and_then(|index| wrapped.wrap_boundaries.get(index))
                    .map_or(px(0.), glyph_x);
                let end = wrapped
                    .wrap_boundaries
                    .get(row)
                    .map_or(wrapped.unwrapped_layout.width, glyph_x);
                point.x -= offset(bounds.size.width, end - start, align);
                break;
            }
            top += height;
            byte += line.len() + 1;
        }
    }
    layout.index_for_position(point)
}

pub(crate) fn offset(width: Pixels, line: Pixels, align: TextAlign) -> Pixels {
    let extra = (width - line).max(px(0.));
    match align {
        TextAlign::Center => extra / 2.,
        TextAlign::Right => extra,
        TextAlign::Left => px(0.),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_alignment_offsets_match_each_wrapped_line_width() {
        assert_eq!(offset(px(224.), px(96.), TextAlign::Center), px(64.));
        assert_eq!(offset(px(224.), px(96.), TextAlign::Right), px(128.));
        assert_eq!(offset(px(224.), px(224.), TextAlign::Center), px(0.));
        assert_eq!(offset(px(224.), px(96.), TextAlign::Left), px(0.));
    }
}
