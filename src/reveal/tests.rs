use super::*;

#[test]
fn reveal_matches_only_its_semantic_block_cell_and_byte_range() {
    let position = Position { block: 2, byte: 10 };
    let target = Location {
        block: 2,
        cell: Some(3),
        byte: 19,
        offset: 0.,
    };
    assert!(inside(target, position, Some(3), 9));
    assert!(!inside(target, position, Some(2), 9));
    assert!(!inside(target, position, Some(3), 8));
    assert!(!inside(
        Location { block: 1, ..target },
        position,
        Some(3),
        9
    ));
}

#[test]
fn visible_reading_sample_prefers_top_left_text_over_later_words_and_rows() {
    let list = ListState::new(0, gpui::ListAlignment::Top, px(100.));
    let handle = Reveal::new(list, EntityId::from(1));
    let mut state = handle.borrow_mut();
    let bounds = Bounds::new(point(px(0.), px(100.)), gpui::size(px(600.), px(400.)));
    state.begin_frame(bounds);
    let location = |byte| Location {
        byte,
        ..Location::default()
    };
    state.record(location(300), point(px(0.), px(140.)));
    state.record(location(100), point(px(300.), px(95.)));
    state.record(location(20), point(px(0.), px(95.)));
    state.record(location(200), point(px(400.), px(95.)));
    assert_eq!(state.reading.byte, 20);
}

#[test]
fn new_frame_allows_scrolled_reading_position_and_rejects_horizontal_overscan() {
    let list = ListState::new(0, gpui::ListAlignment::Top, px(100.));
    let handle = Reveal::new(list, EntityId::from(1));
    let mut state = handle.borrow_mut();
    let bounds = Bounds::new(point(px(0.), px(100.)), gpui::size(px(600.), px(400.)));
    state.begin_frame(bounds);
    state.record(Location::default(), bounds.origin);
    state.begin_frame(bounds);
    state.record(
        Location {
            byte: 5000,
            ..Location::default()
        },
        bounds.origin,
    );
    let outside = Bounds::new(point(px(700.), px(110.)), gpui::size(px(100.), px(24.)));
    assert_eq!((state.reading.byte, state.samples(outside)), (5000, false));
}

#[test]
fn unicode_code_reveal_ignores_unpainted_context_and_accepts_final_endpoint() {
    let position = Position {
        block: 2,
        byte: 100,
    };
    let target = |byte| Location {
        block: 2,
        byte,
        ..Location::default()
    };
    assert!(!code_inside(target(120), position, &(150..200), 250));
    assert!(!code_inside(target(220), position, &(150..200), 250));
    assert!(!code_inside(target(200), position, &(150..200), 250));
    assert!(code_inside(target(175), position, &(150..200), 250));
    assert!(code_inside(target(200), position, &(150..200), 200));
}
