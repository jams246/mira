mod reveal;

use gpui::{
    AnyElement, App, AvailableSpace, Bounds, Element, ElementId, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, Pixels, Style, Window, font, point, px, size,
};
use mira::document::BlockKind;

use crate::{
    code_cache::{CodeCache, MIN_UNICODE_BYTES, Source},
    code_glyphs::CodeGlyphs,
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
    type RequestLayoutState = (Pixels, Pixels, Pixels);
    type PrepaintState = (Vec<AnyElement>, Vec<CodeGlyphs>);

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
            let fallback = f32::from(advance) * layout.max_columns().max(1) as f32;
            let width =
                CodeCache::width(&self.resources.codes, self.block, advance.into(), fallback);
            style.size = size(
                px(width.max(1.)).into(),
                (line_height * layout.line_count() as f32).into(),
            );
        }
        (
            window.request_layout(style, [], cx),
            (advance, line_height, font_size),
        )
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
        let (advance, line_height, font_size) = *metrics;
        let loaded = &self.resources.loaded;
        let Some(layout) = loaded.code.get(&self.block) else {
            return (Vec::new(), Vec::new());
        };
        let BlockKind::Code { runs, .. } = &loaded.document.blocks[self.block].kind else {
            return (Vec::new(), Vec::new());
        };
        let pending_row = reveal::prepare(self, bounds, *metrics, window, cx);
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
        let mut unicode = Vec::new();
        for row in first..last {
            let Some(line) = layout.line(row) else {
                continue;
            };
            if line.ascii && line.columns < first_column {
                continue;
            }
            if !line.ascii && line.bytes.len() >= MIN_UNICODE_BYTES {
                let source = Source {
                    block: self.block,
                    row,
                };
                let entry =
                    CodeCache::request(&self.resources.codes, source, &line, font_size.into());
                let target = if pending_row == Some(row) {
                    f32::INFINITY
                } else {
                    f32::from((mask.right() - bounds.left()).max(px(0.))) + 256.
                };
                CodeCache::demand(&self.resources.codes, &entry, target, window, cx);
                unicode.push(CodeGlyphs::prepare(
                    &self.resources,
                    source,
                    &entry.borrow(),
                    Bounds::new(
                        point(bounds.left(), bounds.top() + line_height * row as f32),
                        size(bounds.size.width, line_height),
                    ),
                    font_size,
                    window,
                ));
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
                &self.resources,
            );
            if !range.is_empty() {
                child = self.resources.paint.wrap(child);
            }
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
        (children, unicode)
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
        for child in &mut children.0 {
            child.paint(window, cx);
        }
        for row in &children.1 {
            if row.paint(&self.resources, window, cx) {
                self.resources.paint.mark(window);
            }
        }
    }
}
