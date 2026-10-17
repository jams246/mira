use std::rc::Rc;

use gpui::{
    Context, MouseDownEvent, Window, anchored, canvas, deferred, div, point, prelude::*, px,
};

use super::{
    AcceptZoom, BackTabZoom, DismissZoom, FirstZoom, LastZoom, NextZoom, PreviousZoom, Shell,
    TabZoom, ToggleZoom, ZoomMenu,
};
use crate::{appearance::Palette, controls};

impl Shell {
    pub(in crate::shell) fn zoom_select(
        &self,
        loaded: bool,
        zoom: u16,
        palette: Palette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        let shell = cx.weak_entity();
        let bounds = Rc::clone(&self.zoom.bounds);
        div()
            .id("zoom-select-group")
            .relative()
            .flex_shrink_0()
            .w(px(76.))
            .h(px(28.))
            .track_focus(&self.zoom.group)
            .tab_stop(false)
            .child(
                canvas(move |area, _, _| bounds.set(area), |_, _, _, _| {})
                    .absolute()
                    .size_full(),
            )
            .child(
                controls::button(
                    "zoom-select",
                    format!("{zoom}%  ▾"),
                    "",
                    loaded,
                    palette,
                    move |window, cx| {
                        let _ = shell.update(cx, |shell, cx| shell.toggle_zoom_menu(window, cx));
                    },
                )
                .w_full()
                .border_color(palette.border)
                .key_context("ZoomSelect")
                .track_focus(&self.zoom.trigger)
                .accessible_name(format!("Zoom, {zoom}%, choose zoom level"))
                .accessible_value(format!("{zoom}%"))
                .on_action(cx.listener(|shell, _: &ToggleZoom, window, cx| {
                    shell.toggle_zoom_menu(window, cx)
                })),
            )
            .when_some(self.zoom.menu.as_ref(), |select, menu| {
                select.child(
                    div().absolute().top_0().left_0().child(deferred(
                        anchored()
                            .position_mode(gpui::AnchoredPositionMode::Local)
                            .position(point(px(76.), px(32.)))
                            .anchor(gpui::Corner::TopRight)
                            .snap_to_window()
                            .child(self.zoom_options(menu, zoom, palette, window, cx)),
                    )),
                )
            })
    }

    fn zoom_options(
        &self,
        menu: &ZoomMenu,
        zoom: u16,
        palette: Palette,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        div()
            .id("zoom-options")
            .key_context("ZoomMenu")
            .flex()
            .flex_col()
            .w(px(100.))
            .max_h((window.viewport_size().height - px(40.)).max(px(28.)))
            .p_1()
            .border_1()
            .border_color(palette.border)
            .rounded_sm()
            .bg(palette.background)
            .shadow_md()
            .overflow_y_scroll()
            .track_scroll(&menu.scroll)
            .occlude()
            .on_mouse_down_out(cx.listener(|shell, event: &MouseDownEvent, window, cx| {
                if !shell.zoom.bounds.get().contains(&event.position) {
                    shell.close_zoom_menu(window, cx);
                }
            }))
            .on_action(cx.listener(|shell, _: &PreviousZoom, window, cx| {
                shell.step_zoom_choice(true, window, cx)
            }))
            .on_action(cx.listener(|shell, _: &NextZoom, window, cx| {
                shell.step_zoom_choice(false, window, cx)
            }))
            .on_action(cx.listener(|shell, _: &FirstZoom, window, cx| {
                shell.focus_zoom_choice(0, window, cx)
            }))
            .on_action(cx.listener(|shell, _: &LastZoom, window, cx| {
                shell.focus_zoom_choice(10, window, cx)
            }))
            .on_action(
                cx.listener(|shell, _: &DismissZoom, window, cx| shell.close_zoom_menu(window, cx)),
            )
            .on_action(cx.listener(|shell, _: &TabZoom, window, cx| {
                shell.close_zoom_menu(window, cx);
                window.focus_next();
            }))
            .on_action(cx.listener(|shell, _: &BackTabZoom, window, cx| {
                shell.close_zoom_menu(window, cx);
                window.focus_prev();
            }))
            .children((100..=200).step_by(10).enumerate().map(|(index, level)| {
                let shell = cx.weak_entity();
                controls::button(
                    ("zoom-level", index),
                    format!("{level}%"),
                    format!("Zoom to {level}%"),
                    true,
                    palette,
                    move |window, cx| {
                        let _ = shell.update(cx, |shell, cx| shell.choose_zoom(level, window, cx));
                    },
                )
                .w_full()
                .justify_start()
                .key_context("ZoomMenu")
                .track_focus(&menu.focus[index])
                .tab_stop(false)
                .accessible_role(gpui::AccessibilityRole::ListItem)
                .accessible_selected(zoom == level)
                .accessible_set_position(index + 1, 11)
                .when(zoom == level, |option| option.bg(palette.selection))
                .on_action(cx.listener(
                    move |shell, _: &AcceptZoom, window, cx| shell.choose_zoom(level, window, cx),
                ))
            }))
    }
}
