mod oracle;

use std::{cell::Cell, path::PathBuf, time::Duration};

use gpui::{
    AppContext, AsyncApp, Context, IntoElement, ListAlignment, ListState, Render, TextAlign,
    Window, canvas, div, prelude::*, px, relative,
};

use super::*;
use crate::{
    appearance::Palette,
    code_cache::CodeCache,
    content_paint::ContentPaint,
    image_cache::{ImageCache, ImageStatus},
    render_context::RenderContext,
    render_text::{TextFormat, TextKind},
    reveal::Reveal,
    selection::Position,
    text_cache::TextCache,
    timing::Timing,
    viewer::native_test_support as support,
};

struct Case {
    resources: RenderContext,
    index: Arc<FlowIndex>,
    entry: EntryHandle,
    format: TextFormat,
    finished: Cell<bool>,
}

impl Case {
    fn input(&self) -> Input<'_> {
        Input {
            resources: &self.resources,
            position: Position { block: 0, byte: 0 },
            format: self.format,
            width: self.resources.width,
        }
    }
}

pub(crate) async fn run(cx: &mut AsyncApp) {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".verification/reading-features/native-integration");
    std::fs::create_dir_all(&directory).expect("native flow image directory");
    for (name, width, height) in [("flow-tall.png", 14, 42), ("flow-wide.png", 38, 10)] {
        image::RgbaImage::new(width, height)
            .save(directory.join(name))
            .expect("native flow image fixture");
    }
    let source = format!(
        "{}{}\\\n\\\nleft **{}** right ![alt](flow-tall.png)\\\nend",
        "![](flow-tall.png)![](flow-wide.png)".repeat(69),
        " a **B** *é* `q` [東京](target.md) e\u{301} 😀 ".repeat(42),
        "東京😀".repeat(240),
    );
    let loaded = Arc::new(LoadedDocument {
        path: directory.join("flow-native.md"),
        document: mira::markdown::parse(source),
        read_time: Duration::ZERO,
        parse_time: Duration::ZERO,
        code: HashMap::new(),
        code_index_time: Duration::ZERO,
    });
    assert_eq!(loaded.document.blocks.len(), 1);
    let source = Source::whole(0, None);
    let index = Arc::new(FlowIndex::new(
        &loaded.document,
        source.runs(&loaded.document),
    ));
    assert!(index.pieces().len() > 128);
    assert!(index.pieces().iter().any(|piece| piece.range.len() > 2048));
    let probe = cx
        .open_window(support::window_options(), |_, cx| {
            cx.new(|_| Probe { case: None })
        })
        .expect("hidden native flow window");
    support::realize(probe, cx).await;
    let base = probe
        .update(cx, |_, _, cx| {
            resources(Arc::clone(&loaded), cx.entity_id(), 190., 1.)
        })
        .unwrap();
    support::until(
        cx,
        |cx| {
            probe
                .update(cx, |_, window, cx| {
                    (0..2).all(|target| {
                        match ImageCache::request(&base.images, target, window, cx) {
                            ImageStatus::Ready(_) => true,
                            ImageStatus::Loading => false,
                            ImageStatus::Failed(error) => panic!("native flow image: {error}"),
                        }
                    })
                })
                .unwrap()
        },
        "native flow images decoded",
    )
    .await;
    for width in [190., 800.] {
        for scale in [1., 1.5, 2.] {
            for align in [TextAlign::Left, TextAlign::Center, TextAlign::Right] {
                let case = probe
                    .update(cx, |probe, _, cx| {
                        let mut resources =
                            resources(Arc::clone(&loaded), cx.entity_id(), width, scale);
                        resources.images = Rc::clone(&base.images);
                        let format = TextFormat {
                            align,
                            ..TextKind::Body.into()
                        };
                        let entry = FlowCache::request(
                            &resources.flows,
                            source,
                            Config {
                                width,
                                font_size: 16. * scale,
                                line_height: 24. * scale,
                                format,
                                images: resources.images.borrow().geometry_revision(),
                            },
                        );
                        let case = Rc::new(Case {
                            resources,
                            index: Arc::clone(&index),
                            entry,
                            format,
                            finished: Cell::new(false),
                        });
                        probe.case = Some(Rc::clone(&case));
                        cx.notify();
                        case
                    })
                    .unwrap();
                support::until(
                    cx,
                    |cx| {
                        cx.update_window(probe.into(), |_, window, cx| {
                            window.refresh();
                            window.draw(cx).clear();
                        })
                        .unwrap();
                        case.finished.get()
                    },
                    "batched flow matches full native flex geometry",
                )
                .await;
            }
        }
    }
    probe
        .update(cx, |_, window, _| window.remove_window())
        .unwrap();
}

fn resources(
    loaded: Arc<LoadedDocument>,
    owner: gpui::EntityId,
    width: f32,
    scale: f32,
) -> RenderContext {
    let resources = RenderContext {
        palette: Palette::light(),
        images: ImageCache::new(Arc::clone(&loaded), owner),
        texts: TextCache::new(Arc::clone(&loaded), owner),
        codes: CodeCache::new(Arc::clone(&loaded), owner),
        flows: FlowCache::new(Arc::clone(&loaded), owner),
        loaded,
        owner,
        width,
        scale,
        links: Rc::new(|_, _, _| {}),
        selection: Default::default(),
        tables: Default::default(),
        paint: ContentPaint::new(Timing::start()),
        cell: None,
        highlights: Default::default(),
        reveal: Reveal::new(ListState::new(1, ListAlignment::Top, px(500.)), owner),
        code_scrolls: Default::default(),
    };
    resources
        .selection
        .borrow_mut()
        .select_all(&resources.loaded.document);
    {
        let mut highlights = resources.highlights.borrow_mut();
        highlights.enabled = true;
        highlights.active = Some(0);
        highlights.matches = Arc::new(vec![mira::search::SearchMatch {
            block: 0,
            cell: None,
            range: 0..10,
        }]);
    }
    resources
}

struct Probe {
    case: Option<Rc<Case>>,
}

impl Render for Probe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let case = self.case.clone();
        let scale = case.as_ref().map_or(1., |case| case.resources.scale);
        div()
            .size_full()
            .font_family("Segoe UI")
            .text_size(px(16. * scale))
            .line_height(relative(1.5))
            .child(
                canvas(
                    move |_, window, cx| {
                        let Some(case) = case.filter(|case| !case.finished.get()) else {
                            return;
                        };
                        FlowCache::advance(
                            &case.resources.flows,
                            &case.entry,
                            case.input(),
                            f32::INFINITY,
                            window,
                            cx,
                        );
                        if case.entry.borrow().complete {
                            oracle::verify(&case, window, cx);
                            case.finished.set(true);
                        }
                    },
                    |_, (), _, _| {},
                )
                .size_full(),
            )
    }
}
