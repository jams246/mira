use std::{ops::Range, rc::Rc};

use gpui::{
    AnyElement, EntityId, IntoElement, MouseButton, SharedString, TextLayout, TextRun, div,
    prelude::*, rgb,
};

use crate::{
    selection::{Position, SelectionHandle, boundary},
    text_map::TextMap,
};

pub(crate) fn wrap(
    index: usize,
    text: SharedString,
    layout: TextLayout,
    child: AnyElement,
    selection: SelectionHandle,
    owner: EntityId,
    mapping: TextMap,
) -> AnyElement {
    let drag_selection = Rc::clone(&selection);
    let drag_layout = layout.clone();
    let drag_text = text.clone();
    let drag_mapping = mapping.clone();
    let nowrap = mapping.preformatted;
    div()
        .id(("selection", index))
        .when(!nowrap, |text| text.w_full())
        .when(nowrap, |text| text.flex_none().whitespace_nowrap())
        .min_w_0()
        .cursor_text()
        .on_mouse_down(MouseButton::Left, move |event, _window, cx| {
            let byte = layout
                .index_for_position(event.position)
                .unwrap_or_else(|nearest| nearest);
            selection.borrow_mut().start(Position {
                block: index,
                byte: mapping.to_source(boundary(&text, byte)),
            });
            cx.notify(owner);
        })
        .on_mouse_move(move |event, _window, cx| {
            if event.pressed_button != Some(MouseButton::Left) {
                return;
            }
            let byte = drag_layout
                .index_for_position(event.position)
                .unwrap_or_else(|nearest| nearest);
            if drag_selection.borrow_mut().extend(Position {
                block: index,
                byte: drag_mapping.to_source(boundary(&drag_text, byte)),
            }) {
                cx.notify(owner);
            }
        })
        .child(child)
        .into_any_element()
}

pub(crate) fn highlight(runs: Vec<TextRun>, selection: Option<Range<usize>>) -> Vec<TextRun> {
    let Some(selection) = selection else {
        return runs;
    };
    let mut output = Vec::with_capacity(runs.len() + 2);
    let mut start = 0;
    for run in runs {
        let end = start + run.len;
        let mut boundaries = [
            start,
            selection.start.clamp(start, end),
            selection.end.clamp(start, end),
            end,
        ];
        boundaries.sort_unstable();
        for pair in boundaries.windows(2) {
            if pair[0] == pair[1] {
                continue;
            }
            let mut part = run.clone();
            part.len = pair[1] - pair[0];
            if selection.contains(&pair[0]) {
                part.background_color = Some(rgb(0xb6d7ff).into());
            }
            output.push(part);
        }
        start = end;
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{FontStyle, FontWeight, font};

    #[test]
    fn selection_splits_runs_without_losing_nested_styles() {
        let mut font = font("Segoe UI");
        font.style = FontStyle::Italic;
        font.weight = FontWeight::BOLD;
        let run = TextRun {
            len: 8,
            font,
            color: rgb(0x242424).into(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let selected = highlight(vec![run.clone()], Some(2..6));
        assert_eq!(
            selected.iter().map(|run| run.len).collect::<Vec<_>>(),
            vec![2, 4, 2]
        );
        assert!(selected[1].background_color.is_some());
        assert_eq!(selected[1].font, run.font);
        assert!(selected[0].background_color.is_none());
    }
}
