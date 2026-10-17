mod drop;
mod lifecycle;
mod navigation;

use std::sync::{Arc, atomic::Ordering};

use gpui::{AppContext, AsyncApp};

use super::Shell;
use crate::{
    timing::Timing,
    viewer::native_test_support::{self as support, gated, until},
};

pub(crate) async fn run(cx: &mut AsyncApp) {
    let a = support::fixture("first-é-😀.md", "# First\n\nFirst content.\n");
    let b = support::fixture("second.md", "# Second\n\nSecond content.\n");
    let missing = a.with_file_name("missing-native.md");
    if let Err(error) = std::fs::remove_file(&missing) {
        assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    }
    let alias = a.parent().unwrap().join(".").join(a.file_name().unwrap());
    let window = cx
        .open_window(support::window_options(), |window, cx| {
            cx.new(|cx| Shell::new(window, cx, Timing::start()))
        })
        .expect("hidden shell window");
    support::realize(window, cx).await;
    let prepared = a.clone();
    let (task, gate) = gated(cx.background_executor(), move || {
        crate::file::load(&prepared)
    });
    window
        .update(cx, |shell, window, cx| {
            shell.open_initial_paths(
                vec![a.clone(), alias.clone(), b.clone(), missing.clone()],
                task,
                window,
                cx,
            );
            shell.open_paths(vec![alias], window, cx);
        })
        .expect("ordered prepared CLI paths and concurrent duplicate");
    until(
        cx,
        |cx| {
            gate.ready.load(Ordering::Acquire)
                && window
                    .update(cx, |shell, _, cx| {
                        shell.tabs.len() == 3
                            && shell.tabs[1].reader.read(cx).is_loaded()
                            && support::failed(shell.tabs[2].reader.read(cx))
                            && shell.opening_task.is_none()
                    })
                    .unwrap()
        },
        "out-of-order real file completion",
    )
    .await;
    window
        .update(cx, |shell, _, cx| {
            assert_eq!(
                shell.order.ids,
                shell.tabs.iter().map(|tab| tab.id).collect::<Vec<_>>()
            );
            assert_eq!(shell.order.active, Some(shell.tabs[0].id));
            assert!(!shell.tabs[0].reader.read(cx).is_loaded());
            assert_eq!(
                shell.tabs[1].reader.read(cx).loaded_path(),
                Some(b.canonicalize().unwrap().as_path())
            );
            assert_eq!(
                shell.tabs[2].reader.read(cx).requested_path(),
                Some(missing.as_path())
            );
        })
        .unwrap();
    drop(gate);
    until(
        cx,
        |cx| {
            window
                .update(cx, |shell, _, cx| shell.tabs[0].reader.read(cx).is_loaded())
                .unwrap()
        },
        "prepared first file publication",
    )
    .await;
    window
        .update(cx, |shell, window, cx| {
            assert_eq!(
                shell.tabs[0].reader.read(cx).loaded_path(),
                Some(a.canonicalize().unwrap().as_path())
            );
            let active = shell.order.active;
            let order = shell.order.ids.clone();
            shell.open_paths(Vec::new(), window, cx);
            assert_eq!(shell.order.active, active);
            assert_eq!(shell.order.ids, order);
            assert!(shell.opening_task.is_none());
        })
        .unwrap();
    let c = support::fixture("close-loading.md", "# Closed pending file\n\nText.\n");
    let path = c.clone();
    let (task, gate) = gated(cx.background_executor(), move || crate::file::load(&path));
    window
        .update(cx, |shell, window, cx| {
            shell.open_initial_paths(vec![c.clone()], task, window, cx);
        })
        .unwrap();
    until(
        cx,
        |cx| {
            gate.ready.load(Ordering::Acquire)
                && window
                    .update(cx, |shell, _, _| shell.tabs.len() == 4)
                    .unwrap()
        },
        "loading tab created",
    )
    .await;
    let (closed, weak) = window
        .update(cx, |shell, window, cx| {
            let tab = &shell.tabs[3];
            let id = tab.id;
            let weak = tab.reader.downgrade();
            assert!(!tab.reader.read(cx).is_loaded());
            shell.close(id, window, cx);
            (id, weak)
        })
        .unwrap();
    until(
        cx,
        |_| weak.upgrade().is_none(),
        "closed loading reader released",
    )
    .await;
    let done = Arc::clone(&gate.done);
    drop(gate);
    until(
        cx,
        |_| done.load(Ordering::Acquire),
        "closed file worker completion",
    )
    .await;
    window
        .update(cx, |shell, window, cx| {
            assert_eq!(shell.tabs.len(), 3);
            assert!(!shell.order.ids.contains(&closed));
            shell.open_paths(vec![c.clone()], window, cx);
        })
        .unwrap();
    until(
        cx,
        |cx| {
            window
                .update(cx, |shell, _, cx| {
                    shell.tabs.len() == 4 && shell.tabs[3].reader.read(cx).is_loaded()
                })
                .unwrap()
        },
        "reopening closed pending path",
    )
    .await;
    window
        .update(cx, |shell, _, cx| {
            assert!(shell.tabs[3].id > closed);
            assert_eq!(
                shell.tabs[3].reader.read(cx).loaded_path(),
                Some(c.canonicalize().unwrap().as_path())
            );
            assert_eq!(shell.order.active, Some(shell.tabs[3].id));
        })
        .unwrap();
    drop::run(window, cx).await;
    navigation::run(window, cx).await;
    lifecycle::run(window, cx);
    window
        .update(cx, |_, window, _| window.remove_window())
        .unwrap();
}
