use gpui::{FontStyle, FontWeight, TextRun, font};

use crate::selection_text;

use super::*;

fn state(matches: Vec<SearchMatch>, active: Option<usize>) -> HighlightHandle {
    Rc::new(RefCell::new(Highlights {
        matches: Arc::new(matches),
        active,
        enabled: true,
    }))
}

fn item(block: usize, cell: Option<usize>, range: Range<usize>) -> SearchMatch {
    SearchMatch { block, cell, range }
}

fn run(len: usize) -> TextRun {
    let mut font = font("Segoe UI");
    font.style = FontStyle::Italic;
    font.weight = FontWeight::BOLD;
    TextRun {
        len,
        font,
        color: Palette::light().link,
        background_color: None,
        underline: Some(gpui::UnderlineStyle::default()),
        strikethrough: None,
    }
}

#[test]
fn visible_ranges_use_block_cell_and_global_byte_offsets() {
    let state = state(
        vec![
            item(0, None, 0..5),
            item(1, Some(0), 2..7),
            item(1, Some(1), 8..12),
            item(1, Some(1), 15..20),
            item(2, None, 0..4),
        ],
        Some(3),
    );
    assert_eq!(
        state
            .borrow()
            .ranges(Position { block: 1, byte: 10 }, Some(1), 8),
        [(0..2, false), (5..8, true)]
    );
    state.borrow_mut().enabled = false;
    assert!(
        state
            .borrow()
            .ranges(Position { block: 1, byte: 0 }, Some(1), 20)
            .is_empty()
    );
}

#[test]
fn highlights_split_style_runs_and_distinguish_active_match() {
    let palette = Palette::light();
    let original = run(7);
    let highlighted = highlight(
        vec![original.clone(), run(5)],
        Position { block: 0, byte: 0 },
        None,
        12,
        &TextMap::default(),
        &state(vec![item(0, None, 2..5), item(0, None, 6..10)], Some(1)),
        palette,
    );
    assert_eq!(
        highlighted.iter().map(|run| run.len).collect::<Vec<_>>(),
        [2, 3, 1, 1, 3, 2]
    );
    assert_eq!(highlighted[1].background_color, Some(palette.find));
    assert_eq!(highlighted[3].background_color, Some(palette.find_active));
    assert!(
        highlighted
            .iter()
            .all(|run| run.font == original.font && run.underline == original.underline)
    );
}

#[test]
fn selection_overrides_find_without_losing_other_matches() {
    let palette = Palette::dark();
    let highlighted = highlight(
        vec![run(10)],
        Position { block: 0, byte: 0 },
        None,
        10,
        &TextMap::default(),
        &state(vec![item(0, None, 1..9)], Some(0)),
        palette,
    );
    let selected = selection_text::highlight(highlighted, Some(3..7), palette);
    assert_eq!(
        selected.iter().map(|run| run.len).collect::<Vec<_>>(),
        [1, 2, 4, 2, 1]
    );
    assert_eq!(selected[1].background_color, Some(palette.find_active));
    assert_eq!(selected[2].background_color, Some(palette.selection));
}

#[test]
fn semantic_tabs_and_unicode_map_to_complete_display_ranges() {
    let palette = Palette::light();
    let (text, mapping) = TextMap::expand("a\t東京");
    let highlighted = highlight(
        vec![run(text.len())],
        Position { block: 0, byte: 20 },
        None,
        8,
        &mapping,
        &state(vec![item(0, None, 21..28)], None),
        palette,
    );
    assert_eq!(
        highlighted.iter().map(|run| run.len).collect::<Vec<_>>(),
        [1, 9]
    );
    assert_eq!(highlighted[1].background_color, Some(palette.find));
}
