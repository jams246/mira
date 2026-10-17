use std::{cell::RefCell, collections::HashMap, ops::Range, rc::Rc};

use gpui::{
    AnyElement, App, IntoElement, ScrollHandle, TextAlign, Window, div, prelude::*, px, rgb,
};
use mira::document::{BlockKind, Document};
use pulldown_cmark::Alignment;

use crate::{
    render_context::RenderContext,
    render_inline,
    render_text::{TextFormat, TextKind},
    selection::Position,
};

pub(crate) type TableScrolls = Rc<RefCell<HashMap<usize, ScrollHandle>>>;
const COLUMN_WIDTH: f32 = 240.;

pub(crate) fn row(
    document: &Document,
    index: usize,
    resources: &RenderContext,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let BlockKind::TableRow {
        table,
        header,
        cells,
    } = &document.blocks[index].kind
    else {
        return div().into_any_element();
    };
    let scroll = resources
        .tables
        .borrow_mut()
        .entry(*table)
        .or_default()
        .clone();
    let columns = visible_columns(cells.len(), -f32::from(scroll.offset().x), resources.width);
    let mut resources = resources.clone();
    let mut contents = div()
        .flex()
        .flex_none()
        .w(px(cells.len() as f32 * COLUMN_WIDTH))
        .child(
            div()
                .flex_none()
                .w(px(columns.start as f32 * COLUMN_WIDTH))
                .h_0(),
        );
    let mut offset = cells[..columns.start]
        .iter()
        .map(|runs| {
            runs.iter()
                .map(|run| run.text.as_str(&document.source).len())
                .sum::<usize>()
                + 1
        })
        .sum();
    // shortcut: Wide rows measure active columns; index unseen heights for varied-height wide rows.
    for (column, runs) in cells
        .iter()
        .enumerate()
        .take(columns.end)
        .skip(columns.start)
    {
        resources.width = if column == 0 { 222. } else { 223. };
        let align = document.tables[*table]
            .alignments
            .get(column)
            .copied()
            .unwrap_or(Alignment::None);
        let align = match align {
            Alignment::Center => TextAlign::Center,
            Alignment::Right => TextAlign::Right,
            _ => TextAlign::Left,
        };
        let text = render_inline::content(
            document,
            Position {
                block: index,
                byte: offset,
            },
            runs,
            TextFormat {
                kind: if *header {
                    TextKind::Heading
                } else {
                    TextKind::Body
                },
                align,
                inline: false,
            },
            &resources,
            window,
            cx,
        );
        offset += runs
            .iter()
            .map(|run| run.text.as_str(&document.source).len())
            .sum::<usize>()
            + 1;
        contents = contents.child(
            div()
                .id(("table-cell", column))
                .w(px(COLUMN_WIDTH))
                .flex_none()
                .p_2()
                .text_align(align)
                .border_b_1()
                .border_r_1()
                .border_color(rgb(0xd0d4d8))
                .when(column == 0, |cell| cell.border_l_1())
                .when(*header, |cell| cell.border_t_1().bg(rgb(0xeeeeee)))
                .child(text),
        );
    }
    contents = contents.child(
        div()
            .flex_none()
            .w(px((cells.len() - columns.end) as f32 * COLUMN_WIDTH))
            .h_0(),
    );
    div()
        .id(("table-row", index))
        .w_full()
        .flex()
        .overflow_x_scroll()
        .track_scroll(&scroll)
        .child(contents)
        .into_any_element()
}

fn visible_columns(count: usize, left: f32, width: f32) -> Range<usize> {
    if count <= 64 {
        return 0..count;
    }
    let first = ((left.max(0.) / COLUMN_WIDTH) as usize)
        .saturating_sub(1)
        .min(count);
    let last = (((left.max(0.) + width.max(1.)) / COLUMN_WIDTH).ceil() as usize)
        .saturating_add(1)
        .min(count);
    first..last
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_tables_bound_visible_columns_with_one_column_overscan() {
        assert_eq!(visible_columns(40_000, 0., 936.), 0..5);
        assert_eq!(visible_columns(40_000, 2400., 936.), 9..15);
        assert_eq!(visible_columns(40_000, 9_599_064., 936.), 39_995..40_000);
        assert_eq!(visible_columns(3, 500., 400.), 0..3);
    }
}
