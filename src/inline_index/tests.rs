use crate::document::{BlockKind, InlineStyle};
use crate::markdown;

use super::*;

#[test]
fn slicing_styled_link_text_retains_utf8_source_ranges_and_metadata() {
    let document = markdown::parse("before [***東京***](next.md) after".into());
    let BlockKind::Paragraph(runs) = &document.blocks[0].kind else {
        panic!("Expected paragraph");
    };
    let index = InlineIndex::new(&document, runs);
    let sliced = index.slice(&document, runs, 7..13);
    assert_eq!(sliced.len(), 1);
    assert_eq!(sliced[0].text.as_str(&document.source), "東京");
    assert!(matches!(sliced[0].text, Text::Source(_)));
    assert_eq!(
        sliced[0].style,
        InlineStyle {
            bold: true,
            italic: true,
            ..Default::default()
        }
    );
    assert_eq!(
        sliced[0]
            .link
            .map(|target| document.targets[target].as_str()),
        Some("next.md")
    );
    assert_eq!(index.len(), "before 東京 after".len());
}

#[test]
fn slicing_normalized_literal_copies_only_requested_bytes() {
    let document = markdown::parse("`one\ntwo` tail".into());
    let BlockKind::Paragraph(runs) = &document.blocks[0].kind else {
        panic!("Expected paragraph");
    };
    let sliced = InlineIndex::new(&document, runs).slice(&document, runs, 1..6);
    assert_eq!(sliced[0].text, Text::Literal("ne tw".into()));
    assert!(sliced[0].style.code);
}

#[test]
fn slicing_image_alt_preserves_original_image_and_link_destinations() {
    let document = markdown::parse("a [![sample](local.png)](next.md) z".into());
    let BlockKind::Paragraph(runs) = &document.blocks[0].kind else {
        panic!("Expected paragraph");
    };
    let sliced = InlineIndex::new(&document, runs).slice(&document, runs, 3..6);
    assert_eq!(sliced[0].text.as_str(&document.source), "amp");
    assert_eq!(
        sliced[0]
            .image
            .map(|target| document.targets[target].as_str()),
        Some("local.png")
    );
    assert_eq!(
        sliced[0]
            .link
            .map(|target| document.targets[target].as_str()),
        Some("next.md")
    );
}

#[test]
fn slicing_across_indented_code_runs_preserves_all_requested_text() {
    let document = markdown::parse("    first\n    東京\n    last\n".into());
    let BlockKind::Code { runs, .. } = &document.blocks[0].kind else {
        panic!("Expected code");
    };
    let index = InlineIndex::new(&document, runs);
    let sliced = index.slice(&document, runs, 3..16);
    assert_eq!(
        sliced
            .iter()
            .map(|run| run.text.as_str(&document.source))
            .collect::<String>(),
        "st\n東京\nlas"
    );
}

#[test]
fn empty_reversed_outside_and_invalid_utf8_ranges_do_not_create_invalid_text() {
    let document = markdown::parse("😀".into());
    let BlockKind::Paragraph(runs) = &document.blocks[0].kind else {
        panic!("Expected paragraph");
    };
    let index = InlineIndex::new(&document, runs);
    assert!(
        [0..0, Range { start: 4, end: 1 }, 20..30, 1..3]
            .into_iter()
            .all(|range| index.slice(&document, runs, range).is_empty())
    );
    let empty = InlineIndex::new(&document, &[]);
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);
}
