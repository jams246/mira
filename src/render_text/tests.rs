use super::*;
use mira::{document::BlockKind, markdown};

#[test]
fn combines_native_styles_and_preserves_utf8_link_ranges() {
    let document = markdown::parse(
        "***Grüße*** and [東京](https://example.com) [**café**](next.md) ~~gone~~".into(),
    );
    let BlockKind::Paragraph(inlines) = &document.blocks[0].kind else {
        panic!("paragraph");
    };
    let prepared = prepare(&document, inlines, TextKind::Body, Palette::light());
    assert_eq!(prepared.text, "Grüße and 東京 café gone");
    assert_eq!(
        prepared.runs.iter().map(|run| run.len).sum::<usize>(),
        prepared.text.len()
    );
    assert!(
        prepared
            .runs
            .iter()
            .any(|run| run.font.weight == FontWeight::BOLD && run.font.style == FontStyle::Italic)
    );
    assert_eq!(
        prepared
            .links
            .iter()
            .map(|(range, target)| {
                (
                    &prepared.text[range.clone()],
                    document.targets[*target].as_str(),
                )
            })
            .collect::<Vec<_>>(),
        [("東京", "https://example.com"), ("café", "next.md")]
    );
    assert!(prepared.runs.iter().any(|run| run.strikethrough.is_some()));
}

#[test]
fn code_link_ranges_follow_tab_expansion_without_changing_destinations() {
    let document = markdown::parse("prefix\t[é\t中](target.md) [tail](next.md)".into());
    let BlockKind::Paragraph(inlines) = &document.blocks[0].kind else {
        panic!("paragraph");
    };
    let prepared = prepare(&document, inlines, TextKind::Code, Palette::light());
    assert_eq!(prepared.text, "prefix  é   中 tail");
    assert_eq!(
        prepared
            .links
            .iter()
            .map(|(range, target)| {
                (
                    &prepared.text[range.clone()],
                    document.targets[*target].as_str(),
                )
            })
            .collect::<Vec<_>>(),
        [("é   中", "target.md"), ("tail", "next.md")]
    );
}

#[test]
fn code_preserves_indentation_blank_lines_and_native_monospace_runs() {
    let document = markdown::parse("```\nlet x = 1;\n\n\t  x\n```".into());
    let BlockKind::Code { runs, .. } = &document.blocks[0].kind else {
        panic!("code");
    };
    let prepared = prepare(&document, runs, TextKind::Code, Palette::light());
    assert_eq!(prepared.text, "let x = 1;\n\n      x\n");
    assert_eq!(prepared.source_len, "let x = 1;\n\n\t  x\n".len());
    assert_eq!(
        prepared.runs.iter().map(|run| run.len).sum::<usize>(),
        prepared.text.len()
    );
    assert!(
        prepared
            .runs
            .iter()
            .all(|run| run.font.family.as_ref() == "Consolas")
    );
}
