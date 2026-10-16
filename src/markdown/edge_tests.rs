use crate::document::{BlockKind, Text};

use super::{parse, tests::text};

#[test]
fn entities_and_escapes_use_parser_normalized_text() {
    let document = parse("A &amp; B \\*literal\\* &#x1F600;".into());
    let BlockKind::Paragraph(runs) = &document.blocks[0].kind else {
        panic!("Expected paragraph");
    };
    assert_eq!(text(&document, runs), "A & B *literal* 😀");
}

#[test]
fn inline_code_normalizes_newlines_and_surrounding_spaces() {
    let document = parse("` one\ntwo `".into());
    let BlockKind::Paragraph(runs) = &document.blocks[0].kind else {
        panic!("Expected paragraph");
    };
    assert_eq!(runs[0].text, Text::Literal("one two".into()));
}

#[test]
fn inline_code_without_normalization_references_inner_source() {
    let document = parse("`example`".into());
    let BlockKind::Paragraph(runs) = &document.blocks[0].kind else {
        panic!("Expected paragraph");
    };
    assert_eq!(runs[0].text, Text::Source(1..8));
}

#[test]
fn nested_strong_restores_outer_style() {
    let document = parse("**outer **inner** outer**".into());
    let BlockKind::Paragraph(runs) = &document.blocks[0].kind else {
        panic!("Expected paragraph");
    };
    assert!(runs.iter().all(|run| run.style.bold));
}

#[test]
fn unicode_lf_and_crlf_render_identically() {
    let lf = parse("日本語 café 😀\nnext  \nlast\n".into());
    let crlf = parse("日本語 café 😀\r\nnext  \r\nlast\r\n".into());
    let BlockKind::Paragraph(lf_runs) = &lf.blocks[0].kind else {
        panic!("Expected paragraph");
    };
    let BlockKind::Paragraph(crlf_runs) = &crlf.blocks[0].kind else {
        panic!("Expected paragraph");
    };
    assert_eq!(
        (text(&lf, lf_runs), text(&crlf, crlf_runs)),
        (
            "日本語 café 😀 next\nlast".into(),
            "日本語 café 😀 next\nlast".into()
        )
    );
}

#[test]
fn code_crlf_and_lf_preserve_equal_logical_lines() {
    let document = parse("```\r\nfirst\r\nsecond\r\n```\r\n".into());
    let BlockKind::Code { runs, .. } = &document.blocks[0].kind else {
        panic!("Expected code");
    };
    assert_eq!(text(&document, runs), "first\nsecond\n");
}

#[test]
fn raw_html_remains_literal_text() {
    let document = parse("<script>alert('no execution')</script>\n\ntext <b>bold</b>".into());
    let BlockKind::RawHtml(raw) = &document.blocks[0].kind else {
        panic!("Expected literal HTML");
    };
    let BlockKind::Paragraph(inline) = &document.blocks[1].kind else {
        panic!("Expected paragraph");
    };
    assert_eq!(
        (text(&document, raw), text(&document, inline)),
        (
            "<script>alert('no execution')</script>\n".into(),
            "text <b>bold</b>".into()
        )
    );
}

#[test]
fn malformed_markdown_remains_readable() {
    let document = parse("**open [unfinished](\n\n```rust\nlet x = 1;".into());
    let BlockKind::Paragraph(runs) = &document.blocks[0].kind else {
        panic!("Expected paragraph");
    };
    let BlockKind::Code { runs: code, .. } = &document.blocks[1].kind else {
        panic!("Expected unclosed code");
    };
    assert_eq!(
        (text(&document, runs), text(&document, code)),
        ("**open [unfinished](".into(), "let x = 1;".into())
    );
}

#[test]
fn empty_list_items_have_visible_marker_leaves() {
    let document = parse("-\n- content\n".into());
    assert_eq!(
        document
            .blocks
            .iter()
            .map(|block| block.marker)
            .collect::<Vec<_>>(),
        [true, true]
    );
}

#[test]
fn list_with_nested_list_first_retains_outer_marker() {
    let document = parse("-\n  - child\n".into());
    assert_eq!(
        document
            .blocks
            .iter()
            .map(|block| (block.context, block.marker))
            .collect::<Vec<_>>(),
        [(Some(0), true), (Some(1), true)]
    );
}

#[test]
fn table_images_and_escaped_pipes_remain_in_cell() {
    let document = parse("| x | y |\n|---|---|\n| a\\|b | ![alt](local.png) |\n".into());
    let BlockKind::TableRow { cells, .. } = &document.blocks[1].kind else {
        panic!("Expected row");
    };
    assert_eq!(
        (
            text(&document, &cells[0]),
            cells[1][0]
                .image
                .map(|index| document.targets[index].as_str())
        ),
        ("a|b".into(), Some("local.png"))
    );
}

#[test]
fn large_plain_paragraph_keeps_original_source_without_text_copy() {
    let source = "日本語 sample ".repeat(300_000);
    let length = source.trim_end().len();
    let document = parse(source);
    let BlockKind::Paragraph(runs) = &document.blocks[0].kind else {
        panic!("Expected paragraph");
    };
    assert_eq!((runs.len(), &runs[0].text), (1, &Text::Source(0..length)));
}

#[test]
fn deep_quotes_use_iterative_context_storage() {
    let document = parse(format!("{}deep\n", "> ".repeat(512)));
    assert_eq!(
        (document.contexts.len(), document.blocks[0].context),
        (512, Some(511))
    );
}

#[test]
fn all_source_ranges_remain_valid_utf8_boundaries() {
    let document = parse("# Héllo 😀\n\n- **日本語** &amp; `x`\n\n![alt](image.png)\n".into());
    assert!(
        document
            .blocks
            .iter()
            .all(|block| document.source.get(block.source_range.clone()).is_some())
    );
}

#[test]
fn literal_text_accessor_rejects_mismatched_source_without_panicking() {
    assert_eq!(Text::Source(1..5).as_str("é"), "");
}

#[test]
fn nested_images_flatten_alt_in_source_order_and_keep_outer_link() {
    let document = parse(
        "[![prefix ![one](one.png) middle ![two **bold**](two.png) suffix](outer.png)](next.md)"
            .into(),
    );
    let BlockKind::Image { target, alt, link } = &document.blocks[0].kind else {
        panic!("Expected outer image");
    };
    assert_eq!(
        (
            text(&document, alt),
            document.targets[*target].as_str(),
            link.map(|index| document.targets[index].as_str()),
        ),
        (
            "prefix one middle two bold suffix".into(),
            "outer.png",
            Some("next.md"),
        )
    );
}
