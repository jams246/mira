use std::{rc::Rc, sync::Arc};

use gpui::{AsyncApp, ListOffset, WindowHandle, px};
use mira::reading_state::Location;

use super::{Viewer, support};

pub(super) async fn run(window: WindowHandle<Viewer>, cx: &mut AsyncApp) {
    let original = "# Retained\n\nFirst paragraph.\n\n# Removed\n\nLast café 😀 paragraph.\n";
    let path = support::fixture("reload.md", original);
    support::load(window, path.clone(), cx).await;
    let (loaded, selected) = window
        .update(cx, |reader, window, cx| {
            reader.contents = true;
            reader.zoom_in(window, cx);
            reader
                .selection
                .borrow_mut()
                .select_all(&reader.loaded.as_ref().unwrap().document);
            reader.navigate_fragment("removed", false, window, cx);
            reader.reveal.borrow_mut().reading = Location {
                block: 3,
                byte: 12,
                offset: -2.,
                cell: None,
            };
            reader.list.scroll_to(ListOffset {
                item_ix: 3,
                offset_in_item: px(0.),
            });
            (
                Arc::clone(reader.loaded.as_ref().unwrap()),
                Rc::clone(&reader.selection),
            )
        })
        .unwrap();
    std::fs::write(&path, [0xff, 0xfe]).unwrap();
    window
        .update(cx, |reader, window, cx| reader.reload(window, cx))
        .unwrap();
    support::until(
        cx,
        |cx| {
            window
                .update(cx, |reader, _, _| support::settled(reader))
                .unwrap()
        },
        "failed reload",
    )
    .await;
    window
        .update(cx, |reader, _, _| {
            assert!(Arc::ptr_eq(reader.loaded.as_ref().unwrap(), &loaded));
            assert!(Rc::ptr_eq(&reader.selection, &selected));
            assert!(reader.can_back());
            assert!(
                reader
                    .link_error
                    .as_ref()
                    .unwrap()
                    .starts_with("Cannot open ")
            );
            assert_eq!(reader.location().block, 3);
        })
        .unwrap();
    std::fs::write(&path, original).unwrap();
    window
        .update(cx, |reader, window, cx| reader.reload(window, cx))
        .unwrap();
    support::until(
        cx,
        |cx| {
            window
                .update(cx, |reader, _, _| support::settled(reader))
                .unwrap()
        },
        "unchanged reload retry",
    )
    .await;
    window
        .update(cx, |reader, _, _| {
            assert_eq!(reader.loaded.as_ref().unwrap().document.source, original);
            assert!(Rc::ptr_eq(&reader.selection, &selected));
            assert!(reader.can_back());
            assert_eq!(reader.zoom_percent(), 110);
            assert!(reader.contents && reader.link_error.is_none());
            assert_eq!(reader.reveal.borrow().reading.block, 3);
        })
        .unwrap();
    std::fs::write(&path, "Short café.").unwrap();
    window
        .update(cx, |reader, window, cx| reader.reload(window, cx))
        .unwrap();
    support::until(
        cx,
        |cx| {
            window
                .update(cx, |reader, _, _| support::settled(reader))
                .unwrap()
        },
        "changed shortened reload",
    )
    .await;
    window
        .update(cx, |reader, _, _| {
            assert_eq!(
                reader.loaded.as_ref().unwrap().document.source,
                "Short café."
            );
            assert!(!Rc::ptr_eq(&reader.selection, &selected));
            assert!(!reader.can_back() && !reader.can_forward());
            assert!(!reader.contents && !reader.has_headings());
            assert_eq!(reader.zoom_percent(), 110);
            let position = reader.reveal.borrow().reading;
            assert_eq!(position.block, 0);
            assert!("Short café.".is_char_boundary(position.byte));
            assert!(position.byte <= "Short café.".len());
        })
        .unwrap();
    assert_eq!(std::fs::read_to_string(path).unwrap(), "Short café.");
}
