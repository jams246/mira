use std::{path::PathBuf, time::Duration};

use gpui::{AppContext, AsyncApp, Context, IntoElement, Render, WindowHandle, div, px};

use super::*;
use crate::{
    appearance::Palette,
    render_text,
    viewer::{Viewer, native_test_support as support},
};

pub(crate) async fn run(window: WindowHandle<Viewer>, cx: &mut AsyncApp) {
    let loaded = Arc::new(LoadedDocument {
        path: PathBuf::new(),
        document: mira::markdown::parse(
            "café **東京** e\u{301} 😀 [linked](target.md) short words ".repeat(96),
        ),
        read_time: Duration::ZERO,
        parse_time: Duration::ZERO,
        code: HashMap::new(),
        code_index_time: Duration::ZERO,
    });
    let source = Source::whole(0, None);
    let config = Config {
        width: 188.,
        font_size: 16.,
        kind: TextKind::Body,
    };
    let index = Arc::new(InlineIndex::new(
        &loaded.document,
        source.runs(&loaded.document),
    ));
    let helper = cx
        .open_window(support::window_options(), |_, cx| cx.new(|_| Probe))
        .expect("text shaping helper window");
    support::realize(helper, cx).await;
    let cache = window
        .update(cx, |_, _, cx| {
            TextCache::new(Arc::clone(&loaded), cx.entity_id())
        })
        .unwrap();
    let entry = TextCache::request(&cache, source, config, Some(Arc::clone(&index)));
    helper
        .update(cx, |_, window, cx| {
            TextCache::demand(&cache, &entry, source, 0, window, cx);
            assert!(!entry.borrow().scheduled);
            TextCache::demand(&cache, &entry, source, 1, window, cx);
            assert!(entry.borrow().scheduled);
            TextCache::demand(&cache, &entry, source, usize::MAX, window, cx);
            TextCache::demand(&cache, &entry, source, 0, window, cx);
            assert!(entry.borrow().scheduled);
            // No subsequent frame can start a deferred shaping job for this window.
            window.remove_window();
        })
        .unwrap();
    support::until(
        cx,
        |_| entry.borrow().rows > 0,
        "native shaping starts without a subsequent frame",
    )
    .await;
    while !entry.borrow().complete {
        let previous = entry.borrow().rows;
        window
            .update(cx, |_, window, cx| {
                TextCache::demand(&cache, &entry, source, previous, window, cx);
                assert!(!entry.borrow().scheduled);
                TextCache::demand(&cache, &entry, source, usize::MAX, window, cx);
            })
            .unwrap();
        support::until(
            cx,
            |_| {
                let state = entry.borrow();
                state.complete || state.rows > previous
            },
            "incremental native shaping completes",
        )
        .await;
    }
    let system = window
        .update(cx, |_, window, _| window.text_system().background_shaper())
        .unwrap();
    let oracle_document = Arc::clone(&loaded);
    let expected_rows = cx
        .background_executor()
        .spawn(async move {
            let prepared = render_text::prepare(
                &oracle_document.document,
                source.runs(&oracle_document.document),
                config.kind,
                Palette::light(),
            );
            system
                .shape_text(
                    prepared.text.into(),
                    px(config.font_size),
                    &prepared.runs,
                    Some(px(config.width)),
                    None,
                )
                .expect("independent native layout")
                .iter()
                .map(|line| line.wrap_boundaries().len() + 1)
                .sum::<usize>()
        })
        .await;
    {
        let state = entry.borrow();
        assert_eq!(state.error, None);
        assert_eq!(state.cursor, index.len());
        assert_eq!(state.rows, expected_rows);
        assert!(state.chunks.len() > 1);
        let mut row = 0;
        for chunk in &state.chunks {
            assert_eq!(chunk.row, row);
            for byte in [chunk.range.start, chunk.range.end] {
                assert_eq!(
                    index.floor_boundary(&loaded.document, source.runs(&loaded.document), byte),
                    byte,
                );
            }
            row += chunk.rows;
        }
        assert_eq!(row, expected_rows);
    }
    window
        .update(cx, |_, window, cx| {
            TextCache::demand(&cache, &entry, source, usize::MAX, window, cx);
            assert!(!entry.borrow().scheduled);
        })
        .unwrap();
    drop(entry);
    let pending = TextCache::request(
        &cache,
        source,
        Config {
            width: 189.,
            ..config
        },
        Some(index),
    );
    let weak_entry = Rc::downgrade(&pending);
    let weak_cache = Rc::downgrade(&cache);
    window
        .update(cx, |_, window, cx| {
            TextCache::demand(&cache, &pending, source, usize::MAX, window, cx);
        })
        .unwrap();
    drop(pending);
    drop(cache);
    assert!(weak_entry.upgrade().is_none() && weak_cache.upgrade().is_none());
    cx.background_executor()
        .timer(Duration::from_millis(20))
        .await;
    assert!(weak_entry.upgrade().is_none() && weak_cache.upgrade().is_none());
}

struct Probe;

impl Render for Probe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}
