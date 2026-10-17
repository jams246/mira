use gpui::{Action, App, Context, Div, KeyBinding, Window, prelude::*};

use super::Shell;
use crate::viewer::{
    Back, Bottom, CloseFind, Contents, Copy, Find, FindNext, FindPrevious, Forward, PageDown,
    PageUp, Reload, SelectAll, Top, Viewer, ZoomIn, ZoomOut, ZoomReset,
};

pub(super) fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("ctrl-f", Find, Some("MiraWindow")),
        KeyBinding::new("f3", FindNext, Some("MiraWindow")),
        KeyBinding::new("shift-f3", FindPrevious, Some("MiraWindow")),
        KeyBinding::new("escape", CloseFind, Some("MiraWindow")),
        KeyBinding::new("ctrl-shift-o", Contents, Some("MiraWindow")),
        KeyBinding::new("alt-left", Back, Some("MiraWindow")),
        KeyBinding::new("alt-right", Forward, Some("MiraWindow")),
        KeyBinding::new("f5", Reload, Some("MiraWindow")),
        KeyBinding::new("ctrl-=", ZoomIn, Some("MiraWindow")),
        KeyBinding::new("ctrl-+", ZoomIn, Some("MiraWindow")),
        KeyBinding::new("ctrl-shift-=", ZoomIn, Some("MiraWindow")),
        KeyBinding::new("ctrl--", ZoomOut, Some("MiraWindow")),
        KeyBinding::new("ctrl-0", ZoomReset, Some("MiraWindow")),
        KeyBinding::new("ctrl-c", Copy, Some("MiraWindow")),
        KeyBinding::new("ctrl-a", SelectAll, Some("MiraWindow")),
        KeyBinding::new("home", Top, Some("MiraWindow")),
        KeyBinding::new("end", Bottom, Some("MiraWindow")),
        KeyBinding::new("ctrl-home", Top, Some("MiraWindow")),
        KeyBinding::new("ctrl-end", Bottom, Some("MiraWindow")),
        KeyBinding::new("pageup", PageUp, Some("MiraWindow")),
        KeyBinding::new("pagedown", PageDown, Some("MiraWindow")),
    ]);
}

pub(super) fn reader_actions(root: Div, cx: &mut Context<Shell>) -> Div {
    let root = delegate::<Find>(root, Viewer::open_find, cx);
    let root = delegate::<FindNext>(root, Viewer::find_next, cx);
    let root = delegate::<FindPrevious>(root, Viewer::find_previous, cx);
    let root = delegate::<CloseFind>(root, Viewer::close_find, cx);
    let root = delegate::<Contents>(root, Viewer::toggle_contents, cx);
    let root = delegate::<Back>(root, Viewer::back, cx);
    let root = delegate::<Forward>(root, Viewer::forward, cx);
    let root = delegate::<Reload>(root, Viewer::reload, cx);
    let root = delegate::<ZoomIn>(root, Viewer::zoom_in, cx);
    let root = delegate::<ZoomOut>(root, Viewer::zoom_out, cx);
    let root = delegate::<ZoomReset>(root, Viewer::zoom_reset, cx);
    let root = delegate::<Copy>(
        root,
        |reader, window, cx| reader.copy(&Copy, window, cx),
        cx,
    );
    let root = delegate::<SelectAll>(
        root,
        |reader, window, cx| reader.select_all(&SelectAll, window, cx),
        cx,
    );
    let root = delegate::<Top>(root, |reader, window, cx| reader.top(&Top, window, cx), cx);
    let root = delegate::<Bottom>(
        root,
        |reader, window, cx| reader.bottom(&Bottom, window, cx),
        cx,
    );
    let root = delegate::<PageUp>(
        root,
        |reader, window, cx| reader.page_up(&PageUp, window, cx),
        cx,
    );
    delegate::<PageDown>(
        root,
        |reader, window, cx| reader.page_down(&PageDown, window, cx),
        cx,
    )
}

fn delegate<A: Action>(
    root: Div,
    command: fn(&mut Viewer, &mut Window, &mut Context<Viewer>),
    cx: &mut Context<Shell>,
) -> Div {
    root.on_action(cx.listener(move |shell, _: &A, window, cx| {
        if let Some(reader) = shell.active() {
            reader.update(cx, |reader, cx| command(reader, window, cx));
        }
    }))
}
