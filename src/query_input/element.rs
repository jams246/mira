use gpui::{
    App, Bounds, ContentMask, Element, ElementId, ElementInputHandler, Entity, GlobalElementId,
    IntoElement, LayoutId, PaintQuad, Pixels, Point, ShapedLine, Style, TextRun, UnderlineStyle,
    Window, fill, point, px, relative, size,
};

use super::QueryInput;
use crate::appearance::Palette;

pub(super) struct QueryText {
    pub input: Entity<QueryInput>,
}

pub(super) struct PrepaintState {
    line: ShapedLine,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
    origin: Point<Pixels>,
    scroll: Pixels,
}

impl IntoElement for QueryText {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for QueryText {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> PrepaintState {
        let input = self.input.read(cx);
        let palette = Palette::for_window(window);
        let style = window.text_style();
        let empty = input.content.is_empty();
        let text = if empty {
            "Find in document".into()
        } else {
            input.content.clone()
        };
        let run = TextRun {
            len: text.len(),
            font: style.font(),
            color: if empty { palette.muted } else { palette.text },
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = match &input.marked {
            Some(marked) => [
                TextRun {
                    len: marked.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked.len(),
                    underline: Some(UnderlineStyle {
                        color: Some(palette.text),
                        thickness: px(1.),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: text.len() - marked.end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect(),
            None => vec![run],
        };
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(text, font_size, &runs, None);
        let cursor_x = line.x_for_index(input.cursor());
        let available = (bounds.size.width - px(2.)).max(px(0.));
        let mut scroll = input.scroll;
        if cursor_x < scroll {
            scroll = cursor_x;
        }
        if cursor_x > scroll + available {
            scroll = cursor_x - available;
        }
        let content_width = if empty { px(0.) } else { line.width };
        scroll = scroll
            .min((content_width - available).max(px(0.)))
            .max(px(0.));
        let origin = point(bounds.left() - scroll, bounds.top());
        let (selection, cursor) = if input.selected.is_empty() {
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(origin.x + cursor_x, bounds.top()),
                        size(px(1.), bounds.size.height),
                    ),
                    palette.text,
                )),
            )
        } else {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(
                            origin.x + line.x_for_index(input.selected.start),
                            bounds.top(),
                        ),
                        point(
                            origin.x + line.x_for_index(input.selected.end),
                            bounds.bottom(),
                        ),
                    ),
                    palette.selection,
                )),
                None,
            )
        };
        PrepaintState {
            line,
            cursor,
            selection,
            origin,
            scroll,
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        prepaint: &mut PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus = self.input.read(cx).focus.clone();
        window.handle_input(
            &focus,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        let focused = focus.is_focused(window);
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            if let Some(selection) = prepaint.selection.take() {
                window.paint_quad(selection);
            }
            let _ = prepaint
                .line
                .paint(prepaint.origin, window.line_height(), window, cx);
            if focused && let Some(cursor) = prepaint.cursor.take() {
                window.paint_quad(cursor);
            }
        });
        self.input.update(cx, |input, _| {
            input.last_layout = Some(prepaint.line.clone());
            input.last_bounds = Some(bounds);
            input.scroll = prepaint.scroll;
        });
    }
}
