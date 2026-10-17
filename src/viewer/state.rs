use super::*;
#[cfg(all(test, windows))]
pub(crate) mod native_tests;
use gpui::Focusable;
use mira::{
    links::{self, LinkTarget},
    reading_state::Location,
};

impl Viewer {
    pub(crate) fn deactivate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.active = false;
        self.find.cancel();
        self.reveal.borrow_mut().target = None;
        self.release_caches(window, cx);
    }
    pub(crate) fn activate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.active {
            if self.find.visible {
                window.focus(&self.find.input.focus_handle(cx));
            } else {
                window.focus(&self.focus);
            }
            return;
        }
        self.active = true;
        let location = self.location();
        if let Some(loaded) = &self.loaded {
            self.list.reset(loaded.document.blocks.len());
        }
        self.jump(location, false, cx);
        if self.find.visible {
            window.focus(&self.find.input.focus_handle(cx));
            self.search(false, window, cx);
        } else {
            window.focus(&self.focus);
        }
        cx.notify();
    }
    pub(super) fn release_caches(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.texts = None;
        self.codes = None;
        self.flows = None;
        if let Some(images) = self.images.take() {
            images.borrow_mut().clear(window, cx);
        }
    }
    pub(super) fn location(&self) -> Location {
        let top = self.list.logical_scroll_top();
        let location = self.reveal.borrow().reading;
        if self
            .loaded
            .as_ref()
            .and_then(|loaded| loaded.document.blocks.get(top.item_ix))
            .is_some_and(|block| {
                matches!(
                    block.kind,
                    mira::document::BlockKind::Image { .. } | mira::document::BlockKind::Rule
                )
            })
        {
            return Location {
                block: top.item_ix,
                offset: -f32::from(top.offset_in_item),
                ..Location::default()
            };
        }
        if location.block == top.item_ix {
            location
        } else {
            Location {
                block: top.item_ix.min(self.list.item_count().saturating_sub(1)),
                ..Location::default()
            }
        }
    }
    pub(super) fn jump(&mut self, location: Location, record: bool, cx: &mut Context<Self>) {
        let timing = Timing::start();
        if record {
            self.history.jump(self.location(), location);
        }
        self.follow_bottom = false;
        self.reveal.borrow_mut().seek(location, cx);
        if self
            .loaded
            .as_ref()
            .and_then(|loaded| loaded.document.blocks.get(location.block))
            .is_some_and(|block| {
                matches!(
                    block.kind,
                    mira::document::BlockKind::Image { .. } | mira::document::BlockKind::Rule
                )
            })
        {
            self.reveal.borrow_mut().target = None;
            self.list.scroll_to(gpui::ListOffset {
                item_ix: location.block,
                offset_in_item: px((-location.offset).max(0.)),
            });
        }
        cx.notify();
        timing.mark("navigation_callback");
    }
    pub(crate) fn navigate_fragment(
        &mut self,
        anchor: &str,
        initial: bool,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.loading && self.loaded.is_none() {
            self.pending_anchor = Some(anchor.into());
            return;
        }
        let block = if anchor.is_empty() {
            Some(0)
        } else {
            self.headings
                .iter()
                .find(|h| h.fragment == anchor)
                .map(|h| h.block)
        };
        if let Some(block) = block {
            self.link_error = None;
            self.jump(
                Location {
                    block,
                    ..Location::default()
                },
                !initial,
                cx,
            );
        } else {
            self.link_error = Some(format!("Heading not found: #{anchor}"));
            cx.notify();
        }
    }
    pub(super) fn activate_link(
        &mut self,
        target: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(loaded) = &self.loaded else { return };
        let Some(target) = loaded.document.targets.get(target) else {
            return;
        };
        self.link_error = None;
        match links::resolve(&loaded.path, target) {
            Ok(LinkTarget::Web(url)) => cx.open_url(&url),
            Ok(LinkTarget::Document { path, anchor }) => {
                if let Some(open) = &self.open_handler {
                    open(path, anchor, window, cx);
                }
            }
            Ok(LinkTarget::Anchor(anchor)) => self.navigate_fragment(&anchor, false, window, cx),
            Err(error) => self.link_error = Some(error),
        }
        cx.notify();
    }
    pub(crate) fn back(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(location) = self.history.back(self.location()) {
            self.jump(location, false, cx);
        }
    }
    pub(crate) fn forward(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(location) = self.history.forward(self.location()) {
            self.jump(location, false, cx);
        }
    }
    pub(crate) fn zoom_in(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_zoom((self.zoom + 10).min(200), window, cx);
    }
    pub(crate) fn zoom_out(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_zoom(self.zoom.saturating_sub(10).max(50), window, cx);
    }
    pub(crate) fn zoom_reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_zoom(100, window, cx);
    }
    fn set_zoom(&mut self, zoom: u16, window: &mut Window, cx: &mut Context<Self>) {
        if self.zoom == zoom || self.loaded.is_none() {
            return;
        }
        let location = self.location();
        let timing = Timing::start();
        self.zoom = zoom;
        self.table_scrolls.borrow_mut().invalidate_heights();
        self.release_caches(window, cx);
        if let Some(loaded) = &self.loaded {
            self.list.reset(loaded.document.blocks.len());
        }
        self.jump(location, false, cx);
        cx.notify();
        timing.mark("zoom_callback");
    }
}
