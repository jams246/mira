use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, AvailableSpace, Bounds, Element, ElementId, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, Pixels, Window, size,
};
use mira::flow_index::FlowIndex;

use super::Row;
use crate::{
    flow_view::row, render_context::RenderContext, render_text::TextFormat, selection::Position,
};

pub(crate) struct Input<'a> {
    pub resources: &'a RenderContext,
    pub position: Position,
    pub format: TextFormat,
    pub width: f32,
}

pub(super) fn next(
    index: &FlowIndex,
    cursor: usize,
    input: &Input<'_>,
    window: &mut Window,
    cx: &mut App,
) -> Vec<Row> {
    let pieces = index.pieces();
    let mut count = 64;
    loop {
        let mut end = (cursor + count).min(pieces.len());
        let byte_end = pieces[cursor].range.start.saturating_add(1024);
        if count == 64 {
            end = (cursor + 1..end)
                .find(|i| pieces[*i].range.start >= byte_end)
                .unwrap_or(end);
        }
        let ids = Rc::new(RefCell::new(Vec::new()));
        let mut native = row::build(index, cursor..end, input, true, Some(&ids), window, cx);
        let root = native.request_layout(window, cx);
        window.compute_layout(
            root,
            size(
                AvailableSpace::Definite(input.width.into()),
                AvailableSpace::MinContent,
            ),
            cx,
        );
        let origin = window.layout_bounds(root).origin;
        let bounds = ids
            .borrow()
            .iter()
            .map(|(piece, id)| {
                let mut bounds = window.layout_bounds(*id);
                bounds.origin -= origin;
                (*piece, bounds)
            })
            .collect::<Vec<_>>();
        let mut rows = group(&bounds, cursor);
        if end == pieces.len() {
            if let Some(last) = rows.last_mut() {
                last.pieces.end = end;
            }
            return rows;
        }
        if rows.len() > 1 {
            rows.pop();
            return rows;
        }
        // A native flex row is indivisible; grow only until its ending is known.
        count = count.saturating_mul(2).min(pieces.len() - cursor);
    }
}

fn group(bounds: &[(usize, Bounds<Pixels>)], start: usize) -> Vec<Row> {
    let mut rows: Vec<Row> = Vec::new();
    let mut pending = start;
    for (piece, bounds) in bounds {
        let top = f32::from(bounds.top());
        let bottom = f32::from(bounds.bottom());
        if bounds.size.height <= Pixels::ZERO {
            if let Some(row) = rows.last_mut() {
                row.pieces.end = piece + 1;
                pending = piece + 1;
            }
            continue;
        }
        if let Some(row) = rows.last_mut()
            && top < row.top + row.height - 0.01
            && bottom > row.top + 0.01
        {
            let old_bottom = row.top + row.height;
            row.top = row.top.min(top);
            row.height = bottom.max(old_bottom) - row.top;
            row.pieces.end = piece + 1;
        } else {
            rows.push(Row {
                pieces: pending..piece + 1,
                top,
                height: bottom - top,
            });
        }
        pending = piece + 1;
    }
    rows
}

pub(crate) type MeasuredIds = Rc<RefCell<Vec<(usize, LayoutId)>>>;

pub(crate) struct Measured {
    pub child: AnyElement,
    pub piece: usize,
    pub ids: MeasuredIds,
}

impl IntoElement for Measured {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for Measured {
    type RequestLayoutState = ();
    type PrepaintState = ();
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let id = self.child.request_layout(window, cx);
        self.ids.borrow_mut().push((self.piece, id));
        (id, ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut Window,
        _: &mut App,
    ) {
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        _: &mut Window,
        _: &mut App,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, px};

    #[test]
    fn native_baseline_items_share_rows_and_zero_height_breaks_advance() {
        let rect = |x, y, w, h| Bounds::new(point(px(x), px(y)), size(px(w), px(h)));
        let rows = group(
            &[
                (0, rect(0., 156., 30., 24.)),
                (1, rect(30., 0., 320., 180.)),
                (2, rect(0., 180., 936., 0.)),
                (3, rect(0., 180., 70., 24.)),
            ],
            0,
        );
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].pieces, 0..3);
        assert_eq!(rows[0].top, 0.);
        assert_eq!(rows[0].height, 180.);
        assert_eq!(rows[1].pieces, 3..4);
    }
}
