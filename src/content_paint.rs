use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, Pixels, Window,
};

use crate::timing::Timing;

#[derive(Clone)]
pub(crate) struct ContentPaint {
    marked: Rc<Cell<bool>>,
    timing: Timing,
}

impl ContentPaint {
    pub(crate) fn new(timing: Timing) -> Self {
        Self {
            marked: Rc::new(Cell::new(false)),
            timing,
        }
    }

    pub(crate) fn mark(&self, window: &Window) {
        if !self.marked.replace(true) {
            self.timing.mark("first_content_paint");
            let timing = self.timing;
            window.on_next_frame(move |_, _| timing.mark("first_content_callback"));
        }
    }

    pub(crate) fn wrap(&self, child: AnyElement) -> AnyElement {
        Painted {
            child,
            marker: self.clone(),
        }
        .into_any_element()
    }
}

struct Painted {
    child: AnyElement,
    marker: ContentPaint,
}

impl IntoElement for Painted {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for Painted {
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
        (self.child.request_layout(window, cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.prepaint(window, cx);
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
        if bounds.intersects(&window.content_mask().bounds) {
            self.marker.mark(window);
        }
    }
}
