use super::*;
use std::{path::PathBuf, time::Duration};

fn cache(markdown: &str) -> CacheHandle {
    TextCache::new(
        Arc::new(LoadedDocument {
            path: PathBuf::new(),
            document: mira::markdown::parse(markdown.into()),
            read_time: Duration::ZERO,
            parse_time: Duration::ZERO,
            code: HashMap::new(),
            code_index_time: Duration::ZERO,
        }),
        EntityId::from(1),
    )
}

fn config() -> Config {
    Config {
        width: 80.,
        font_size: 16.,
        kind: TextKind::Body,
    }
}

fn index(cache: &CacheHandle, source: Source) -> Arc<InlineIndex> {
    let cache = cache.borrow();
    Arc::new(InlineIndex::new(
        &cache.loaded.document,
        source.runs(&cache.loaded.document),
    ))
}

fn plain(cache: &CacheHandle, entry: &Entry, source: Source, range: Range<usize>) -> String {
    let cache = cache.borrow();
    entry
        .slice(&cache.loaded.document, source, range)
        .iter()
        .map(|run| run.text.as_str(&cache.loaded.document.source))
        .collect()
}

#[test]
fn nonzero_spans_reuse_metadata_and_slice_absolute_mother_offsets() {
    let cache = cache("prefix **東京** [café](plain.md) suffix");
    let source = Source {
        span: Some((7, 19)),
        ..Source::whole(0, None)
    };
    let seed = index(&cache, source);
    let entry = TextCache::request(&cache, source, config(), Some(Arc::clone(&seed)));
    let state = entry.borrow();
    assert!(Arc::ptr_eq(state.index.as_ref().unwrap(), &seed));
    assert_eq!(
        (state.cursor, state.context_start, state.start, state.len),
        (7, 7, 7, 19)
    );
    assert_eq!(state.intrinsic_width(), IntrinsicWidth::Unknown);
    assert!(!state.complete);
    assert_eq!(plain(&cache, &state, source, 7..19), "東京 café");
    assert_eq!(plain(&cache, &state, source, 0..usize::MAX), "東京 café");
    let document = &cache.borrow().loaded.document;
    let slice = state.slice(document, source, 7..19);
    assert!(slice[0].style.bold);
    assert_eq!(document.targets[slice[2].link.unwrap()], "plain.md");
}

#[test]
fn same_mother_separate_spans_and_whole_have_independent_entries() {
    let cache = cache("prefix 東京 café suffix");
    let whole = Source::whole(0, None);
    let first = Source {
        span: Some((7, 13)),
        ..whole
    };
    let second = Source {
        span: Some((14, 19)),
        ..whole
    };
    let seed = index(&cache, whole);
    let first_entry = TextCache::request(&cache, first, config(), Some(Arc::clone(&seed)));
    let second_entry = TextCache::request(&cache, second, config(), Some(Arc::clone(&seed)));
    let whole_entry = TextCache::request(&cache, whole, config(), None);
    assert!(!Rc::ptr_eq(&first_entry, &second_entry));
    assert!(!Rc::ptr_eq(&whole_entry, &first_entry));
    assert!(Rc::ptr_eq(
        &first_entry,
        &TextCache::request(&cache, first, config(), Some(seed))
    ));
    assert_eq!(cache.borrow().entries.len(), 3);
    assert_eq!(whole_entry.borrow().cursor, 0);
    assert_eq!(whole_entry.borrow().len, 26);
}

#[test]
fn estimates_use_span_length_and_consumed_bytes_without_mother_prefix() {
    let cache = cache(&("prefix ".repeat(100) + "東京 café suffix"));
    let source = Source {
        span: Some((700, 712)),
        ..Source::whole(0, None)
    };
    let entry = TextCache::request(&cache, source, config(), Some(index(&cache, source)));
    assert_eq!(entry.borrow().estimated_rows(), 2);
    let mut state = entry.borrow_mut();
    state.cursor = 706;
    state.rows = 2;
    assert_eq!(state.estimated_rows(), 4);
    state.complete = true;
    assert_eq!(state.estimated_rows(), 2);
}

#[test]
fn standalone_image_alt_and_table_cell_ranges_borrow_the_correct_mother() {
    let image = cache("![styled 東京 café](images/tall.png)");
    let source = Source {
        span: Some((7, 13)),
        ..Source::whole(0, None)
    };
    assert!(matches!(
        image.borrow().loaded.document.blocks[0].kind,
        mira::document::BlockKind::Image { .. }
    ));
    let entry = TextCache::request(&image, source, config(), Some(index(&image, source)));
    assert_eq!(plain(&image, &entry.borrow(), source, 7..13), "東京");

    let table = cache("| A | B |\n|---|---|\n| one | 東京 café |");
    let source = Source {
        span: Some((7, 12)),
        ..Source::whole(1, Some(1))
    };
    let entry = TextCache::request(&table, source, config(), Some(index(&table, source)));
    assert_eq!(plain(&table, &entry.borrow(), source, 7..12), "café");
    assert_eq!(entry.borrow().cursor, 7);
}

#[test]
fn empty_and_invalid_spans_terminate_without_requesting_native_work() {
    let cache = cache("東京 café");
    let source = Source {
        span: Some((6, 6)),
        ..Source::whole(0, None)
    };
    let entry = TextCache::request(&cache, source, config(), Some(index(&cache, source)));
    assert!(entry.borrow().complete);
    assert_eq!(entry.borrow().intrinsic_width(), IntrinsicWidth::Exact(0.));
    assert_eq!(entry.borrow().estimated_rows(), 1);
    for span in [(20, 30), (8, 2)] {
        let source = Source {
            span: Some(span),
            ..source
        };
        let entry = TextCache::request(&cache, source, config(), None);
        assert!(entry.borrow().complete);
        assert!(
            entry
                .borrow()
                .error
                .as_deref()
                .is_some_and(|error| error.contains("span"))
        );
        assert_eq!(entry.borrow().intrinsic_width(), IntrinsicWidth::Full);
    }
}

#[test]
fn seed_can_fill_an_existing_request_without_replacing_its_entry() {
    let cache = cache("whole source 東京");
    let source = Source::whole(0, None);
    let entry = TextCache::request(&cache, source, config(), None);
    let seed = index(&cache, source);
    let reused = TextCache::request(&cache, source, config(), Some(Arc::clone(&seed)));
    assert!(Rc::ptr_eq(&entry, &reused));
    assert!(Arc::ptr_eq(reused.borrow().index.as_ref().unwrap(), &seed));
}
