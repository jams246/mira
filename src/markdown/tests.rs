use crate::document::{BlockKind, ContextKind, Document, Inline, InlineStyle, Text};

use super::parse;

pub(super) fn text(document: &Document, runs: &[Inline]) -> String {
    runs.iter()
        .map(|run| run.text.as_str(&document.source))
        .collect()
}

#[test]
fn empty_markdown_has_no_blocks() {
    assert!(parse(String::new()).blocks.is_empty());
}

#[test]
fn plain_text_references_original_source() {
    let document = parse("Hello, world!".into());
    let BlockKind::Paragraph(runs) = &document.blocks[0].kind else {
        panic!("Expected paragraph");
    };
    assert_eq!(runs[0].text, Text::Source(0..13));
}

#[test]
fn headings_preserve_all_six_levels() {
    let document = parse("# One\n## Two\n### Three\n#### Four\n##### Five\n###### Six\n".into());
    let levels: Vec<_> = document
        .blocks
        .iter()
        .filter_map(|block| match block.kind {
            BlockKind::Heading { level, .. } => Some(level),
            _ => None,
        })
        .collect();
    assert_eq!(levels, [1, 2, 3, 4, 5, 6]);
}

#[test]
fn mixed_inline_formatting_combines_styles() {
    let document = parse("***both*** ~~gone~~ `code`".into());
    let BlockKind::Paragraph(runs) = &document.blocks[0].kind else {
        panic!("Expected paragraph");
    };
    let styles: Vec<_> = runs
        .iter()
        .filter(|run| !run.text.as_str(&document.source).trim().is_empty())
        .map(|run| run.style)
        .collect();
    assert_eq!(
        styles,
        [
            InlineStyle {
                bold: true,
                italic: true,
                ..Default::default()
            },
            InlineStyle {
                strike: true,
                ..Default::default()
            },
            InlineStyle {
                code: true,
                ..Default::default()
            },
        ]
    );
}

#[test]
fn tight_nested_lists_preserve_counters_and_contexts() {
    let document = parse("3. first\n   - child\n4. second\n".into());
    let contexts: Vec<_> = document
        .contexts
        .iter()
        .map(|context| (context.parent, &context.kind))
        .collect();
    assert_eq!(
        contexts,
        [
            (
                None,
                &ContextKind::ListItem {
                    number: Some(3),
                    checked: None
                }
            ),
            (
                Some(0),
                &ContextKind::ListItem {
                    number: None,
                    checked: None
                }
            ),
            (
                None,
                &ContextKind::ListItem {
                    number: Some(4),
                    checked: None
                }
            ),
        ]
    );
}

#[test]
fn continuation_paragraphs_do_not_repeat_list_marker() {
    let document = parse("- first\n\n  continuation\n\n- second\n".into());
    assert_eq!(
        document
            .blocks
            .iter()
            .map(|block| block.marker)
            .collect::<Vec<_>>(),
        [true, false, true]
    );
}

#[test]
fn task_lists_retain_checkbox_state() {
    let document = parse("- [x] done\n- [ ] pending\n".into());
    assert_eq!(
        document
            .contexts
            .iter()
            .map(|context| &context.kind)
            .collect::<Vec<_>>(),
        [
            &ContextKind::ListItem {
                number: None,
                checked: Some(true)
            },
            &ContextKind::ListItem {
                number: None,
                checked: Some(false)
            },
        ]
    );
}

#[test]
fn quote_list_quote_keeps_ancestor_order() {
    let document = parse("> - item\n>   > inner\n".into());
    assert_eq!(
        document.contexts,
        [
            crate::document::BlockContext {
                parent: None,
                kind: ContextKind::Quote
            },
            crate::document::BlockContext {
                parent: Some(0),
                kind: ContextKind::ListItem {
                    number: None,
                    checked: None
                }
            },
            crate::document::BlockContext {
                parent: Some(1),
                kind: ContextKind::Quote
            },
        ]
    );
}

#[test]
fn fenced_code_preserves_whitespace_and_information() {
    let document = parse("```rust\nlet x = 1;\n\n\tline\n```\n".into());
    let BlockKind::Code { runs, language } = &document.blocks[0].kind else {
        panic!("Expected code");
    };
    assert_eq!(
        (
            text(&document, runs),
            language
                .as_ref()
                .map(|value| value.as_str(&document.source))
        ),
        ("let x = 1;\n\n\tline\n".into(), Some("rust"))
    );
}

#[test]
fn indented_code_removes_only_markdown_indentation() {
    let document = parse("    one\n      two\n".into());
    let BlockKind::Code { runs, language } = &document.blocks[0].kind else {
        panic!("Expected code");
    };
    assert_eq!(
        (text(&document, runs), language),
        ("one\n  two\n".into(), &None)
    );
}
