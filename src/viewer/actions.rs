use gpui::{App, KeyBinding, actions};
actions!(
    mira,
    [
        Open,
        Copy,
        SelectAll,
        Top,
        Bottom,
        PageUp,
        PageDown,
        Find,
        FindNext,
        FindPrevious,
        CloseFind,
        Contents,
        Back,
        Forward,
        Reload,
        ZoomIn,
        ZoomOut,
        ZoomReset
    ]
);
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
        KeyBinding::new("ctrl-f", Find, Some("Mira")),
        KeyBinding::new("f3", FindNext, Some("Mira")),
        KeyBinding::new("shift-f3", FindPrevious, Some("Mira")),
        KeyBinding::new("enter", FindNext, Some("Mira")),
        KeyBinding::new("shift-enter", FindPrevious, Some("Mira")),
        KeyBinding::new("escape", CloseFind, Some("Mira")),
        KeyBinding::new("ctrl-shift-o", Contents, Some("Mira")),
        KeyBinding::new("alt-left", Back, Some("Mira")),
        KeyBinding::new("alt-right", Forward, Some("Mira")),
        KeyBinding::new("f5", Reload, Some("Mira")),
        KeyBinding::new("ctrl-=", ZoomIn, Some("Mira")),
        KeyBinding::new("ctrl-+", ZoomIn, Some("Mira")),
        KeyBinding::new("ctrl-shift-=", ZoomIn, Some("Mira")),
        KeyBinding::new("ctrl--", ZoomOut, Some("Mira")),
        KeyBinding::new("ctrl-0", ZoomReset, Some("Mira")),
    ]);
}
