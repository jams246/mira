pub(crate) mod row;

use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, AvailableSpace, Bounds, ContentMask, Element, ElementId, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, Pixels, Window, div, point, prelude::*, px, size,
};

use crate::{
    flow_cache::{Config, EntryHandle, FlowCache, Input},
    render_context::RenderContext,
    render_text::TextFormat,
    selection::Position,
    text_cache::Source,
};

pub(crate) struct FlowView {
    position: Position,
    format: TextFormat,
    resources: RenderContext,
}

impl FlowView {
    pub(crate) fn new(position: Position, format: TextFormat, resources: &RenderContext) -> Self {
        Self {
            position,
            format,
            resources: resources.clone(),
        }
    }
}

impl IntoElement for FlowView {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

type Layout = Rc<RefCell<Option<EntryHandle>>>;
type PaintedRow = (AnyElement, Bounds<Pixels>);

impl Element for FlowView {
    type RequestLayoutState = Layout;
    type PrepaintState = Vec<PaintedRow>;

    fn id(&self) -> Option<ElementId> {
        Some(
            gpui::SharedString::from(format!(
                "flow-{}-{}",
                self.position.block, self.position.byte
            ))
            .into(),
        )
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        _: &mut App,
    ) -> (LayoutId, Layout) {
        let text = window.text_style();
        let font_size = text.font_size.to_pixels(window.rem_size());
        let line_height = text
            .line_height
            .to_pixels(font_size.into(), window.rem_size());
        let result = Rc::new(RefCell::new(None));
        let measured = Rc::clone(&result);
        let resources = self.resources.clone();
        let source = Source {
            block: self.position.block,
            cell: resources.cell,
        };
        let format = self.format;
        let id =
            window.request_measured_layout(Default::default(), move |known, available, _, _| {
                let width = known
                    .width
                    .unwrap_or_else(|| match available.width {
                        AvailableSpace::Definite(width) => width,
                        _ => px(resources.width),
                    })
                    .max(px(1.));
                let entry = FlowCache::request(
                    &resources.flows,
                    source,
                    Config {
                        width: width.into(),
                        font_size: font_size.into(),
                        line_height: line_height.into(),
                        format,
                        images: resources.images.borrow().geometry_revision(),
                    },
                );
                let height = px(entry.borrow().estimated_height());
                measured.borrow_mut().replace(entry);
                size(width, height)
            });
        (id, result)
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Layout,
        window: &mut Window,
        cx: &mut App,
    ) -> Vec<PaintedRow> {
        let mask = window.content_mask().bounds;
        if !bounds.intersects(&mask) {
            return Vec::new();
        }
        let Some(entry) = layout.borrow().clone() else {
            return Vec::new();
        };
        let target = if mask.bottom() >= bounds.bottom() - px(48.) {
            f32::INFINITY
        } else {
            f32::from((mask.bottom() - bounds.top()).max(px(0.))) + 200.
        };
        let input = Input {
            resources: &self.resources,
            position: self.position,
            format: self.format,
            width: bounds.size.width.into(),
        };
        FlowCache::advance(&self.resources.flows, &entry, input, target, window, cx);
        let entry = entry.borrow();
        let Some(index) = &entry.index else {
            return loading(bounds, mask, window, cx);
        };
        let first = f32::from((mask.top() - bounds.top()).max(px(0.)));
        let last = f32::from((mask.bottom() - bounds.top()).max(px(0.)));
        let start = entry
            .rows
            .partition_point(|row| row.top + row.height < first);
        let input = Input {
            resources: &self.resources,
            position: self.position,
            format: self.format,
            width: bounds.size.width.into(),
        };
        let mut children = Vec::new();
        for row in entry.rows[start..].iter().take_while(|row| row.top <= last) {
            let mut child = row::build(index, row.pieces.clone(), &input, false, None, window, cx);
            let row_bounds = Bounds::new(
                point(bounds.left(), bounds.top() + px(row.top)),
                size(bounds.size.width, px(row.height)),
            );
            window.with_content_mask(Some(ContentMask { bounds: row_bounds }), |window| {
                child.prepaint_as_root(
                    row_bounds.origin,
                    size(
                        AvailableSpace::Definite(bounds.size.width),
                        AvailableSpace::MinContent,
                    ),
                    window,
                    cx,
                );
            });
            children.push((child, row_bounds));
        }
        if children.is_empty() && !entry.complete {
            return loading(bounds, mask, window, cx);
        }
        children
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Layout,
        children: &mut Vec<PaintedRow>,
        window: &mut Window,
        cx: &mut App,
    ) {
        for (child, bounds) in children {
            window.with_content_mask(Some(ContentMask { bounds: *bounds }), |window| {
                child.paint(window, cx)
            });
        }
    }
}

fn loading(
    bounds: Bounds<Pixels>,
    mask: Bounds<Pixels>,
    window: &mut Window,
    cx: &mut App,
) -> Vec<PaintedRow> {
    let mut child = div().child("Loading text...").into_any_element();
    child.prepaint_as_root(
        point(bounds.left(), bounds.top().max(mask.top())),
        size(
            AvailableSpace::Definite(bounds.size.width),
            AvailableSpace::MinContent,
        ),
        window,
        cx,
    );
    vec![(child, bounds)]
}
