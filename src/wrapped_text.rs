use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, AvailableSpace, Bounds, ContentMask, Element, ElementId, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, Pixels, Window, div, point, prelude::*, px, size,
};

use crate::{
    render_context::RenderContext,
    render_text::{self, TextFormat},
    selection::Position,
    text_cache::{Config, EntryHandle, Source, TextCache},
};

pub(crate) struct WrappedText {
    source: Source,
    position: Position,
    format: TextFormat,
    resources: RenderContext,
}

impl WrappedText {
    pub(crate) fn new(position: Position, format: TextFormat, resources: &RenderContext) -> Self {
        Self {
            source: Source {
                block: position.block,
                cell: resources.cell,
            },
            position,
            format,
            resources: resources.clone(),
        }
    }
}

impl IntoElement for WrappedText {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

type Layout = (Rc<RefCell<Option<EntryHandle>>>, Pixels);
type PaintedChunk = (AnyElement, Bounds<Pixels>, bool);

impl Element for WrappedText {
    type RequestLayoutState = Layout;
    type PrepaintState = Vec<PaintedChunk>;

    fn id(&self) -> Option<ElementId> {
        Some(
            gpui::SharedString::from(format!(
                "wrapped-{}-{}",
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
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = style
            .line_height
            .to_pixels(font_size.into(), window.rem_size());
        let result = Rc::new(RefCell::new(None));
        let result_for_measure = Rc::clone(&result);
        let resources = self.resources.clone();
        let source = self.source;
        let kind = self.format.kind;
        let layout = window.request_measured_layout(
            Default::default(),
            move |known, available, window, cx| {
                let width = known
                    .width
                    .unwrap_or_else(|| match available.width {
                        AvailableSpace::Definite(width) => width,
                        _ => px(resources.width),
                    })
                    .max(px(1.));
                let entry = TextCache::request(
                    &resources.texts,
                    source,
                    Config {
                        width: width.into(),
                        font_size: font_size.into(),
                        kind,
                    },
                    window,
                    cx,
                );
                let height = line_height * entry.borrow().estimated_rows() as f32;
                result_for_measure.borrow_mut().replace(entry);
                size(width, height)
            },
        );
        (layout, (result, line_height))
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Layout,
        window: &mut Window,
        cx: &mut App,
    ) -> Vec<PaintedChunk> {
        let (entry, line_height) = layout;
        let Some(entry) = entry.borrow().clone() else {
            return Vec::new();
        };
        let mask = window.content_mask().bounds;
        if !bounds.intersects(&mask) {
            return Vec::new();
        }
        let first = ((mask.top() - bounds.top()).max(px(0.)) / *line_height) as usize;
        let last = (((mask.bottom() - bounds.top()).max(px(0.)) / *line_height).ceil() as usize)
            .saturating_add(1);
        let target = if mask.bottom() >= bounds.bottom() - *line_height * 2. {
            usize::MAX
        } else {
            last.saturating_add(8)
        };
        TextCache::demand(
            &self.resources.texts,
            &entry,
            self.source,
            target,
            window,
            cx,
        );
        let entry = entry.borrow();
        let start = entry
            .chunks
            .partition_point(|chunk| chunk.row + chunk.rows < first);
        let mut children = Vec::new();
        for chunk in entry.chunks[start..]
            .iter()
            .take_while(|chunk| chunk.row <= last)
        {
            let slice = entry.slice(
                &self.resources.loaded.document,
                self.source,
                chunk.range.clone(),
            );
            let text = render_text::text(
                &self.resources.loaded.document,
                Position {
                    byte: self.position.byte + chunk.range.start,
                    ..self.position
                },
                &slice,
                self.format,
                Rc::clone(&self.resources.links),
                Rc::clone(&self.resources.selection),
                self.resources.owner,
            );
            let mut child = div()
                .id(("text-chunk", chunk.row))
                .w_full()
                .child(text)
                .into_any_element();
            let bounds = Bounds::new(
                point(
                    bounds.left(),
                    bounds.top() + *line_height * chunk.row as f32,
                ),
                size(bounds.size.width, *line_height * chunk.rows as f32),
            );
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                child.prepaint_as_root(
                    point(
                        bounds.left(),
                        bounds.top() - *line_height * chunk.skip_rows as f32,
                    ),
                    size(
                        AvailableSpace::Definite(bounds.size.width),
                        AvailableSpace::MinContent,
                    ),
                    window,
                    cx,
                );
            });
            children.push((child, bounds, true));
        }
        if children.is_empty() {
            let mut child = div()
                .child(
                    entry
                        .error
                        .clone()
                        .unwrap_or_else(|| "Loading text...".into()),
                )
                .into_any_element();
            child.prepaint_as_root(
                point(bounds.left(), bounds.top().max(mask.top())),
                size(
                    AvailableSpace::Definite(bounds.size.width),
                    AvailableSpace::MinContent,
                ),
                window,
                cx,
            );
            children.push((child, bounds, entry.error.is_some()));
        }
        children
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Layout,
        children: &mut Vec<PaintedChunk>,
        window: &mut Window,
        cx: &mut App,
    ) {
        for (child, bounds, ready) in children {
            window.with_content_mask(Some(ContentMask { bounds: *bounds }), |window| {
                child.paint(window, cx)
            });
            if *ready && bounds.intersects(&window.content_mask().bounds) {
                self.resources.paint.mark(window);
            }
        }
    }
}
