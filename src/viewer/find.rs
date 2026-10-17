use super::*;
use gpui::Focusable;
use mira::{reading_state::Location, search};
use std::time::Duration;

impl Viewer {
    pub(crate) fn open_find(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.loaded.is_none() {
            return;
        }
        self.find.visible = true;
        self.find.highlights.borrow_mut().enabled = true;
        self.find.input.update(cx, |input, cx| input.select_all(cx));
        window.focus(&self.find.input.focus_handle(cx));
        self.search(false, window, cx);
        cx.notify();
    }

    pub(crate) fn close_find(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.find.cancel();
        self.find.visible = false;
        self.find.highlights.borrow_mut().enabled = false;
        self.reveal.borrow_mut().target = None;
        window.focus(&self.focus);
        cx.notify();
    }

    pub(super) fn search(&mut self, navigate: bool, window: &mut Window, cx: &mut Context<Self>) {
        if !navigate && !self.find.dirty {
            return;
        }
        self.find.dirty = true;
        self.find.cancel();
        if navigate {
            self.reveal.borrow_mut().target = None;
        }
        if !self.find.visible || !self.active {
            return;
        }
        let Some(loaded) = self.loaded.clone() else {
            return;
        };
        let query = self.find.input.read(cx).query().to_owned();
        {
            let mut highlights = self.find.highlights.borrow_mut();
            highlights.matches = Arc::new(Vec::new());
            highlights.active = None;
            highlights.enabled = true;
        }
        if query.is_empty() {
            self.find.dirty = false;
            cx.notify();
            return;
        }
        let cancel = Arc::clone(&self.find.cancel);
        let match_case = self.find.match_case;
        let executor = cx.background_executor().clone();
        let timer = executor.timer(Duration::from_millis(25));
        let task = executor.spawn(async move {
            timer.await;
            search::search(&loaded.document, &query, match_case, &cancel)
        });
        self.await_search(task, navigate, window, cx);
    }

    pub(super) fn await_search(
        &mut self,
        task: Task<Vec<search::SearchMatch>>,
        navigate: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.find.pending = true;
        let revision = self.find.revision;
        let generation = self.generation;
        let location = self.location();
        let search_timing = Timing::start();
        self.find.task = Some(cx.spawn_in(window, async move |viewer, cx| {
            let results = task.await;
            let _ = viewer.update_in(cx, |viewer, _, cx| {
                if viewer.generation != generation
                    || viewer.find.revision != revision
                    || !viewer.active
                {
                    return;
                }
                let first = results.partition_point(|m| {
                    (m.block, m.range.start) < (location.block, location.byte)
                });
                let timing = Timing::start();
                let active =
                    (!results.is_empty()).then_some(if first < results.len() { first } else { 0 });
                {
                    let mut highlights = viewer.find.highlights.borrow_mut();
                    highlights.matches = Arc::new(results);
                    highlights.active = active;
                }
                viewer.find.pending = false;
                viewer.find.dirty = false;
                if navigate {
                    viewer.reveal_match(cx);
                }
                cx.notify();
                timing.mark("find_publish_callback");
                search_timing.mark("find_results_ready");
            });
        }));
        cx.notify();
    }

    pub(crate) fn find_next(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.step_match(false, cx);
    }
    pub(crate) fn find_previous(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.step_match(true, cx);
    }
    fn step_match(&mut self, previous: bool, cx: &mut Context<Self>) {
        if !self.find.visible || self.find.pending {
            return;
        }
        {
            let mut highlights = self.find.highlights.borrow_mut();
            let len = highlights.matches.len();
            if len == 0 {
                return;
            }
            let current = highlights.active.unwrap_or(0);
            highlights.active = Some(if previous {
                (current + len - 1) % len
            } else {
                (current + 1) % len
            });
        }
        self.reveal_match(cx);
        cx.notify();
    }
    fn reveal_match(&mut self, cx: &mut Context<Self>) {
        let target = {
            let highlights = self.find.highlights.borrow();
            highlights
                .active
                .and_then(|i| highlights.matches.get(i))
                .map(|m| Location {
                    block: m.block,
                    cell: m.cell,
                    byte: m.range.start,
                    offset: 0.,
                })
        };
        if let Some(target) = target {
            self.jump(target, false, cx);
        }
    }
}
