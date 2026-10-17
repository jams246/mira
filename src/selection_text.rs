use std::{ops::Range, rc::Rc};

use gpui::{
    AnyElement, DispatchPhase, EntityId, IntoElement, MouseButton, MouseMoveEvent, SharedString,
    TextAlign, TextLayout, TextRun, canvas, div, prelude::*,
};

use crate::{
    appearance::Palette,
    render_text::LinkHandler,
    selection::{Position, SelectionHandle, boundary},
    text_hit_test,
    text_map::TextMap,
};

pub(crate) struct Interaction {
    pub resources: crate::render_context::RenderContext,
    pub source_len: usize,
    pub position: Position,
    pub selection: SelectionHandle,
    pub owner: EntityId,
    pub mapping: TextMap,
    pub align: TextAlign,
    pub inline: bool,
    pub links: Vec<(Range<usize>, usize)>,
    pub activate_link: LinkHandler,
}

pub(crate) fn wrap(
    text: SharedString,
    layout: TextLayout,
    child: AnyElement,
    interaction: Interaction,
) -> AnyElement {
    let Interaction {
        resources,
        source_len,
        position,
        selection,
        owner,
        mapping,
        align,
        inline,
        links,
        activate_link,
    } = interaction;
    let drag_selection = Rc::clone(&selection);
    let link_selection = Rc::clone(&selection);
    let drag_layout = layout.clone();
    let link_layout = layout.clone();
    let drag_text = text.clone();
    let link_text = text.clone();
    let drag_mapping = mapping.clone();
    let link_mapping = mapping.clone();
    let capture_selection = Rc::clone(&selection);
    let capture_text = text.clone();
    let capture_layout = layout.clone();
    let capture_mapping = mapping.clone();
    let capture = canvas(
        move |_, window, _| {
            crate::reveal::prepaint::text(
                &resources,
                position,
                source_len,
                &capture_mapping,
                &capture_layout,
                window,
            );
            (resources, capture_mapping, capture_layout)
        },
        move |_, (resources, capture_mapping, capture_layout), window, cx| {
            crate::reveal::observe(
                &resources,
                position,
                source_len,
                &capture_mapping,
                &capture_layout,
                &capture_text,
                cx,
            );
            window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
                if phase != DispatchPhase::Capture
                    || event.pressed_button != Some(MouseButton::Left)
                    || !capture_selection.borrow().dragging_from(position)
                {
                    return;
                }
                let bounds = capture_layout.bounds();
                if event.position.y < bounds.top() || event.position.y >= bounds.bottom() {
                    return;
                }
                let byte =
                    text_hit_test::index(&capture_layout, &capture_text, event.position, align)
                        .unwrap_or_else(|nearest| nearest);
                if capture_selection.borrow_mut().extend(Position {
                    block: position.block,
                    byte: position.byte + capture_mapping.to_source(boundary(&capture_text, byte)),
                }) {
                    cx.notify(owner);
                }
            });
        },
    )
    .absolute()
    .size_full();
    let nowrap = mapping.preformatted;
    let has_links = !links.is_empty();
    div()
        .relative()
        .id(SharedString::from(format!(
            "selection-{}-{}",
            position.block, position.byte
        )))
        .when(!nowrap && !inline, |text| text.w_full())
        .when(inline, |text| text.flex_none().max_w_full())
        .when(nowrap, |text| text.flex_none().whitespace_nowrap())
        .min_w_0()
        .cursor_text()
        .when(has_links, |text| text.cursor_pointer())
        .on_mouse_down(MouseButton::Left, move |event, _window, cx| {
            let hit = text_hit_test::index(&layout, &text, event.position, align);
            let exact = hit.is_ok();
            let byte = hit.unwrap_or_else(|nearest| nearest);
            selection.borrow_mut().start_text(
                Position {
                    block: position.block,
                    byte: position.byte + mapping.to_source(boundary(&text, byte)),
                },
                position,
                exact,
            );
            cx.notify(owner);
        })
        .on_mouse_move(move |event, _window, cx| {
            if event.pressed_button != Some(MouseButton::Left) {
                return;
            }
            let byte = text_hit_test::index(&drag_layout, &drag_text, event.position, align)
                .unwrap_or_else(|nearest| nearest);
            if drag_selection.borrow_mut().extend(Position {
                block: position.block,
                byte: position.byte + drag_mapping.to_source(boundary(&drag_text, byte)),
            }) {
                cx.notify(owner);
            }
        })
        .on_mouse_up(MouseButton::Left, move |event, window, cx| {
            let up = text_hit_test::index(&link_layout, &link_text, event.position, align).ok();
            let target = {
                let selection = link_selection.borrow();
                selection
                    .link_anchor()
                    .filter(|anchor| anchor.block == position.block)
                    .and_then(|anchor| anchor.byte.checked_sub(position.byte))
                    .and_then(|down| {
                        let down = link_mapping.to_display(down);
                        links
                            .iter()
                            .find(|(range, _)| {
                                range.contains(&down) && up.is_some_and(|up| range.contains(&up))
                            })
                            .map(|(_, target)| *target)
                    })
            };
            if let Some(target) = target {
                activate_link(target, window, cx);
            }
        })
        .child(child)
        .child(capture)
        .into_any_element()
}

pub(crate) fn highlight(
    runs: Vec<TextRun>,
    selection: Option<Range<usize>>,
    palette: Palette,
) -> Vec<TextRun> {
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
                part.background_color = Some(palette.selection);
                part.color = palette.text;
            }
            output.push(part);
        }
        start = end;
    }
    output
}

#[cfg(test)]
mod tests;
