use super::*;
use crate::{appearance::Palette, controls};
use gpui::{div, prelude::*, px};

impl Viewer {
    pub(super) fn find_bar(&self, palette: Palette, cx: &mut Context<Self>) -> gpui::Div {
        let highlights = self.find.highlights.borrow();
        let len = highlights.matches.len();
        let status = if self.find.pending {
            "Searching...".into()
        } else if self.find.input.read(cx).query().is_empty() {
            String::new()
        } else if len == 0 {
            "No matches".into()
        } else {
            format!("{} / {len}", highlights.active.map_or(0, |i| i + 1))
        };
        let previous = Self::command(cx, Self::find_previous);
        let next = Self::command(cx, Self::find_next);
        let close = Self::command(cx, Self::close_find);
        let case = Self::command(cx, |this, window, cx| {
            this.find.match_case = !this.find.match_case;
            this.search(true, window, cx);
        });
        div()
            .flex()
            .flex_shrink_0()
            .flex_wrap()
            .items_center()
            .gap_1()
            .p_1()
            .border_b_1()
            .border_color(palette.border)
            .child(div().flex_1().min_w(px(80.)).child(self.find.input.clone()))
            .child(
                div()
                    .id("find-status")
                    .accessible_role(gpui::AccessibilityRole::StaticText)
                    .accessible_name(if !self.find.pending && len > 0 {
                        format!("Match {} of {len}", highlights.active.map_or(0, |i| i + 1))
                    } else {
                        status.clone()
                    })
                    .accessible_live(!self.find.pending)
                    .text_sm()
                    .text_color(palette.muted)
                    .when(!self.find.pending && len > 0, |label| label.child(status)),
            )
            .child(controls::button(
                "find-previous",
                "↑",
                "Previous match (Shift+F3)",
                len > 0 && !self.find.pending,
                palette,
                previous,
            ))
            .child(controls::button(
                "find-next",
                "↓",
                "Next match (F3)",
                len > 0 && !self.find.pending,
                palette,
                next,
            ))
            .child(
                controls::button(
                    "find-case",
                    if self.find.match_case { "Aa ✓" } else { "Aa" },
                    "Match case",
                    true,
                    palette,
                    case,
                )
                .accessible_role(gpui::AccessibilityRole::CheckBox)
                .accessible_selected(self.find.match_case),
            )
            .child(controls::button(
                "find-close",
                "×",
                "Close Find (Esc)",
                true,
                palette,
                close,
            ))
    }
}
