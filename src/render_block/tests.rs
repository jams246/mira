use super::visible_ancestors;
use mira::{
    document::{Block, BlockKind, ContextKind, Document},
    markdown,
};

fn paragraph<'a>(document: &'a Document, expected: &str) -> &'a Block {
    document
        .blocks
        .iter()
        .find(|block| match &block.kind {
            BlockKind::Paragraph(runs) => {
                runs.iter()
                    .map(|run| run.text.as_str(&document.source))
                    .collect::<String>()
                    == expected
            }
            _ => false,
        })
        .expect("Parsed paragraph")
}

#[test]
fn deep_quote_items_keep_their_marker_and_continuation_ancestry() {
    let quotes = "> ".repeat(9);
    for (source, number, checked) in [
        (
            format!("- {quotes}FIRST\n  {quotes}\n  {quotes}CONTINUATION\n"),
            None,
            None,
        ),
        (
            format!("42. {quotes}FIRST\n    {quotes}\n    {quotes}CONTINUATION\n"),
            Some(42),
            None,
        ),
        (
            format!("- [x]\n  {quotes}FIRST\n  {quotes}\n  {quotes}CONTINUATION\n"),
            None,
            Some(true),
        ),
    ] {
        let document = markdown::parse(source);
        let first = paragraph(&document, "FIRST");
        let continuation = paragraph(&document, "CONTINUATION");
        assert!(first.marker);
        assert!(!continuation.marker);
        assert_eq!(first.context, continuation.context);
        let contexts = visible_ancestors(&document, first.context);
        assert_eq!(contexts.len(), 9);
        assert!(
            contexts[..8]
                .iter()
                .all(|kind| matches!(kind, ContextKind::Quote))
        );
        assert_eq!(contexts[8], &ContextKind::ListItem { number, checked });
        assert_eq!(visible_ancestors(&document, continuation.context), contexts);
    }
}

#[test]
fn nearest_retained_list_does_not_append_an_older_outer_list() {
    let quotes = "> ".repeat(9);
    let document = markdown::parse(format!("- {quotes}outer\n  {quotes}\n  {quotes}7. inner\n"));
    assert_eq!(
        document
            .contexts
            .iter()
            .filter(|context| matches!(context.kind, ContextKind::ListItem { .. }))
            .count(),
        2
    );
    let inner = paragraph(&document, "inner");
    assert!(inner.marker);
    let contexts = visible_ancestors(&document, inner.context);
    assert_eq!(contexts.len(), 8);
    assert_eq!(
        contexts[0],
        &ContextKind::ListItem {
            number: Some(7),
            checked: None
        }
    );
    assert!(
        contexts[1..]
            .iter()
            .all(|kind| matches!(kind, ContextKind::Quote))
    );
}

#[test]
fn ordinary_nested_context_order_is_unchanged() {
    let document = markdown::parse("- outer\n\n    7. > inner\n".into());
    let inner = paragraph(&document, "inner");
    assert_eq!(
        visible_ancestors(&document, inner.context),
        [
            &ContextKind::Quote,
            &ContextKind::ListItem {
                number: Some(7),
                checked: None
            },
            &ContextKind::ListItem {
                number: None,
                checked: None
            },
        ]
    );
}

#[test]
fn deeply_quoted_non_list_content_keeps_only_eight_visual_frames() {
    let document = markdown::parse(format!("{}deep\n", "> ".repeat(512)));
    assert_eq!(document.contexts.len(), 512);
    let contexts = visible_ancestors(&document, paragraph(&document, "deep").context);
    assert_eq!(contexts.len(), 8);
    assert!(
        contexts
            .iter()
            .all(|kind| matches!(kind, ContextKind::Quote))
    );
}

#[test]
fn unnested_and_empty_documents_need_no_context_wrappers() {
    let document = markdown::parse("plain".into());
    assert!(visible_ancestors(&document, paragraph(&document, "plain").context).is_empty());
    assert!(visible_ancestors(&markdown::parse(String::new()), None).is_empty());
}
