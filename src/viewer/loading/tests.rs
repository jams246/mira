use super::*;

#[test]
fn changed_document_restores_relative_to_surviving_heading() {
    let old = mira::markdown::parse("# A\n\ntext\n\n# B\n\nbody".into());
    let new = mira::markdown::parse("# Intro\n\nnew\n\n# A\n\ntext\n\n# B\n\nbody".into());
    let result = restore_location(
        Location {
            block: 3,
            ..Location::default()
        },
        &headings::index(&old),
        &headings::index(&new),
        &new,
        false,
    );
    assert_eq!(result.block, 5);
}

#[test]
fn shortened_document_clamps_removed_location() {
    let document = mira::markdown::parse("short".into());
    let result = restore_location(
        Location {
            block: 100,
            ..Location::default()
        },
        &[],
        &[],
        &document,
        false,
    );
    assert_eq!(result.block, 0);
}

#[test]
fn removed_nearest_heading_uses_previous_surviving_heading() {
    let old = mira::markdown::parse("# A\n\ntext\n\n# Removed\n\nbody".into());
    let new =
        mira::markdown::parse("# Intro\n\nnew\n\n# A\n\ntext\n\nreplacement\n\nbody\n\nend".into());
    let result = restore_location(
        Location {
            block: 3,
            ..Location::default()
        },
        &headings::index(&old),
        &headings::index(&new),
        &new,
        false,
    );
    assert_eq!(result.block, 5);
}

#[test]
fn shortened_unicode_text_clamps_to_character_boundary_and_end() {
    let document = mira::markdown::parse("a😀b".into());
    for (byte, expected) in [(3, 1), (usize::MAX, 6)] {
        let result = restore_location(
            Location {
                byte,
                ..Location::default()
            },
            &[],
            &[],
            &document,
            false,
        );
        assert_eq!(result.byte, expected);
    }
}

#[test]
fn shortened_table_clamps_cell_and_block_global_utf8_offset() {
    let document = mira::markdown::parse("| left | 😀 |\n| - | - |\n".into());
    let result = restore_location(
        Location {
            cell: Some(9),
            byte: 7,
            ..Location::default()
        },
        &[],
        &[],
        &document,
        false,
    );
    assert_eq!(result.cell, Some(1));
    assert_eq!(result.byte, 5);
}
