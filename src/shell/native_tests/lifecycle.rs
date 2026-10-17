use gpui::{AppContext, AsyncApp, WindowHandle};

use super::{Shell, support};

pub(super) fn run(window: WindowHandle<Shell>, cx: &mut AsyncApp) {
    draw(window, cx);
    let destinations = window
        .update(cx, |shell, _, _| [shell.tabs[0].id, shell.tabs[1].id])
        .unwrap();
    for destination in destinations.into_iter().cycle().take(6) {
        let retained = window
            .update(cx, |shell, window, cx| {
                let retained = support::track_caches(shell.active().unwrap().read(cx));
                shell.select(destination, window, cx);
                retained
            })
            .unwrap();
        draw(window, cx);
        draw(window, cx);
        assert_eq!(
            retained(),
            [false; 4],
            "inactive caches retained: text/code/flow/image"
        );
    }
    let (retained, closed) = window
        .update(cx, |shell, window, cx| {
            let reader = shell.active().unwrap();
            let retained = support::track_caches(reader.read(cx));
            let closed = reader.downgrade();
            shell.close(shell.order.active.unwrap(), window, cx);
            (retained, closed)
        })
        .unwrap();
    draw(window, cx);
    draw(window, cx);
    assert_eq!(
        retained(),
        [false; 4],
        "closed caches retained: text/code/flow/image"
    );
    assert!(
        closed.upgrade().is_none(),
        "closed rendered reader retained"
    );
}

fn draw(window: WindowHandle<Shell>, cx: &mut AsyncApp) {
    cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear())
        .expect("render shell lifecycle frame");
}
