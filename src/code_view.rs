use gpui::{
    AnyElement, App, AvailableSpace, Bounds, Element, ElementId, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, Pixels, Style, Window, font, point, px, size,
};
use mira::document::BlockKind;

use crate::{
    render_context::RenderContext,
    render_text::{self, TextKind},
    selection::Position,
};

pub(crate) struct CodeView {
    block: usize,
    resources: RenderContext,
}

impl CodeView {
    pub(crate) fn new(block: usize, resources: &RenderContext) -> Self {
        Self {
            block,
            resources: resources.clone(),
        }
    }
}

impl IntoElement for CodeView {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for CodeView {
    type RequestLayoutState = (Pixels, Pixels);
    type PrepaintState = Vec<AnyElement>;

    fn id(&self) -> Option<ElementId> {
        Some(("code-lines", self.block).into())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let text = window.text_style();
        let font_size = text.font_size.to_pixels(window.rem_size());
        let line_height = text
            .line_height
            .to_pixels(font_size.into(), window.rem_size());
        let mut run = text.to_run(1);
        run.font = font("Consolas");
        let advance = window
            .text_system()
            .shape_line("M".into(), font_size, &[run], None)
            .width
            .max(px(1.));
        let mut style = Style {
            flex_shrink: 0.,
            ..Default::default()
        };
        if let Some(layout) = self.resources.loaded.code.get(&self.block) {
            style.size = size(
                (advance * layout.max_columns().max(1) as f32).into(),
                (line_height * layout.line_count() as f32).into(),
            );
        }
        (window.request_layout(style, [], cx), (advance, line_height))
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        metrics: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let (advance, line_height) = *metrics;
        let loaded = &self.resources.loaded;
        let Some(layout) = loaded.code.get(&self.block) else {
            return Vec::new();
        };
        let BlockKind::Code { runs, .. } = &loaded.document.blocks[self.block].kind else {
            return Vec::new();
        };
        let mask = window.content_mask().bounds;
        let first =
            (((mask.top() - bounds.top()).max(px(0.)) / line_height) as usize).saturating_sub(4);
        let last = ((((mask.bottom() - bounds.top()).max(px(0.)) / line_height).ceil() as usize)
            + 4)
        .min(layout.line_count());
        let first_column =
            (((mask.left() - bounds.left()).max(px(0.)) / advance) as usize).saturating_sub(32) / 4
                * 4;
        let last_column =
            ((mask.right() - bounds.left()).max(px(0.)) / advance).ceil() as usize + 32;
        let mut children = Vec::with_capacity(last.saturating_sub(first));
        for row in first..last {
            let Some(line) = layout.line(row) else {
                continue;
            };
            if line.ascii && line.columns < first_column {
                continue;
            }
            let range = if line.ascii {
                layout.byte_for_column(&line, first_column)
                    ..layout.byte_for_column(&line, last_column)
            } else {
                line.bytes.clone()
            };
            let column = if line.ascii {
                layout.column_for_byte(&line, range.start)
            } else {
                0
            };
            let slice = layout.slice(&loaded.document, runs, range.clone());
            let mut child = render_text::text(
                &loaded.document,
                Position {
                    block: self.block,
                    byte: range.start,
                },
                &slice,
                TextKind::Code.into(),
                self.resources.links.clone(),
                self.resources.selection.clone(),
                self.resources.owner,
            );
            child.prepaint_as_root(
                point(
                    bounds.left() + advance * column as f32,
                    bounds.top() + line_height * row as f32,
                ),
                size(AvailableSpace::MinContent, AvailableSpace::MinContent),
                window,
                cx,
            );
            children.push(child);
        }
        children
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _metrics: &mut Self::RequestLayoutState,
        children: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        if !children.is_empty() {
            self.resources.paint.mark(window);
        }
        for child in children {
            child.paint(window, cx);
        }
    }
}
