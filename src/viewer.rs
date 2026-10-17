mod actions;
mod empty;
mod find;
mod find_bar;
mod loading;
mod navigation;
mod outline;
mod state;
mod view;
#[cfg(all(test, windows))]
pub(crate) use state::native_tests::support as native_test_support;

use crate::{
    code_cache,
    content_paint::ContentPaint,
    file::LoadedDocument,
    find_state::FindState,
    flow_cache,
    image_cache::CacheHandle,
    navigation_state::CodeScrolls,
    query_input::QueryInput,
    render_table::TableScrolls,
    reveal::{Reveal, RevealHandle},
    selection::SelectionHandle,
    text_cache,
    timing::Timing,
};
pub(crate) use actions::*;
use gpui::{
    App, AppContext, Context, FocusHandle, ListAlignment, ListState, Subscription, Task, Window, px,
};
use mira::{headings::Heading, reading_state::History};
use std::{
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};
pub(crate) type OpenHandler = Rc<dyn Fn(PathBuf, Option<String>, &mut Window, &mut App)>;

pub(crate) struct Viewer {
    loaded: Option<Arc<LoadedDocument>>,
    requested: Option<PathBuf>,
    message: String,
    generation: u64,
    load_task: Option<Task<()>>,
    loading: bool,
    focus: FocusHandle,
    _appearance: Subscription,
    _query: Subscription,
    timing: Timing,
    list: ListState,
    follow_bottom: bool,
    pending_anchor: Option<String>,
    link_error: Option<String>,
    content_paint: ContentPaint,
    selection: SelectionHandle,
    table_scrolls: TableScrolls,
    code_scrolls: CodeScrolls,
    images: Option<CacheHandle>,
    texts: Option<text_cache::CacheHandle>,
    codes: Option<code_cache::CacheHandle>,
    flows: Option<flow_cache::CacheHandle>,
    headings: Arc<Vec<Heading>>,
    contents: bool,
    zoom: u16,
    history: History,
    reveal: RevealHandle,
    find: FindState,
    active: bool,
    open_handler: Option<OpenHandler>,
}

impl Viewer {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>, timing: Timing) -> Self {
        let focus = cx.focus_handle();
        window.focus(&focus);
        let appearance = cx.observe_window_appearance(window, |_, _, cx| cx.notify());
        let list = ListState::new(0, ListAlignment::Top, px(500.));
        list.set_scroll_handler(cx.listener(|viewer, _, _, cx| {
            viewer.follow_bottom = false;
            cx.notify();
        }));
        let input = cx.new(QueryInput::new);
        let query = cx.subscribe_in(&input, window, |viewer, _, _, window, cx| {
            viewer.search(true, window, cx)
        });
        let reveal = Reveal::new(list.clone(), cx.entity_id());
        Self {
            loaded: None,
            requested: None,
            message: "Open a Markdown file to start reading.".into(),
            generation: 0,
            load_task: None,
            loading: false,
            focus,
            _appearance: appearance,
            _query: query,
            timing,
            list,
            follow_bottom: false,
            pending_anchor: None,
            link_error: None,
            content_paint: ContentPaint::new(timing),
            selection: Default::default(),
            table_scrolls: Default::default(),
            code_scrolls: Default::default(),
            images: None,
            texts: None,
            codes: None,
            flows: None,
            headings: Arc::new(Vec::new()),
            contents: false,
            zoom: 100,
            history: History::default(),
            reveal,
            find: FindState::new(input),
            active: true,
            open_handler: None,
        }
    }
    pub(crate) fn set_open_handler(&mut self, handler: OpenHandler) {
        self.open_handler = Some(handler);
    }
    pub(crate) fn loaded_path(&self) -> Option<&Path> {
        self.loaded.as_ref().map(|d| d.path.as_path())
    }
    pub(crate) fn requested_path(&self) -> Option<&Path> {
        self.requested.as_deref()
    }
    pub(crate) fn zoom_percent(&self) -> u16 {
        self.zoom
    }
    #[cfg(test)]
    pub(crate) fn can_back(&self) -> bool {
        self.history.can_back()
    }
    #[cfg(test)]
    pub(crate) fn can_forward(&self) -> bool {
        self.history.can_forward()
    }
    pub(crate) fn has_headings(&self) -> bool {
        !self.headings.is_empty()
    }
    pub(crate) fn is_loaded(&self) -> bool {
        self.loaded.is_some()
    }
    pub(super) fn command(
        cx: &mut Context<Self>,
        method: fn(&mut Self, &mut Window, &mut Context<Self>),
    ) -> impl Fn(&mut Window, &mut App) + use<> {
        let viewer = cx.weak_entity();
        move |window, cx| {
            let _ = viewer.update(cx, |viewer, cx| method(viewer, window, cx));
        }
    }
}
