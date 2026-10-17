use gpui::{Context, ExternalPaths, MouseButton, NavigationDirection, Window, div, prelude::*, px};

use super::{CloseTab, FocusNext, FocusPrevious, NextTab, PreviousTab, Shell};
use crate::{appearance::Palette, controls, viewer::Open};

impl Shell {
    fn tab_strip(&self, palette: Palette, cx: &mut Context<Self>) -> gpui::Stateful<gpui::Div> {
        div()
            .id("document-tabs")
            .flex()
            .flex_shrink_0()
            .overflow_x_scroll()
            .track_scroll(&self.strip_scroll)
            .border_b_1()
            .border_color(palette.border)
            .children(self.tabs.iter().enumerate().map(|(index, tab)| {
                let id = tab.id;
                let path = tab.reader.read(cx).loaded_path().unwrap_or(&tab.path);
                let title = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                let path: gpui::SharedString = path.to_string_lossy().into_owned().into();
                let shell = cx.weak_entity();
                div()
                    .id(("tab", id))
                    .flex()
                    .items_center()
                    .flex_shrink_0()
                    .max_w(px(220.))
                    .bg(if self.order.active == Some(id) {
                        palette.panel
                    } else {
                        palette.background
                    })
                    .border_r_1()
                    .border_color(palette.border)
                    .child(
                        controls::button(
                            ("select-tab", id),
                            title.clone(),
                            path,
                            true,
                            palette,
                            move |window, cx| {
                                let _ = shell.update(cx, |shell, cx| shell.select(id, window, cx));
                            },
                        )
                        .accessible_name(title)
                        .accessible_role(gpui::AccessibilityRole::Tab)
                        .accessible_selected(self.order.active == Some(id))
                        .accessible_set_position(index + 1, self.tabs.len())
                        .max_w(px(180.))
                        .overflow_hidden(),
                    )
                    .child(controls::button(
                        ("close-tab", id),
                        "×",
                        "Close tab (Ctrl+W)",
                        true,
                        palette,
                        {
                            let shell = cx.weak_entity();
                            move |window, cx| {
                                let _ = shell.update(cx, |shell, cx| shell.close(id, window, cx));
                            }
                        },
                    ))
            }))
    }
}

impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::for_window(window);
        let title = self
            .order
            .active
            .and_then(|id| self.tabs.iter().find(|tab| tab.id == id))
            .map(|tab| {
                format!(
                    "{} | MIRA",
                    tab.path.file_name().unwrap_or_default().to_string_lossy()
                )
            })
            .unwrap_or_else(|| "MIRA".into());
        window.set_window_title(&title);
        let root = super::actions::reader_actions(div(), cx)
            .key_context("MiraWindow")
            .track_focus(&self.focus)
            .tab_group()
            .tab_stop(false)
            .size_full()
            .flex()
            .flex_col()
            .bg(palette.background)
            .text_color(palette.text)
            .font_family("Segoe UI")
            .on_action(cx.listener(Self::open))
            .on_action(cx.listener(|shell, _: &NextTab, window, cx| shell.cycle(false, window, cx)))
            .on_action(
                cx.listener(|shell, _: &PreviousTab, window, cx| shell.cycle(true, window, cx)),
            )
            .on_action(cx.listener(|shell, _: &CloseTab, window, cx| {
                if let Some(id) = shell.order.active {
                    shell.close(id, window, cx);
                }
            }))
            .on_action(|_: &FocusNext, window, _| window.focus_next())
            .on_action(|_: &FocusPrevious, window, _| window.focus_prev())
            .on_mouse_down(
                MouseButton::Navigate(NavigationDirection::Back),
                cx.listener(|shell, _, window, cx| {
                    if let Some(reader) = shell.active() {
                        reader.update(cx, |reader, cx| reader.back(window, cx));
                    }
                }),
            )
            .on_mouse_down(
                MouseButton::Navigate(NavigationDirection::Forward),
                cx.listener(|shell, _, window, cx| {
                    if let Some(reader) = shell.active() {
                        reader.update(cx, |reader, cx| reader.forward(window, cx));
                    }
                }),
            )
            .on_drop::<ExternalPaths>(cx.listener(|shell, paths: &ExternalPaths, window, cx| {
                shell.open_paths(paths.paths().to_vec(), window, cx);
            }))
            .child(self.toolbar(palette, cx))
            .when(!self.tabs.is_empty(), |root| {
                root.child(self.tab_strip(palette, cx))
            })
            .when_some(self.message.clone(), |root, message| {
                root.child(div().px_3().py_1().text_size(px(12.)).child(message))
            });
        if let Some(reader) = self.active() {
            return root.child(div().flex_1().min_h_0().child(reader.clone()));
        }
        root.child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_3()
                .child(div().text_3xl().child("MIRA"))
                .child("Open a Markdown file to start reading.")
                .child(controls::button(
                    "empty-open",
                    "Open Markdown (Ctrl+O)",
                    "Open Markdown",
                    true,
                    palette,
                    {
                        let shell = cx.weak_entity();
                        move |window, cx| {
                            let _ = shell.update(cx, |shell, cx| shell.open(&Open, window, cx));
                        }
                    },
                )),
        )
    }
}
