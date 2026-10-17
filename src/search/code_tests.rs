use crate::markdown;

use super::code_text;

#[test]
fn fenced_code_preserves_tabs_blanks_unicode_and_trailing_newline() {
    let document = markdown::parse("```rust\n\tlet 東京 = 1;\n\n  tail\n```\n".into());
    assert_eq!(
        code_text(&document, 0).as_deref(),
        Some("\tlet 東京 = 1;\n\n  tail\n")
    );
}

#[test]
fn indented_code_copies_parser_text_without_markdown_indentation() {
    let document = markdown::parse("    one\n    \ttwo\n\n    three\n".into());
    assert_eq!(
        code_text(&document, 0).as_deref(),
        Some("one\n\ttwo\n\nthree\n")
    );
}

#[test]
fn crlf_code_uses_parser_normalized_line_endings() {
    let document = markdown::parse("```\r\none\r\n\ttwo\r\n```\r\n".into());
    assert_eq!(code_text(&document, 0).as_deref(), Some("one\n\ttwo\n"));
}

#[test]
fn empty_code_and_non_code_blocks_are_distinct() {
    let document = markdown::parse("```\n```\n\nparagraph".into());
    assert_eq!(code_text(&document, 0).as_deref(), Some(""));
    assert_eq!(code_text(&document, 1), None);
    assert_eq!(code_text(&document, 999), None);
}

#[test]
fn oversized_code_copies_complete_unmounted_content() {
    let content = "\t東京  \n".repeat(100_000);
    let document = markdown::parse(format!("```\n{content}```\n"));
    assert_eq!(code_text(&document, 0).as_deref(), Some(content.as_str()));
}
