use gpui::{
    App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId,
    Pixels, Style, StyledText, WhiteSpace, Window, px, relative,
};

pub(super) struct MeasuredText {
    pub text: StyledText,
    pub nowrap: bool,
}

impl IntoElement for MeasuredText {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl Element for MeasuredText {
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
        // Measurement needs native text and box geometry without interaction state.
        let mut style = Style {
            flex_shrink: 0.,
            ..Default::default()
        };
        style.min_size.width = px(0.).into();
        style.max_size.width = relative(1.).into();
        if self.nowrap {
            style.text.white_space = Some(WhiteSpace::Nowrap);
        }
        let id = window.with_text_style(style.text_style().cloned(), |window| {
            let (child, ()) = self.text.request_layout(None, None, window, cx);
            window.request_layout(style, [child], cx)
        });
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
