use super::*;
use crate::markdown;

fn code(source: &str) -> (Document, CodeLayout) {
    let document = markdown::parse(source.into());
    let BlockKind::Code { runs, .. } = &document.blocks[0].kind else {
        panic!("code");
    };
    let layout = CodeLayout::build(&document, runs);
    (document, layout)
}

#[test]
fn lines_preserve_blank_lines_and_final_newline_without_copying_source() {
    let (_, layout) = code("```\na\n\nb\n```\n");
    assert_eq!(layout.line_count(), 4);
    assert_eq!(layout.line(0).map(|line| line.bytes), Some(0..1));
    assert_eq!(layout.line(1).map(|line| line.bytes), Some(2..2));
    assert_eq!(layout.line(3).map(|line| line.bytes), Some(5..5));
}

#[test]
fn source_columns_round_trip_at_every_boundary_and_inside_expanded_tabs() {
    let (_, layout) = code("```\na\tb\tcd\n\tx\n```\n");
    let line = layout.line(0).expect("line");
    assert_eq!(line.columns, 10);
    assert_eq!(layout.byte_for_column(&line, 2), 1);
    assert_eq!(layout.byte_for_column(&line, 4), 2);
    for byte in line.bytes.clone() {
        assert_eq!(
            layout.byte_for_column(&line, layout.column_for_byte(&line, byte)),
            byte
        );
    }
    let next = layout.line(1).expect("line");
    assert_eq!(layout.column_for_byte(&next, next.bytes.start + 1), 4);
}

#[test]
fn slices_across_normalized_indented_runs_preserve_utf8_and_exact_text() {
    let (document, layout) = code("    first\n    東京\n    \tlast\n");
    let BlockKind::Code { runs, .. } = &document.blocks[0].kind else {
        panic!("code");
    };
    let line = layout.line(1).expect("line");
    assert!(!line.ascii);
    let slice = layout.slice(&document, runs, line.bytes);
    assert_eq!(
        slice
            .iter()
            .map(|run| run.text.as_str(&document.source))
            .collect::<String>(),
        "東京"
    );
    assert!(slice.iter().all(|run| matches!(run.text, Text::Source(_))));
}

#[test]
fn ordinary_code_avoids_indexes_and_large_code_keeps_semantic_block_identity() {
    let document = markdown::parse(format!(
        "# Head\n\n```\n{}\n```\n\n```\nsmall\n```",
        "a".repeat(MIN_INDEXED_BYTES)
    ));
    let indexes = index(&document);
    assert_eq!(indexes.len(), 1);
    assert!(indexes.contains_key(&1));
}
