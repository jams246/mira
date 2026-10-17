use gpui::{Context, Window, div, prelude::*, px};

use super::Shell;
use crate::{
    appearance::Palette,
    controls,
    viewer::{Open, Viewer},
};

impl Shell {
    pub(super) fn toolbar(
        &self,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        let active = self.active().cloned();
        let state = active.as_ref().map(|reader| reader.read(cx));
        let loaded = state.as_ref().is_some_and(|reader| reader.is_loaded());
        let requested = state
            .as_ref()
            .is_some_and(|reader| reader.requested_path().is_some());
        let back = state.as_ref().is_some_and(|reader| reader.can_back());
        let forward = state.as_ref().is_some_and(|reader| reader.can_forward());
        let headings = state.as_ref().is_some_and(|reader| reader.has_headings());
        let zoom = state.as_ref().map_or(100, |reader| reader.zoom_percent());
        let reader_button =
            |id,
             label: String,
             description,
             enabled,
             method: fn(&mut Viewer, &mut Window, &mut Context<Viewer>)| {
                let reader = active.clone();
                controls::button(
                    id,
                    label,
                    description,
                    enabled,
                    palette,
                    move |window, cx| {
                        if let Some(reader) = &reader {
                            reader.update(cx, |reader, cx| method(reader, window, cx));
                        }
                    },
                )
            };
        let shell = cx.weak_entity();
        div()
            .id("reader-toolbar")
            .flex()
            .flex_shrink_0()
            .items_center()
            .gap(px(2.))
            .px_1()
            .py_1()
            .overflow_x_scroll()
            .border_b_1()
            .border_color(palette.border)
            .child(reader_button(
                "back",
                "‹".into(),
                "Back (Alt+Left)",
                back,
                Viewer::back,
            ))
            .child(reader_button(
                "forward",
                "›".into(),
                "Forward (Alt+Right)",
                forward,
                Viewer::forward,
            ))
            .child(controls::button(
                "open",
                "Open",
                "Open Markdown (Ctrl+O)",
                true,
                palette,
                move |window, cx| {
                    let _ = shell.update(cx, |shell, cx| shell.open(&Open, window, cx));
                },
            ))
            .child(reader_button(
                "reload",
                "↻".into(),
                "Reload (F5)",
                requested,
                Viewer::reload,
            ))
            .child(reader_button(
                "contents",
                "≡".into(),
                "Table of Contents (Ctrl+Shift+O)",
                headings,
                Viewer::toggle_contents,
            ))
            .child(reader_button(
                "find",
                "Find".into(),
                "Find (Ctrl+F)",
                loaded,
                Viewer::open_find,
            ))
            .child(reader_button(
                "zoom-out",
                "-".into(),
                "Zoom out (Ctrl+-)",
                loaded && zoom > 50,
                Viewer::zoom_out,
            ))
            .child(
                reader_button(
                    "zoom-reset",
                    format!("{zoom}%"),
                    "Reset zoom (Ctrl+0)",
                    loaded,
                    Viewer::zoom_reset,
                )
                .accessible_name(format!("Reset zoom, {zoom}% (Ctrl+0)")),
            )
            .child(reader_button(
                "zoom-in",
                "+".into(),
                "Zoom in (Ctrl++)",
                loaded && zoom < 200,
                Viewer::zoom_in,
            ))
    }
}
