mod actions;
mod opening;
mod tabs;
mod toolbar;
mod view;

use std::{collections::VecDeque, path::PathBuf};

use gpui::{
    App, Context, Entity, FocusHandle, KeyBinding, ScrollHandle, Subscription, Task, Window,
    actions,
};

use crate::{timing::Timing, viewer::Viewer};
use tabs::Tabs;

actions!(
    mira_window,
    [NextTab, PreviousTab, CloseTab, FocusNext, FocusPrevious]
);

pub(crate) fn bind_keys(cx: &mut App) {
    actions::bind_keys(cx);
    cx.bind_keys([
        KeyBinding::new("ctrl-o", crate::viewer::Open, Some("MiraWindow")),
        KeyBinding::new("ctrl-tab", NextTab, Some("MiraWindow")),
        KeyBinding::new("ctrl-shift-tab", PreviousTab, Some("MiraWindow")),
        KeyBinding::new("ctrl-w", CloseTab, Some("MiraWindow")),
        KeyBinding::new("tab", FocusNext, Some("MiraWindow")),
        KeyBinding::new("shift-tab", FocusPrevious, Some("MiraWindow")),
    ]);
}

struct Tab {
    id: u64,
    path: PathBuf,
    reader: Entity<Viewer>,
    _observation: Subscription,
}

struct OpenRequest {
    path: PathBuf,
    anchor: Option<String>,
    prepared: Option<Task<std::io::Result<crate::file::LoadedDocument>>>,
}

pub(crate) struct Shell {
    tabs: Vec<Tab>,
    order: Tabs,
    opening: VecDeque<Vec<OpenRequest>>,
    opening_task: Option<Task<()>>,
    focus: FocusHandle,
    strip_scroll: ScrollHandle,
    timing: Timing,
    message: Option<String>,
    _appearance: Subscription,
}

impl Shell {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>, timing: Timing) -> Self {
        let focus = cx.focus_handle();
        window.focus(&focus);
        let appearance = cx.observe_window_appearance(window, |_, _, cx| cx.notify());
        Self {
            tabs: Vec::new(),
            order: Tabs::default(),
            opening: VecDeque::new(),
            opening_task: None,
            focus,
            strip_scroll: ScrollHandle::new(),
            timing,
            message: None,
            _appearance: appearance,
        }
    }

    fn active(&self) -> Option<&Entity<Viewer>> {
        let id = self.order.active?;
        self.tabs
            .iter()
            .find(|tab| tab.id == id)
            .map(|tab| &tab.reader)
    }

    fn reader_changed(
        &mut self,
        id: u64,
        reader: &Entity<Viewer>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(path) = reader.read(cx).loaded_path().map(PathBuf::from) {
            if let Some(existing) = self.order.rekey(id, path.clone()) {
                let active = self.order.active == Some(id);
                self.close(id, window, cx);
                if active {
                    self.select(existing, window, cx);
                }
            } else if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == id) {
                tab.path = path;
            }
        }
        cx.notify();
    }

    fn select(&mut self, id: u64, window: &mut Window, cx: &mut Context<Self>) {
        if self.order.active != Some(id) {
            if let Some(reader) = self.active() {
                reader.update(cx, |reader, cx| reader.deactivate(window, cx));
            }
            self.order.active = Some(id);
        }
        if let Some(reader) = self.active() {
            reader.update(cx, |reader, cx| reader.activate(window, cx));
        }
        if let Some(index) = self.order.ids.iter().position(|&other| other == id) {
            self.strip_scroll.scroll_to_item(index);
        }
        cx.notify();
    }

    fn cycle(&mut self, backwards: bool, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.order.neighbor(backwards) {
            self.select(id, window, cx);
        }
    }

    fn close(&mut self, id: u64, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(index) = self.tabs.iter().position(|tab| tab.id == id) {
            self.tabs[index]
                .reader
                .update(cx, |reader, cx| reader.deactivate(window, cx));
            self.tabs.remove(index);
        }
        self.order.close(id);
        if let Some(next) = self.order.active {
            self.select(next, window, cx);
        } else {
            window.focus(&self.focus);
        }
        cx.notify();
    }
}
