use std::sync::atomic::AtomicBool;

use crate::markdown;

use super::{SearchMatch, search};

fn find(source: &str, query: &str, match_case: bool) -> Vec<SearchMatch> {
    search(
        &markdown::parse(source.into()),
        query,
        match_case,
        &AtomicBool::new(false),
    )
}

#[test]
fn formatting_entities_escapes_and_links_search_display_text() {
    let source = "A **bold** &amp; \\*word\\* [link](destination.md) `code`";
    assert_eq!(
        find(source, "bold & *word* link code", true)[0].range,
        2..25
    );
    assert!(find(source, "destination", false).is_empty());
    assert!(find(source, "**", true).is_empty());
}

#[test]
fn case_sensitive_and_scalar_lowercase_keep_original_utf8_ranges() {
    let source = "ÉKİ 東京 ékİ";
    assert_eq!(
        find(source, "ékİ", false)
            .iter()
            .map(|item| item.range.clone())
            .collect::<Vec<_>>(),
        [0..7, 15..20]
    );
    assert_eq!(find(source, "ékİ", true)[0].range, 15..20);
    assert!(find("Straße", "STRASSE", false).is_empty());
    assert!(find("é", "e\u{301}", false).is_empty());
}

#[test]
fn partial_lowercase_expansions_cannot_split_source_characters() {
    assert!(find("İ", "i", false).is_empty());
    assert!(find("İ", "\u{307}", false).is_empty());
    assert_eq!(find("İ", "i\u{307}", false)[0].range, 0..2);
    assert_eq!(find("İi", "i", false)[0].range, 2..3);
}

#[test]
fn images_separate_search_spans_but_keep_semantic_offsets() {
    let source = "left![secret](image.png)right![ ](other.png)tail";
    assert!(find(source, "secret", false).is_empty());
    assert!(find(source, "leftright", false).is_empty());
    assert!(find(source, "righttail", false).is_empty());
    assert_eq!(find(source, "right", true)[0].range, 10..15);
    assert_eq!(find(source, "tail", true)[0].range, 16..20);
    assert!(find("![secret](image.png)", "secret", false).is_empty());
}

#[test]
fn empty_alt_images_still_separate_search_spans() {
    assert!(find("left![](image.png)right", "leftright", true).is_empty());
}

#[test]
fn blocks_cells_and_list_markers_cannot_join_matches() {
    assert!(find("left\n\nright", "left\n\nright", true).is_empty());
    assert!(find("- item", "- item", true).is_empty());
    assert!(find("- [x] task", "[x]", true).is_empty());
    let source = "| alpha | be**ta** |\n| --- | --- |\n| a | b |";
    assert!(find(source, "alpha\tbeta", true).is_empty());
    assert_eq!(
        find(source, "beta", true),
        [SearchMatch {
            block: 0,
            cell: Some(1),
            range: 6..10,
        }]
    );
}

#[test]
fn table_offsets_include_image_alt_and_preceding_separator_bytes() {
    let source = "| a![ignored](a.png)b | target |\n| --- | --- |";
    let result = find(source, "target", false);
    assert_eq!(result[0].cell, Some(1));
    assert_eq!(result[0].range, 10..16);
}

#[test]
fn results_are_exact_nonoverlapping_and_document_ordered() {
    let result = find("aaaaa\n\n````\naaaa\n````\n\n# AAAA", "aa", false);
    assert_eq!(
        result
            .iter()
            .map(|item| (item.block, item.range.clone()))
            .collect::<Vec<_>>(),
        [
            (0, 0..2),
            (0, 2..4),
            (1, 0..2),
            (1, 2..4),
            (2, 0..2),
            (2, 2..4)
        ]
    );
}

#[test]
fn empty_missing_and_cancelled_queries_return_no_matches() {
    assert!(find("text", "", false).is_empty());
    assert!(find("text", "missing", false).is_empty());
    assert!(
        search(
            &markdown::parse("text".into()),
            "text",
            false,
            &AtomicBool::new(true),
        )
        .is_empty()
    );
}

#[test]
fn high_frequency_matches_have_no_silent_limit() {
    let document = markdown::parse("a ".repeat(100_001));
    let matches = search(&document, "a", false, &AtomicBool::new(false));
    assert_eq!(matches.len(), 100_001);
    assert_eq!(matches.last().unwrap().range, 200_000..200_001);
}

#[test]
fn raw_html_is_literal_display_text_and_remains_searchable() {
    assert_eq!(find("<div>text</div>", "text", true)[0].range, 5..9);
}
