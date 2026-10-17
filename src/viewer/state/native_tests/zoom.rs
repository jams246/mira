use std::{rc::Rc, sync::Arc};

use gpui::{AsyncApp, ListOffset, WindowHandle, px};
use mira::{reading_state::Location, search::SearchMatch};

use super::{Viewer, support};
use crate::{code_cache::CodeCache, flow_cache::FlowCache, text_cache::TextCache};

pub(super) async fn run(window: WindowHandle<Viewer>, cx: &mut AsyncApp) {
    let source = format!(
        "# Native zoom\n\nFirst café 😀 paragraph.\n\n```rust\n{}```\n",
        "let value = 42;\n".repeat(200)
    );
    let path = support::fixture("zoom.md", &source);
    support::load(window, path.clone(), cx).await;
    window
        .update(cx, |reader, window, cx| {
            let loaded = Arc::clone(reader.loaded.as_ref().expect("loaded native fixture"));
            reader.selection.borrow_mut().select_all(&loaded.document);
            let selection = Rc::clone(&reader.selection);
            let selected = selection.borrow().text(&loaded.document);
            let location = Location {
                block: 1,
                byte: 6,
                cell: None,
                offset: -3.,
            };
            reader.list.scroll_to(ListOffset {
                item_ix: 1,
                offset_in_item: px(0.),
            });
            reader.reveal.borrow_mut().reading = location;
            reader.texts = Some(TextCache::new(Arc::clone(&loaded), cx.entity_id()));
            reader.codes = Some(CodeCache::new(Arc::clone(&loaded), cx.entity_id()));
            reader.flows = Some(FlowCache::new(Arc::clone(&loaded), cx.entity_id()));
            reader.table_scrolls.borrow_mut().minimum(0, window);
            reader.table_scrolls.borrow_mut().observe(0, px(900.));
            let matches = Arc::new(vec![SearchMatch {
                block: 1,
                cell: None,
                range: 6..11,
            }]);
            reader.find.highlights.borrow_mut().matches = Arc::clone(&matches);
            reader.find.highlights.borrow_mut().active = Some(0);
            reader.set_zoom(150, window, cx);
            assert_eq!(reader.zoom_percent(), 150);
            assert!(reader.texts.is_none() && reader.codes.is_none() && reader.flows.is_none());
            assert_eq!(reader.table_scrolls.borrow_mut().minimum(0, window), px(0.));
            assert!(Arc::ptr_eq(reader.loaded.as_ref().unwrap(), &loaded));
            assert!(Rc::ptr_eq(&reader.selection, &selection));
            assert_eq!(selection.borrow().text(&loaded.document), selected);
            assert_eq!(reader.reveal.borrow().target, Some(location));
            assert!(Arc::ptr_eq(
                &reader.find.highlights.borrow().matches,
                &matches
            ));
            assert_eq!(reader.find.highlights.borrow().active, Some(0));
            for _ in 0..30 {
                reader.zoom_in(window, cx);
            }
            assert_eq!(reader.zoom_percent(), 200);
            for _ in 0..30 {
                reader.zoom_out(window, cx);
            }
            assert_eq!(reader.zoom_percent(), 50);
            reader.set_zoom(u16::MAX, window, cx);
            assert_eq!(reader.zoom_percent(), 200);
            reader.set_zoom(0, window, cx);
            assert_eq!(reader.zoom_percent(), 50);
            reader.zoom_reset(window, cx);
            assert_eq!(reader.zoom_percent(), 100);
        })
        .expect("native zoom actions");
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
}
