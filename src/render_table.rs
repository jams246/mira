use std::{cell::RefCell, collections::HashMap, rc::Rc};

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
    let mut resources = resources.clone();
    resources.width = 223.;
    let mut contents = div().flex().flex_none();
    let mut offset = 0;
    for (column, runs) in cells.iter().enumerate() {
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
                .w(px(240.))
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
    div()
        .id(("table-row", index))
        .w_full()
        .flex()
        .overflow_x_scroll()
        .track_scroll(&scroll)
        .child(contents)
        .into_any_element()
}
