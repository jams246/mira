use gpui::{AppContext, AsyncApp, Keystroke, WindowHandle};

use super::{Shell, until};

pub(super) async fn run(window: WindowHandle<Shell>, cx: &mut AsyncApp) {
    let (active, other, original_zoom, other_zoom) = window
        .update(cx, |shell, window, cx| {
            let active = shell.order.active.unwrap();
            let other = shell
                .tabs
                .iter()
                .find(|tab| tab.id != active && tab.reader.read(cx).is_loaded())
                .unwrap();
            let other_id = other.id;
            let other_zoom = other.reader.read(cx).zoom_percent();
            let original_zoom = shell.active().unwrap().read(cx).zoom_percent();
            for level in (100..=200).step_by(10) {
                shell.toggle_zoom_menu(window, cx);
                assert_eq!(shell.zoom.menu.as_ref().unwrap().focus.len(), 11);
                shell.choose_zoom(level, window, cx);
                assert_eq!(shell.active().unwrap().read(cx).zoom_percent(), level);
                assert!(shell.zoom.menu.is_none());
            }
            shell.choose_zoom(150, window, cx);
            shell.select(other_id, window, cx);
            assert_eq!(shell.active().unwrap().read(cx).zoom_percent(), other_zoom);
            shell.select(active, window, cx);
            assert_eq!(shell.active().unwrap().read(cx).zoom_percent(), 150);
            shell.toggle_zoom_menu(window, cx);
            assert!(shell.zoom.menu.as_ref().unwrap().focus[5].is_focused(window));
            (active, other_id, original_zoom, other_zoom)
        })
        .expect("native zoom choices and per-tab state");

    for (key, index) in [
        ("down", 6),
        ("up", 5),
        ("home", 0),
        ("up", 0),
        ("end", 10),
        ("down", 10),
    ] {
        press(window, key, cx);
        until(
            cx,
            |cx| {
                window
                    .update(cx, |shell, window, _| {
                        shell
                            .zoom
                            .menu
                            .as_ref()
                            .is_some_and(|menu| menu.focus[index].is_focused(window))
                    })
                    .unwrap()
            },
            "zoom option keyboard focus",
        )
        .await;
    }
    press(window, "enter", cx);
    until_closed(window, cx).await;
    window
        .update(cx, |shell, window, cx| {
            assert_eq!(shell.active().unwrap().read(cx).zoom_percent(), 200);
            shell.toggle_zoom_menu(window, cx);
        })
        .unwrap();
    press(window, "home", cx);
    press(window, "escape", cx);
    until_closed(window, cx).await;
    window
        .update(cx, |shell, window, cx| {
            assert_eq!(shell.active().unwrap().read(cx).zoom_percent(), 200);
            shell.select(other, window, cx);
            assert_eq!(shell.active().unwrap().read(cx).zoom_percent(), other_zoom);
            if let Some(failed) = shell
                .tabs
                .iter()
                .find(|tab| !tab.reader.read(cx).is_loaded())
                .map(|tab| tab.id)
            {
                shell.select(failed, window, cx);
                shell.toggle_zoom_menu(window, cx);
                assert!(shell.zoom.menu.is_none());
            }
            shell.select(active, window, cx);
            shell.choose_zoom(original_zoom, window, cx);
        })
        .expect("cancel zoom and restore native tab state");
}

fn press(window: WindowHandle<Shell>, key: &str, cx: &mut AsyncApp) {
    cx.update_window(window.into(), |_, window, cx| {
        window.draw(cx).clear();
        assert!(
            window.dispatch_keystroke(Keystroke::parse(key).unwrap(), cx),
            "unhandled zoom key: {key}"
        );
    })
    .expect("native zoom key dispatch");
}

async fn until_closed(window: WindowHandle<Shell>, cx: &mut AsyncApp) {
    until(
        cx,
        |cx| {
            window
                .update(cx, |shell, _, _| shell.zoom.menu.is_none())
                .unwrap()
        },
        "zoom menu keyboard dismissal",
    )
    .await;
}
