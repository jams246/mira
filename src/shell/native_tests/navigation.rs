use gpui::{
    AppContext, AsyncApp, MouseButton, MouseDownEvent, NavigationDirection, PlatformInput,
    WindowHandle, point, px,
};

use super::{Shell, until};

pub(super) async fn run(window: WindowHandle<Shell>, cx: &mut AsyncApp) {
    window
        .update(cx, |shell, window, cx| {
            shell.active().unwrap().update(cx, |reader, cx| {
                reader.navigate_fragment("tail", false, window, cx);
                assert!(reader.can_back() && !reader.can_forward());
            });
        })
        .unwrap();
    for direction in [NavigationDirection::Back, NavigationDirection::Forward] {
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear();
            window.dispatch_event(
                PlatformInput::MouseDown(MouseDownEvent {
                    button: MouseButton::Navigate(direction),
                    position: point(px(110.), px(200.)),
                    ..Default::default()
                }),
                cx,
            );
        })
        .expect("injected native mouse navigation");
        until(
            cx,
            |cx| {
                window
                    .update(cx, |shell, _, cx| {
                        let reader = shell.active().unwrap().read(cx);
                        match direction {
                            NavigationDirection::Back => !reader.can_back() && reader.can_forward(),
                            NavigationDirection::Forward => {
                                reader.can_back() && !reader.can_forward()
                            }
                        }
                    })
                    .unwrap()
            },
            "native mouse navigation dispatch",
        )
        .await;
    }
}
