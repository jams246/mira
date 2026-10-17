use std::ops::Range;

use gpui::{Bounds, Context, EntityInputHandler, Pixels, Point, UTF16Selection, Window, point};

use super::{QueryInput, coordinates};

impl EntityInputHandler for QueryInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = coordinates::from_utf16_range(&self.content, &range_utf16);
        *actual_range = Some(self.utf16_range(&range));
        Some(self.content[range].to_owned())
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.utf16_range(&self.selected),
            reversed: self.reversed,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked.as_ref().map(|range| self.utf16_range(range))
    }

    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.marked = None;
        cx.notify();
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = self.replacement_range(range_utf16);
        let text = coordinates::single_line(new_text);
        let end = range.start + text.len();
        self.replace(range, &text, cx);
        self.selected = end..end;
        self.reversed = false;
        self.marked = None;
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = self.replacement_range(range_utf16);
        let text = coordinates::single_line(new_text);
        let start = range.start;
        let end = start + text.len();
        let selection = new_selected_range_utf16
            .as_ref()
            .map(|range| coordinates::from_utf16_range(&text, range));
        self.replace(range, &text, cx);
        self.marked = (!text.is_empty()).then_some(start..end);
        self.selected = selection
            .map(|range| start + range.start..start + range.end)
            .unwrap_or(end..end);
        self.reversed = false;
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let bounds = self.last_bounds?;
        let line = self.last_layout.as_ref()?;
        let range = coordinates::from_utf16_range(&self.content, &range_utf16);
        Some(Bounds::from_corners(
            point(
                bounds.left() + line.x_for_index(range.start) - self.scroll,
                bounds.top(),
            ),
            point(
                bounds.left() + line.x_for_index(range.end) - self.scroll,
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        position: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let bounds = self.last_bounds?;
        if !bounds.contains(&position) {
            return None;
        }
        let line = self.last_layout.as_ref()?;
        let index = line.closest_index_for_x(position.x - bounds.left() + self.scroll);
        Some(coordinates::to_utf16(&self.content, index))
    }
}

impl QueryInput {
    fn utf16_range(&self, range: &Range<usize>) -> Range<usize> {
        coordinates::to_utf16(&self.content, range.start)
            ..coordinates::to_utf16(&self.content, range.end)
    }

    fn replacement_range(&self, range_utf16: Option<Range<usize>>) -> Range<usize> {
        range_utf16
            .as_ref()
            .map(|range| coordinates::from_utf16_range(&self.content, range))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selected.clone())
    }
}
