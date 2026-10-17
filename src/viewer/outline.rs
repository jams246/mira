use super::*;
use crate::{appearance::Palette, controls};
use gpui::{div, prelude::*, px, uniform_list};
use mira::reading_state::Location;

impl Viewer {
    pub(crate) fn toggle_contents(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        if !self.headings.is_empty() {
            let location = self.location();
            self.contents = !self.contents;
            if let Some(loaded) = &self.loaded {
                self.list.reset(loaded.document.blocks.len());
            }
            self.jump(location, false, cx);
            cx.notify();
        }
    }
    pub(super) fn outline(
        &self,
        width: f32,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> gpui::Div {
        let current = self
            .headings
            .partition_point(|h| h.block <= self.list.logical_scroll_top().item_ix)
            .checked_sub(1);
        div()
            .w(px(width))
            .flex_none()
            .h_full()
            .border_r_1()
            .border_color(palette.border)
            .child(
                uniform_list(
                    "contents",
                    self.headings.len(),
                    cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                        range
                            .map(|index| {
                                let heading = &this.headings[index];
                                let block = heading.block;
                                let label = gpui::SharedString::from(heading.title.clone());
                                let tip = label.clone();
                                let viewer = cx.weak_entity();
                                let jump = move |_: &mut Window, cx: &mut App| {
                                    let _ = viewer.update(cx, |this, cx| {
                                        this.jump(
                                            Location {
                                                block,
                                                ..Location::default()
                                            },
                                            true,
                                            cx,
                                        )
                                    });
                                };
                                controls::button(
                                    ("heading", index),
                                    label,
                                    tip,
                                    true,
                                    palette,
                                    jump,
                                )
                                .accessible_role(gpui::AccessibilityRole::ListItem)
                                .accessible_selected(current == Some(index))
                                .accessible_set_position(index + 1, this.headings.len())
                                .w_full()
                                .justify_start()
                                .h(px(30.))
                                .pl(px(8. + f32::from(heading.level.saturating_sub(1)) * 10.))
                                .overflow_hidden()
                                .text_ellipsis()
                                .when(current == Some(index), |row| row.bg(palette.selection))
                            })
                            .collect()
                    }),
                )
                .size_full(),
            )
    }
}
