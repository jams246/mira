use std::{cell::Cell, rc::Rc};

use gpui::{
    AppContext, AsyncApp, Context, Entity, IntoElement, ListOffset, Render, Window, WindowHandle,
    canvas, div, prelude::*, px,
};
use mira::reading_state::Location;

use super::{Viewer, support};
use crate::{
    code_cache::{Chunk, CodeCache, Source},
    timing::Timing,
};

pub(super) async fn run(cx: &mut AsyncApp) {
    let code = format!(
        "{}{}\n{}",
        "let value = 42;\n".repeat(20),
        "x".repeat(160),
        "let value = 42;\n".repeat(180)
    );
    let source = format!(
        "# First\n\n{}# Destination\n\n{}\n\n```rust\n{}```\n\n```text\n{}\n```\n\n{}",
        "Spacer paragraph.\n\n".repeat(40),
        "First line.  \nSecond line.  \nThird line.",
        code,
        "é".repeat(2048),
        "Following paragraph.\n\n".repeat(60)
    );
    let path = support::fixture("reveal-frame.md", &source);
    let window = cx
        .open_window(support::window_options(), |window, cx| {
            let reader = cx.new(|cx| Viewer::new(window, cx, Timing::start()));
            reader.update(cx, |reader, cx| reader.load(path, window, cx));
            cx.new(|_| Probe {
                reader,
                painted: Rc::default(),
            })
        })
        .expect("native reveal probe window");
    support::realize(window, cx).await;
    support::until(
        cx,
        |cx| {
            window
                .update(cx, |probe, _, cx| {
                    probe.reader.read(cx).loaded.is_some() && !probe.reader.read(cx).loading
                })
                .unwrap()
        },
        "native reveal fixture load",
    )
    .await;
    for _ in 0..3 {
        draw(window, cx);
    }
    let heading = window
        .update(cx, |probe, _, cx| probe.reader.read(cx).headings[1].block)
        .unwrap();
    seek(
        window,
        Location {
            block: heading,
            ..Location::default()
        },
        cx,
    );
    stable(window, "Table of Contents heading", cx);
    window
        .update(cx, |probe, window, cx| {
            probe.reader.update(cx, |reader, cx| {
                reader.deactivate(window, cx);
                reader.activate(window, cx);
            });
        })
        .unwrap();
    stable(window, "reactivated tab heading", cx);

    seek(
        window,
        Location {
            block: heading + 1,
            byte: "First line.\n".len(),
            offset: -3.,
            ..Location::default()
        },
        cx,
    );
    stable(window, "paragraph semantic anchor", cx);
    seek(
        window,
        Location {
            block: heading + 2,
            byte: 0,
            offset: -3.,
            ..Location::default()
        },
        cx,
    );
    stable(window, "indexed code semantic anchor", cx);
    seek(
        window,
        Location {
            block: heading + 2,
            byte: "let value = 42;\n".len() * 20 + 128,
            offset: -3.,
            ..Location::default()
        },
        cx,
    );
    stable(window, "horizontally revealed code semantic anchor", cx);
    window
        .update(cx, |probe, _, cx| {
            assert!(
                probe.reader.read(cx).code_scrolls.borrow()[&(heading + 2)]
                    .offset()
                    .x
                    < px(0.)
            );
        })
        .unwrap();
    window
        .update(cx, |probe, window, cx| {
            let reader = probe.reader.read(cx);
            let loaded = reader.loaded.as_ref().unwrap();
            let source = Source {
                block: heading + 3,
                row: 0,
            };
            let line = loaded.code[&source.block].line(0).unwrap();
            let entry = CodeCache::request(reader.codes.as_ref().unwrap(), source, &line, 14.);
            let mut chunk = Chunk {
                window: line.bytes.clone(),
                core: line.bytes,
                column: 0,
                x: 0.,
                local_x: 0.,
                width: 0.,
                layouts: None,
            };
            let (text, _) = crate::code_cache::prepare(loaded, source, &chunk);
            chunk.width = crate::code_cache::native(window.text_system(), text, 14.)
                .width
                .into();
            let mut entry = entry.borrow_mut();
            entry.width = chunk.width;
            entry.chunks.push(chunk);
            entry.complete = true;
        })
        .expect("warm Unicode glyphs without a hidden platform frame callback");
    seek(
        window,
        Location {
            block: heading + 3,
            offset: -3.,
            ..Location::default()
        },
        cx,
    );
    stable(window, "Unicode code semantic anchor", cx);
    window
        .update(cx, |_, window, _| window.remove_window())
        .unwrap();
}

fn seek(window: WindowHandle<Probe>, target: Location, cx: &mut AsyncApp) {
    window
        .update(cx, |probe, _, cx| {
            probe.reader.update(cx, |reader, cx| {
                reader.jump(target, false, cx);
            });
        })
        .unwrap();
}

fn stable(window: WindowHandle<Probe>, operation: &str, cx: &mut AsyncApp) {
    let first = draw(window, cx);
    let second = draw(window, cx);
    let third = draw(window, cx);
    assert_eq!(first, second, "first painted frame shifted: {operation}");
    assert_eq!(second, third, "second painted frame shifted: {operation}");
    window
        .update(cx, |probe, _, cx| {
            assert!(probe.reader.read(cx).reveal.borrow().target.is_none());
        })
        .unwrap();
}

fn draw(window: WindowHandle<Probe>, cx: &mut AsyncApp) -> (usize, gpui::Pixels) {
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear();
    })
    .expect("draw native reveal probe");
    window
        .update(cx, |probe, _, _| {
            let offset = probe.painted.get().expect("painted native list offset");
            (offset.item_ix, offset.offset_in_item)
        })
        .unwrap()
}

struct Probe {
    reader: Entity<Viewer>,
    painted: Rc<Cell<Option<ListOffset>>>,
}

impl Render for Probe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let reader = self.reader.clone();
        let painted = Rc::clone(&self.painted);
        div()
            .relative()
            .size_full()
            .child(self.reader.clone())
            .child(
                canvas(
                    move |_, _, cx| {
                        painted.set(Some(reader.read(cx).list.logical_scroll_top()));
                    },
                    |_, (), _, _| {},
                )
                .absolute()
                .size_full(),
            )
    }
}
