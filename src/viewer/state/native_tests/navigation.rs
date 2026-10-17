use gpui::{AppContext, AsyncApp, ListOffset, WindowHandle, px};
use mira::reading_state::Location;

use super::{Viewer, support};
use crate::viewer::{Back, Forward};

pub(super) async fn run(window: WindowHandle<Viewer>, cx: &mut AsyncApp) {
    let source = format!(
        "# First\n\nFirst paragraph.\n\n{}# Second\n\n{}",
        "Spacer paragraph.\n\n".repeat(60),
        "Following paragraph.\n\n".repeat(60)
    );
    support::load(window, support::fixture("navigation.md", &source), cx).await;
    let destination = window
        .update(cx, |reader, window, cx| {
            reader.reveal.borrow_mut().reading = Location::default();
            reader.list.scroll_to(ListOffset {
                item_ix: 0,
                offset_in_item: px(0.),
            });
            reader.navigate_fragment("second", false, window, cx);
            assert!(reader.can_back() && !reader.can_forward());
            let block = reader.headings[1].block;
            reader.reveal.borrow_mut().reading = Location {
                block,
                ..Location::default()
            };
            reader.list.scroll_to(ListOffset {
                item_ix: block,
                offset_in_item: px(0.),
            });
            window.focus(&reader.focus);
            block
        })
        .expect("prepare native Back action");
    cx.update_window(window.into(), |_, window, cx| {
        window.draw(cx).clear();
        window.dispatch_action(Box::new(Back), cx);
    })
    .expect("dispatch native Back action");
    support::until(
        cx,
        |cx| {
            window
                .update(cx, |reader, _, _| {
                    !reader.can_back() && reader.can_forward()
                })
                .unwrap()
        },
        "Back dispatch",
    )
    .await;
    window
        .update(cx, |reader, window, cx| {
            let reveal = reader.reveal.borrow();
            assert_eq!(reveal.target.unwrap_or(reveal.reading).block, 0);
            drop(reveal);
            reader.reveal.borrow_mut().reading = Location::default();
            reader.list.scroll_to(ListOffset {
                item_ix: 0,
                offset_in_item: px(0.),
            });
            window.dispatch_action(Box::new(Forward), cx);
        })
        .expect("dispatch native Forward action");
    support::until(
        cx,
        |cx| {
            window
                .update(cx, |reader, _, _| {
                    reader.can_back() && !reader.can_forward()
                })
                .unwrap()
        },
        "Forward dispatch",
    )
    .await;
    window
        .update(cx, |reader, _, _| {
            let reveal = reader.reveal.borrow();
            assert_eq!(reveal.target.unwrap_or(reveal.reading).block, destination);
        })
        .unwrap();
}
