use gpui::{
    AppContext, AsyncApp, ExternalPaths, FileDropEvent, PlatformInput, WindowHandle, point, px,
};

use super::{Shell, support, until};

pub(super) async fn run(window: WindowHandle<Shell>, cx: &mut AsyncApp) {
    let source = format!(
        "# Dropped first\n\n{}# Tail\n\n{}",
        "Spacer.\n\n".repeat(60),
        "Following.\n\n".repeat(60)
    );
    let first = support::fixture("dropped-é-😀.md", &source);
    let second = support::fixture("dropped-second.md", "# Dropped second\n");
    cx.update_window(window.into(), |_, window, cx| {
        window.draw(cx).clear();
        let position = point(px(110.), px(200.));
        window.dispatch_event(
            PlatformInput::FileDrop(FileDropEvent::Entered {
                position,
                paths: ExternalPaths::from(vec![first.clone(), second.clone()]),
            }),
            cx,
        );
        assert!(cx.has_active_drag());
        window.draw(cx).clear();
        let submitted = window.dispatch_event(
            PlatformInput::FileDrop(FileDropEvent::Submit { position }),
            cx,
        );
        assert!(!submitted.propagate && !cx.has_active_drag());
    })
    .expect("injected native file drop");
    until(
        cx,
        |cx| {
            window
                .update(cx, |shell, _, cx| {
                    shell.tabs.len() == 6
                        && shell.tabs[4].reader.read(cx).is_loaded()
                        && shell.tabs[5].reader.read(cx).is_loaded()
                })
                .unwrap()
        },
        "native file drop dispatch",
    )
    .await;
    window
        .update(cx, |shell, _, cx| {
            assert_eq!(shell.order.active, Some(shell.tabs[4].id));
            assert_eq!(
                shell.tabs[4].reader.read(cx).loaded_path(),
                Some(first.canonicalize().unwrap().as_path())
            );
            assert_eq!(
                shell.tabs[5].reader.read(cx).loaded_path(),
                Some(second.canonicalize().unwrap().as_path())
            );
        })
        .unwrap();
}
