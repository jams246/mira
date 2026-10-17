mod empty;
mod loading;
mod navigation;

use std::{rc::Rc, sync::Arc};

use crate::{
    appearance::Palette,
    code_cache::{self, CodeCache},
    content_paint::ContentPaint,
    file::LoadedDocument,
    flow_cache::{self, FlowCache},
    image_cache::{CacheHandle, ImageCache},
    reader,
    render_context::RenderContext,
    render_table::TableScrolls,
    selection::SelectionHandle,
    text_cache::{self, TextCache},
    timing::Timing,
};
use gpui::{
    App, Context, FocusHandle, KeyBinding, ListAlignment, ListOffset, ListState, MouseButton,
    Subscription, Task, Window, actions, div, prelude::*, px,
};
use mira::{
    document::BlockKind,
    links::{self, LinkTarget},
};

actions!(mira, [Open, Copy, SelectAll, Top, Bottom, PageUp, PageDown]);

pub(crate) fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("ctrl-o", Open, Some("Mira")),
        KeyBinding::new("ctrl-c", Copy, Some("Mira")),
        KeyBinding::new("ctrl-a", SelectAll, Some("Mira")),
        KeyBinding::new("home", Top, Some("Mira")),
        KeyBinding::new("end", Bottom, Some("Mira")),
        KeyBinding::new("ctrl-home", Top, Some("Mira")),
        KeyBinding::new("ctrl-end", Bottom, Some("Mira")),
        KeyBinding::new("pageup", PageUp, Some("Mira")),
        KeyBinding::new("pagedown", PageDown, Some("Mira")),
    ]);
}

pub(crate) struct Viewer {
    loaded: Option<Arc<LoadedDocument>>,
    message: String,
    generation: u64,
    load_task: Option<Task<()>>,
    focus: FocusHandle,
    _appearance: Subscription,
    timing: Timing,
    list: ListState,
    follow_bottom: bool,
    pending_anchor: Option<String>,
    link_error: Option<String>,
    content_paint: ContentPaint,
    selection: SelectionHandle,
    table_scrolls: TableScrolls,
    images: Option<CacheHandle>,
    texts: Option<text_cache::CacheHandle>,
    codes: Option<code_cache::CacheHandle>,
    flows: Option<flow_cache::CacheHandle>,
}

impl Viewer {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>, timing: Timing) -> Self {
        let focus = cx.focus_handle();
        window.focus(&focus);
        let appearance = cx.observe_window_appearance(window, |_, _, cx| cx.notify());
        let list = ListState::new(0, ListAlignment::Top, px(500.));
        list.set_scroll_handler(cx.listener(|viewer, _, _, _| viewer.follow_bottom = false));
        Self {
            loaded: None,
            message: "Open a Markdown file to start reading.".into(),
            generation: 0,
            load_task: None,
            focus,
            _appearance: appearance,
            timing,
            list,
            follow_bottom: false,
            pending_anchor: None,
            link_error: None,
            content_paint: ContentPaint::new(timing),
            selection: SelectionHandle::default(),
            table_scrolls: TableScrolls::default(),
            images: None,
            texts: None,
            codes: None,
            flows: None,
        }
    }

    fn activate_link(&mut self, target: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(loaded) = &self.loaded else {
            return;
        };
        let Some(target) = loaded.document.targets.get(target) else {
            return;
        };
        self.link_error = None;
        match links::resolve(&loaded.path, target) {
            Ok(LinkTarget::Web(url)) => cx.open_url(&url),
            Ok(LinkTarget::Document { path, anchor }) => {
                self.load(path, window, cx);
                self.pending_anchor = anchor;
            }
            Ok(LinkTarget::Anchor(anchor)) => self.jump_to_anchor(&anchor),
            Err(error) => self.link_error = Some(error),
        }
        cx.notify();
    }

    fn jump_to_anchor(&mut self, anchor: &str) {
        self.follow_bottom = false;
        let Some(loaded) = &self.loaded else {
            return;
        };
        let index = if anchor.is_empty() {
            Some(0)
        } else {
            loaded.document.blocks.iter().position(|block| {
                let BlockKind::Heading { runs, .. } = &block.kind else {
                    return false;
                };
                let text: String = runs
                    .iter()
                    .map(|run| run.text.as_str(&loaded.document.source))
                    .collect();
                links::heading_slug(&text) == anchor
            })
        };
        match index {
            Some(item_ix) => self.list.scroll_to(ListOffset {
                item_ix,
                offset_in_item: px(0.),
            }),
            None => self.link_error = Some(format!("Heading not found: #{anchor}")),
        }
    }
}

impl Render for Viewer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::for_window(window);
        if self.follow_bottom {
            self.list.scroll_to(ListOffset {
                item_ix: self.list.item_count(),
                offset_in_item: px(0.),
            });
        }
        let root = div()
            .relative()
            .key_context("Mira")
            .track_focus(&self.focus)
            .on_action(cx.listener(Self::open))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::top))
            .on_action(cx.listener(Self::bottom))
            .on_action(cx.listener(Self::page_up))
            .on_action(cx.listener(Self::page_down))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|viewer, _, _, _| viewer.selection.borrow_mut().finish()),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|viewer, _, _, _| viewer.selection.borrow_mut().finish()),
            )
            .size_full()
            .flex()
            .flex_col()
            .bg(palette.background)
            .text_color(palette.text)
            .font_family("Segoe UI");
        if let Some(loaded) = &self.loaded {
            let images = self
                .images
                .get_or_insert_with(|| ImageCache::new(Arc::clone(loaded), window.current_view()));
            images.borrow_mut().begin_frame();
            let texts = self
                .texts
                .get_or_insert_with(|| TextCache::new(Arc::clone(loaded), window.current_view()));
            let codes = self
                .codes
                .get_or_insert_with(|| CodeCache::new(Arc::clone(loaded), window.current_view()));
            let flows = self
                .flows
                .get_or_insert_with(|| FlowCache::new(Arc::clone(loaded), window.current_view()));
            let activate_link = Rc::new(cx.processor(Self::activate_link));
            return root
                .when_some(self.link_error.clone(), |root, message| {
                    root.child(div().p_3().text_color(palette.error).child(message))
                })
                .when(loaded.document.blocks.is_empty(), |root| {
                    root.child(
                        self.content_paint.wrap(
                            div()
                                .p_8()
                                .child("This Markdown file is empty.")
                                .into_any_element(),
                        ),
                    )
                })
                .child(reader::viewport(
                    Arc::clone(loaded),
                    self.list.clone(),
                    RenderContext {
                        loaded: Arc::clone(loaded),
                        links: activate_link,
                        selection: Rc::clone(&self.selection),
                        tables: Rc::clone(&self.table_scrolls),
                        images: Rc::clone(images),
                        texts: Rc::clone(texts),
                        codes: Rc::clone(codes),
                        flows: Rc::clone(flows),
                        cell: None,
                        paint: self.content_paint.clone(),
                        owner: window.current_view(),
                        palette,
                        width: (f32::from(window.viewport_size().width) - 64.).max(1.),
                    },
                ));
        }
        self.empty(root, palette, window, cx)
    }
}
