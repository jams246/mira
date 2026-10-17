use std::{cell::RefCell, collections::HashMap, rc::Rc};

use gpui::{Font, Pixels, ScrollHandle, Window, px};

pub(crate) type TableScrolls = Rc<RefCell<TableState>>;

#[derive(Default)]
pub(crate) struct TableState {
    pub scrolls: HashMap<usize, ScrollHandle>,
    heights: HashMap<usize, Pixels>,
    metrics: Option<(Font, Pixels, Pixels)>,
}

impl TableState {
    pub(crate) fn minimum(&mut self, row: usize, window: &Window) -> Pixels {
        let style = window.text_style();
        let metrics = (
            style.font(),
            style.font_size.to_pixels(window.rem_size()),
            window.rem_size(),
        );
        if self.metrics.as_ref() != Some(&metrics) {
            self.heights.clear();
            self.metrics = Some(metrics);
        }
        self.heights.get(&row).copied().unwrap_or(px(0.))
    }

    pub(crate) fn observe(&mut self, row: usize, height: Pixels) {
        self.heights
            .entry(row)
            .and_modify(|old| *old = (*old).max(height))
            .or_insert(height);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revealing_shorter_cells_preserves_each_rows_largest_known_height() {
        let mut state = TableState::default();
        state.observe(1, px(41.));
        state.observe(2, px(55.));
        state.observe(1, px(209.));
        state.observe(1, px(41.));
        assert_eq!((state.heights[&1], state.heights[&2]), (px(209.), px(55.)));
    }
}
