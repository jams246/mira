use super::{History, Location};

fn location(block: usize, byte: usize) -> Location {
    Location {
        block,
        byte,
        ..Location::default()
    }
}

#[test]
fn back_and_forward_preserve_actual_scrolled_departures() {
    let mut history = History::default();
    history.jump(location(1, 5), location(4, 0));
    assert_eq!(history.back(location(6, 18)), Some(location(1, 5)));
    assert_eq!(history.forward(location(2, 3)), Some(location(6, 18)));
    assert_eq!(history.back(location(7, 2)), Some(location(2, 3)));
}

#[test]
fn new_jump_discards_forward_history() {
    let mut history = History::default();
    history.jump(location(0, 0), location(1, 0));
    history.back(location(1, 0));
    history.jump(location(0, 0), location(2, 0));
    assert!(history.can_back());
    assert!(!history.can_forward());
    assert_eq!(history.forward(location(2, 0)), None);
}

#[test]
fn unchanged_jump_preserves_forward_branch() {
    let mut history = History::default();
    history.jump(location(0, 0), location(1, 0));
    history.back(location(1, 0));
    history.jump(location(0, 0), location(0, 0));
    assert!(history.can_forward());
    assert!(!history.can_back());
}

#[test]
fn fresh_tabs_have_no_artificial_history_and_remain_independent() {
    let mut first = History::default();
    let second = History::default();
    first.jump(location(0, 0), location(4, 0));
    assert!(first.can_back());
    assert!(!second.can_back());
    assert!(!second.can_forward());
}

#[test]
fn clear_removes_both_directions() {
    let mut history = History::default();
    history.jump(location(0, 0), location(1, 0));
    history.jump(location(1, 0), location(2, 0));
    history.back(location(2, 0));
    history.clear();
    assert!(!history.can_back());
    assert!(!history.can_forward());
}
