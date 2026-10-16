use pulldown_cmark::Alignment;

use crate::document::BlockKind;

use super::{parse, tests::text};

#[test]
fn table_header_body_and_alignment_match_source() {
    let document =
        parse("| Left | Center | Right |\n| :--- | :---: | ---: |\n| a | **b** | c |\n".into());
    let rows: Vec<_> = document
        .blocks
        .iter()
        .filter_map(|block| match &block.kind {
            BlockKind::TableRow {
                table,
                header,
                cells,
            } => Some((
                *table,
                *header,
                cells
                    .iter()
                    .map(|cell| text(&document, cell))
                    .collect::<Vec<_>>(),
            )),
            _ => None,
        })
        .collect();
    assert_eq!(
        (document.tables[0].alignments.clone(), rows),
        (
            vec![Alignment::Left, Alignment::Center, Alignment::Right],
            vec![
                (
                    0,
                    true,
                    vec!["Left".into(), "Center".into(), "Right".into()]
                ),
                (0, false, vec!["a".into(), "b".into(), "c".into()])
            ],
        )
    );
}

#[test]
fn relative_and_reference_links_keep_destinations() {
    let document = parse("[local](next.md) and [named][id]\n\n[id]: https://example.com/\n".into());
    let BlockKind::Paragraph(runs) = &document.blocks[0].kind else {
        panic!("Expected paragraph");
    };
    assert_eq!(
        runs.iter()
            .filter_map(|run| run.link.map(|index| (
                run.text.as_str(&document.source),
                document.targets[index].as_str()
            )))
            .collect::<Vec<_>>(),
        [("local", "next.md"), ("named", "https://example.com/")]
    );
}

#[test]
fn standalone_linked_image_retains_alt_path_and_link() {
    let document = parse("[![**sample** photo](images/picture.png)](next.md)".into());
    let BlockKind::Image { target, alt, link } = &document.blocks[0].kind else {
        panic!("Expected image");
    };
    assert_eq!(
        (
            document.targets[*target].as_str(),
            text(&document, alt),
            link.map(|index| document.targets[index].as_str())
        ),
        ("images/picture.png", "sample photo".into(), Some("next.md"))
    );
}

#[test]
fn inline_images_keep_paragraph_position() {
    let document = parse("before ![alt](image.png) after".into());
    let BlockKind::Paragraph(runs) = &document.blocks[0].kind else {
        panic!("Expected paragraph");
    };
    assert_eq!(
        runs.iter()
            .map(|run| (run.text.as_str(&document.source), run.image.is_some()))
            .collect::<Vec<_>>(),
        [("before ", false), ("alt", true), (" after", false)]
    );
}

#[test]
fn horizontal_rule_produces_separate_leaf() {
    let document = parse("before\n\n---\n\nafter".into());
    assert!(matches!(document.blocks[1].kind, BlockKind::Rule));
}
