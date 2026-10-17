use std::{ops::Range, rc::Rc};

use gpui::{AnyElement, App, IntoElement, TextAlign, Window, canvas, div, prelude::*, px};
use mira::document::{BlockKind, Document};
use pulldown_cmark::Alignment;

use crate::{
    render_context::RenderContext,
    render_inline,
    render_text::{TextFormat, TextKind},
    selection::Position,
};

pub(crate) use crate::table_state::TableScrolls;
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
        .scrolls
        .entry(*table)
        .or_default()
        .clone();
    let columns = visible_columns(cells.len(), -f32::from(scroll.offset().x), resources.width);
    let minimum = resources.tables.borrow_mut().minimum(index, window);
    let tables = Rc::clone(&resources.tables);
    let texts = Rc::clone(&resources.texts);
    let flows = Rc::clone(&resources.flows);
    let observed_columns = columns.clone();
    let prepaint_texts = Rc::clone(&texts);
    let prepaint_flows = Rc::clone(&flows);
    let prepaint_columns = columns.clone();
    let measure = canvas(
        move |_, _, _| {
            prepaint_texts
                .borrow()
                .height_known(index, &prepaint_columns)
                && prepaint_flows
                    .borrow()
                    .height_known(index, &prepaint_columns)
        },
        move |bounds, known_before_cells, _, _| {
            if known_before_cells
                && texts.borrow().height_known(index, &observed_columns)
                && flows.borrow().height_known(index, &observed_columns)
            {
                tables.borrow_mut().observe(index, bounds.size.height);
            }
        },
    )
    .absolute()
    .inset_0()
    .size_full();
    let mut resources = resources.clone();
    let mut contents = div()
        .relative()
        .flex()
        .flex_none()
        .min_h(minimum)
        .w(px(cells.len() as f32 * COLUMN_WIDTH))
        // Observe resolvedness before child prepaint can complete a provisional layout.
        .child(measure)
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
    for (column, runs) in cells
        .iter()
        .enumerate()
        .take(columns.end)
        .skip(columns.start)
    {
        resources.cell = Some(column);
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
                .border_color(resources.palette.border)
                .when(column == 0, |cell| cell.border_l_1())
                .when(*header, |cell| {
                    cell.border_t_1().bg(resources.palette.panel)
                })
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
        .map(|mut scroller| {
            scroller.style().restrict_scroll_to_axis = Some(true);
            scroller
        })
        .track_scroll(&scroll)
        .child(contents)
        .into_any_element()
}

fn visible_columns(count: usize, left: f32, width: f32) -> Range<usize> {
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
        assert_eq!(visible_columns(3, 300., 400.), 0..3);
        assert_eq!(visible_columns(20, 0., 936.), 0..5);
    }
}
