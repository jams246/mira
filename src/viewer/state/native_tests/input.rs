use gpui::{AsyncApp, EntityInputHandler, WindowHandle};

use super::Viewer;

pub(super) fn run(window: WindowHandle<Viewer>, cx: &mut AsyncApp) {
    window
        .update(cx, |reader, window, cx| {
            reader.find.input.update(cx, |input, cx| {
                input.select_all(window, cx);
                input.replace_text_in_range(None, "", window, cx);
                input.replace_and_mark_text_in_range(None, "é😀日", Some(1..3), window, cx);
                assert_eq!(input.query(), "é😀日");
                assert_eq!(input.marked_text_range(window, cx), Some(0..4));
                assert_eq!(
                    input.selected_text_range(false, window, cx).unwrap().range,
                    1..3
                );
                input.replace_and_mark_text_in_range(None, "é😀日本", Some(5..5), window, cx);
                assert_eq!(input.query(), "é😀日本");
                assert_eq!(input.marked_text_range(window, cx), Some(0..5));
                input.replace_text_in_range(None, "café😀日本", window, cx);
                assert_eq!(input.marked_text_range(window, cx), None);
                assert_eq!(
                    input.selected_text_range(false, window, cx).unwrap().range,
                    8..8
                );
                input.replace_text_in_range(Some(4..6), "🔎", window, cx);
                assert_eq!(input.query(), "café🔎日本");
                input.replace_and_mark_text_in_range(Some(0..4), "喫茶", Some(2..2), window, cx);
                assert_eq!(input.query(), "喫茶🔎日本");
                assert_eq!(input.marked_text_range(window, cx), Some(0..2));
                assert_eq!(
                    input.selected_text_range(false, window, cx).unwrap().range,
                    2..2
                );
                input.unmark_text(window, cx);
                assert_eq!(input.marked_text_range(window, cx), None);
                assert_eq!(input.query(), "喫茶🔎日本");
                let mut actual = None;
                assert_eq!(
                    input
                        .text_for_range(2..4, &mut actual, window, cx)
                        .as_deref(),
                    Some("🔎")
                );
                assert_eq!(actual, Some(2..4));
                input.select_all(window, cx);
                input.replace_text_in_range(None, "", window, cx);
            });
            reader.close_find(window, cx);
        })
        .expect("native IME composition callbacks");
}
