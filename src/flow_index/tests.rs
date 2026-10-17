use super::*;
use crate::{
    document::{BlockKind, InlineStyle, Text},
    markdown,
};

fn paragraph(source: &str) -> (Document, FlowIndex) {
    let document = markdown::parse(source.into());
    let index = FlowIndex::new(&document, runs(&document));
    (document, index)
}

fn runs(document: &Document) -> &[Inline] {
    let BlockKind::Paragraph(runs) = &document.blocks[0].kind else {
        panic!("Expected paragraph");
    };
    runs
}

fn plain(document: &Document, runs: &[Inline]) -> String {
    runs.iter()
        .map(|run| run.text.as_str(&document.source))
        .collect()
}

fn source_document(source: &str) -> (Document, Vec<Inline>) {
    let document = Document {
        source: source.into(),
        blocks: Vec::new(),
        contexts: Vec::new(),
        targets: Vec::new(),
        tables: Vec::new(),
    };
    let run = Inline {
        source_range: 0..source.len(),
        text: Text::Source(0..source.len()),
        style: InlineStyle::default(),
        link: None,
        image: None,
    };
    (document, vec![run])
}

fn pieces(index: &FlowIndex) -> Vec<(Range<usize>, PieceKind)> {
    index
        .pieces()
        .iter()
        .map(|piece| (piece.range.clone(), piece.kind))
        .collect()
}

#[test]
fn styled_words_images_and_hard_breaks_keep_known_semantic_offsets() {
    let (document, index) = paragraph("ab**cd**ef ![First](one.png)  \n![](two.png) 東京");
    assert_eq!(
        pieces(&index),
        [
            (0..7, PieceKind::Text),
            (7..12, PieceKind::Image { inline: 3 }),
            (12..13, PieceKind::Break),
            (13..13, PieceKind::Image { inline: 5 }),
            (13..14, PieceKind::Text),
            (14..20, PieceKind::Text),
        ]
    );
    assert_eq!(index.text_len(), 20);
    let word = index.slice(&document, runs(&document), 0..7);
    assert_eq!(
        word.iter()
            .map(|run| (run.text.as_str(&document.source), run.style.bold))
            .collect::<Vec<_>>(),
        [("ab", false), ("cd", true), ("ef ", false)]
    );
    assert!(word.iter().all(|run| matches!(run.text, Text::Source(_))));
    assert_eq!(word, runs(&document)[..3]);
}

#[test]
fn adjacent_empty_images_retain_distinct_occurrences_and_links() {
    let (document, index) = paragraph("[![](one.png)](one.md)[![](two.png)](#end)Z");
    assert_eq!(
        pieces(&index),
        [
            (0..0, PieceKind::Image { inline: 0 }),
            (0..0, PieceKind::Image { inline: 1 }),
            (0..1, PieceKind::Text),
        ]
    );
    let runs = runs(&document);
    assert_eq!(document.targets[runs[0].link.unwrap()], "one.md");
    assert_eq!(document.targets[runs[1].link.unwrap()], "#end");
    assert_ne!(runs[0].source_range, runs[1].source_range);
    assert_eq!(plain(&document, &index.slice(&document, runs, 0..1)), "Z");
}

#[test]
fn unicode_whitespace_and_consecutive_breaks_have_exact_ranges() {
    let (document, runs) = source_document("\tA\u{2003}\u{a0}東京 \n\nZ");
    let index = FlowIndex::new(&document, &runs);
    assert_eq!(
        pieces(&index),
        [
            (0..1, PieceKind::Text),
            (1..5, PieceKind::Text),
            (5..7, PieceKind::Text),
            (7..14, PieceKind::Text),
            (14..15, PieceKind::Break),
            (15..16, PieceKind::Break),
            (16..17, PieceKind::Text),
        ]
    );
    let reconstructed: String = index
        .pieces()
        .iter()
        .map(|piece| {
            plain(
                &document,
                &index.slice(&document, &runs, piece.range.clone()),
            )
        })
        .collect();
    assert_eq!(reconstructed, document.source);
}

#[test]
fn image_alt_whitespace_and_newlines_are_one_atomic_occurrence() {
    let (document, mut runs) = source_document("tail");
    let mut image = runs[0].clone();
    image.text = Text::Literal("alpha\n beta".into());
    image.image = Some(0);
    runs.insert(0, image);
    let index = FlowIndex::new(&document, &runs);
    assert_eq!(
        pieces(&index),
        [
            (0..11, PieceKind::Image { inline: 0 }),
            (11..15, PieceKind::Text),
        ]
    );
    assert_eq!(
        plain(&document, &index.slice(&document, &runs, 11..15)),
        "tail"
    );
}

#[test]
fn normalized_literals_and_links_keep_original_formatting() {
    let (document, index) = paragraph("x **&amp;** [é](next.md) ![alt](image.png)");
    assert_eq!(
        pieces(&index),
        [
            (0..2, PieceKind::Text),
            (2..4, PieceKind::Text),
            (4..7, PieceKind::Text),
            (7..10, PieceKind::Image { inline: 5 }),
        ]
    );
    let normalized = index.slice(&document, runs(&document), 2..4);
    assert_eq!(plain(&document, &normalized), "& ");
    assert!(matches!(&normalized[0].text, Text::Literal(text) if text.as_ref() == "&"));
    assert!(normalized[0].style.bold);
    let linked = index.slice(&document, runs(&document), 4..6);
    assert_eq!(plain(&document, &linked), "é");
    assert_eq!(document.targets[linked[0].link.unwrap()], "next.md");
    assert!(matches!(linked[0].text, Text::Source(_)));
}

#[test]
fn zero_text_length_does_not_discard_empty_image_occurrences() {
    let (document, mut runs) = source_document("");
    let mut image = runs[0].clone();
    image.image = Some(0);
    runs.insert(1, image);
    runs.push(runs[0].clone());
    let index = FlowIndex::new(&document, &runs);
    assert_eq!(index.text_len(), 0);
    assert_eq!(pieces(&index), [(0..0, PieceKind::Image { inline: 1 })]);
    assert!(FlowIndex::new(&document, &[]).pieces().is_empty());
}

#[test]
fn many_styled_runs_and_empty_images_reconstruct_the_known_prose() {
    let source = "**é** ![](one.png)東京  \n".repeat(4_000);
    let (document, index) = paragraph(&source);
    let runs = runs(&document);
    let expected = "é 東京\n".repeat(3_999) + "é 東京";
    let mut output = String::new();
    let mut images = 0;
    for piece in index.pieces() {
        if let PieceKind::Image { inline } = piece.kind {
            assert!(runs[inline].image.is_some());
            assert!(piece.range.is_empty());
            images += 1;
        }
        output.push_str(&plain(
            &document,
            &index.slice(&document, runs, piece.range.clone()),
        ));
    }
    assert_eq!(images, 4_000);
    assert_eq!(output, expected);
    assert_eq!(output.len(), index.text_len());
}
