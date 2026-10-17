use std::{
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use gpui::{AsyncApp, WindowHandle};

use super::{
    Viewer,
    support::{self, gated, until},
};

pub(super) async fn run(window: WindowHandle<Viewer>, cx: &mut AsyncApp) {
    let old = support::fixture("old-search.md", "# Original\n\nold old café");
    let new = support::fixture("new-search.md", "# Replacement\n\nnew text");
    support::load(window, old, cx).await;
    window
        .update(cx, |reader, window, cx| {
            reader.find.visible = true;
            let loaded = Arc::clone(reader.loaded.as_ref().unwrap());
            let task = cx.background_executor().spawn(async move {
                mira::search::search(&loaded.document, "old", false, &AtomicBool::new(false))
            });
            reader.await_search(task, false, window, cx);
        })
        .unwrap();
    until(
        cx,
        |cx| {
            window
                .update(cx, |reader, _, _| {
                    !reader.find.pending && reader.find.highlights.borrow().matches.len() == 2
                })
                .unwrap()
        },
        "live search publication control",
    )
    .await;
    let gate = window
        .update(cx, |reader, window, cx| {
            reader.find.visible = true;
            let loaded = Arc::clone(reader.loaded.as_ref().unwrap());
            let (task, gate) = gated(cx.background_executor(), move || {
                let results =
                    mira::search::search(&loaded.document, "old", false, &AtomicBool::new(false));
                assert_eq!(results.len(), 2);
                results
            });
            reader.await_search(task, true, window, cx);
            gate
        })
        .unwrap();
    until(
        cx,
        |_| gate.ready.load(Ordering::Acquire),
        "old query computed before reload",
    )
    .await;
    support::load(window, new.clone(), cx).await;
    let done = Arc::clone(&gate.done);
    drop(gate);
    until(
        cx,
        |_| done.load(Ordering::Acquire),
        "stale search worker completion",
    )
    .await;
    cx.background_executor()
        .timer(std::time::Duration::from_millis(30))
        .await;
    window
        .update(cx, |reader, _, _| {
            assert_eq!(
                reader.loaded.as_ref().unwrap().path,
                new.canonicalize().unwrap()
            );
            assert!(reader.find.highlights.borrow().matches.is_empty());
            assert!(!reader.find.pending);
        })
        .unwrap();
    let gate = window
        .update(cx, |reader, window, cx| {
            reader.find.visible = true;
            let loaded = Arc::clone(reader.loaded.as_ref().unwrap());
            let (task, gate) = gated(cx.background_executor(), move || {
                mira::search::search(&loaded.document, "new", false, &AtomicBool::new(false))
            });
            reader.await_search(task, true, window, cx);
            gate
        })
        .unwrap();
    until(
        cx,
        |_| gate.ready.load(Ordering::Acquire),
        "query computed before Find closes",
    )
    .await;
    window
        .update(cx, |reader, window, cx| reader.close_find(window, cx))
        .unwrap();
    let done = Arc::clone(&gate.done);
    drop(gate);
    until(
        cx,
        |_| done.load(Ordering::Acquire),
        "closed Find worker completion",
    )
    .await;
    cx.background_executor()
        .timer(std::time::Duration::from_millis(30))
        .await;
    window
        .update(cx, |reader, _, _| {
            assert!(!reader.find.visible && !reader.find.highlights.borrow().enabled);
            assert!(!reader.find.pending);
            assert!(reader.find.highlights.borrow().matches.is_empty());
        })
        .unwrap();
    let (gate, weak, highlights) = window
        .update(cx, |reader, window, cx| {
            reader.find.visible = true;
            let loaded = Arc::clone(reader.loaded.as_ref().unwrap());
            let (task, gate) = gated(cx.background_executor(), move || {
                mira::search::search(&loaded.document, "new", false, &AtomicBool::new(false))
            });
            reader.await_search(task, true, window, cx);
            (
                gate,
                cx.entity().downgrade(),
                Rc::clone(&reader.find.highlights),
            )
        })
        .unwrap();
    until(
        cx,
        |_| gate.ready.load(Ordering::Acquire),
        "query computed before entity closes",
    )
    .await;
    window
        .update(cx, |_, window, _| window.remove_window())
        .unwrap();
    until(cx, |_| weak.upgrade().is_none(), "reader entity released").await;
    let done = Arc::clone(&gate.done);
    drop(gate);
    until(
        cx,
        |_| done.load(Ordering::Acquire),
        "closed entity worker completion",
    )
    .await;
    assert!(highlights.borrow().matches.is_empty());
}
