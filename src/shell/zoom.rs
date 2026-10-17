use std::{cell::Cell, rc::Rc};

mod view;

use gpui::{
    App, Bounds, Context, FocusHandle, KeyBinding, Pixels, ScrollHandle, Subscription, Window,
    actions,
};

use super::Shell;

actions!(
    zoom_select,
    [
        ToggleZoom,
        PreviousZoom,
        NextZoom,
        FirstZoom,
        LastZoom,
        AcceptZoom,
        DismissZoom,
        TabZoom,
        BackTabZoom
    ]
);

pub(super) fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("enter", ToggleZoom, Some("ZoomSelect")),
        KeyBinding::new("space", ToggleZoom, Some("ZoomSelect")),
        KeyBinding::new("down", ToggleZoom, Some("ZoomSelect")),
        KeyBinding::new("alt-down", ToggleZoom, Some("ZoomSelect")),
        KeyBinding::new("up", PreviousZoom, Some("ZoomMenu")),
        KeyBinding::new("down", NextZoom, Some("ZoomMenu")),
        KeyBinding::new("home", FirstZoom, Some("ZoomMenu")),
        KeyBinding::new("end", LastZoom, Some("ZoomMenu")),
        KeyBinding::new("enter", AcceptZoom, Some("ZoomMenu")),
        KeyBinding::new("space", AcceptZoom, Some("ZoomMenu")),
        KeyBinding::new("escape", DismissZoom, Some("ZoomMenu")),
        KeyBinding::new("tab", TabZoom, Some("ZoomMenu")),
        KeyBinding::new("shift-tab", BackTabZoom, Some("ZoomMenu")),
    ]);
}

pub(super) struct ZoomSelect {
    trigger: FocusHandle,
    group: FocusHandle,
    bounds: Rc<Cell<Bounds<Pixels>>>,
    pub(super) menu: Option<ZoomMenu>,
    _blur: Subscription,
}

pub(super) struct ZoomMenu {
    pub(super) focus: [FocusHandle; 11],
    scroll: ScrollHandle,
}

impl ZoomSelect {
    pub(super) fn new(window: &mut Window, cx: &mut Context<Shell>) -> Self {
        let group = cx.focus_handle();
        let blur = cx.on_focus_out(&group, window, |shell, _, window, cx| {
            if let Some(menu) = &shell.zoom.menu
                && !shell.zoom.trigger.is_focused(window)
                && !menu.focus.iter().any(|focus| focus.is_focused(window))
            {
                shell.zoom.menu = None;
                cx.notify();
            }
        });
        Self {
            trigger: cx.focus_handle(),
            group,
            bounds: Rc::new(Cell::new(Bounds::default())),
            menu: None,
            _blur: blur,
        }
    }
}

impl Shell {
    pub(super) fn toggle_zoom_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.zoom.menu.is_some() {
            self.close_zoom_menu(window, cx);
            return;
        }
        let Some(reader) = self.active().filter(|reader| reader.read(cx).is_loaded()) else {
            return;
        };
        let index = usize::from((reader.read(cx).zoom_percent().clamp(100, 200) - 100) / 10);
        self.zoom.menu = Some(ZoomMenu {
            focus: std::array::from_fn(|_| cx.focus_handle()),
            scroll: ScrollHandle::new(),
        });
        self.focus_zoom_choice(index, window, cx);
    }

    pub(super) fn close_zoom_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.zoom.menu = None;
        window.focus(&self.zoom.trigger);
        cx.notify();
    }

    pub(super) fn focus_zoom_choice(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(menu) = &self.zoom.menu {
            let index = index.min(10);
            menu.scroll.scroll_to_item(index);
            window.focus(&menu.focus[index]);
            cx.notify();
        }
    }

    fn step_zoom_choice(&mut self, backwards: bool, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(menu) = &self.zoom.menu {
            let index = menu
                .focus
                .iter()
                .position(|focus| focus.is_focused(window))
                .unwrap_or(0);
            self.focus_zoom_choice(
                if backwards {
                    index.saturating_sub(1)
                } else {
                    index + 1
                },
                window,
                cx,
            );
        }
    }

    pub(super) fn choose_zoom(&mut self, zoom: u16, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(reader) = self.active() {
            reader.update(cx, |reader, cx| reader.set_zoom(zoom, window, cx));
        }
        self.close_zoom_menu(window, cx);
    }
}
