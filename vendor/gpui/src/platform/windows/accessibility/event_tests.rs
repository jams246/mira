use super::*;

fn snapshot(role: AccessibilityRole) -> AccessibilitySnapshot {
    AccessibilitySnapshot {
        id: 7,
        metadata: crate::accessibility::Accessibility {
            role,
            ..Default::default()
        },
        bounds: Default::default(),
        focused: false,
        focusable: true,
        invokable: false,
        writable: false,
    }
}

#[test]
fn unchanged_snapshots_produce_no_repeat_announcements() {
    let node = snapshot(AccessibilityRole::Button);
    assert!(changes(std::slice::from_ref(&node), std::slice::from_ref(&node)).is_empty());
}

#[test]
fn query_value_change_preserves_unicode_in_the_native_property_event() {
    let mut before = snapshot(AccessibilityRole::TextInput);
    before.metadata.value = Some("東京".into());
    let mut after = before.clone();
    after.metadata.value = Some("東京 café".into());
    assert_eq!(
        changes(&[before], &[after]),
        vec![Change::Property(
            7,
            UIA_ValueValuePropertyId,
            Value::Text("東京".into()),
            Value::Text("東京 café".into())
        )]
    );
}

#[test]
fn settled_status_announces_once_and_searching_does_not() {
    let mut empty = snapshot(AccessibilityRole::StaticText);
    empty.metadata.live = true;
    let mut searching = empty.clone();
    searching.metadata.name = "Searching...".into();
    searching.metadata.live = false;
    let pending = changes(
        std::slice::from_ref(&empty),
        std::slice::from_ref(&searching),
    );
    assert!(
        !pending
            .iter()
            .any(|change| matches!(change, Change::Event(_, _)))
    );
    let mut settled = searching.clone();
    settled.metadata.name = "Match 1 of 6".into();
    settled.metadata.live = true;
    let ready = changes(&[searching], std::slice::from_ref(&settled));
    assert_eq!(
        ready
            .iter()
            .filter(|change| matches!(change, Change::Event(7, event) if *event == UIA_LiveRegionChangedEventId))
            .count(),
        1
    );
    assert!(
        changes(
            std::slice::from_ref(&settled),
            std::slice::from_ref(&settled)
        )
        .is_empty()
    );
}

#[test]
fn checkbox_state_change_uses_the_native_toggle_property() {
    let mut before = snapshot(AccessibilityRole::CheckBox);
    before.metadata.selected = Some(false);
    let mut after = before.clone();
    after.metadata.selected = Some(true);
    assert_eq!(
        changes(&[before], &[after]),
        vec![Change::Property(
            7,
            UIA_ToggleToggleStatePropertyId,
            Value::Number(ToggleState_Off.0),
            Value::Number(ToggleState_On.0)
        )]
    );
}

#[test]
fn tab_selection_emits_property_and_selection_notification() {
    let mut before = snapshot(AccessibilityRole::Tab);
    before.metadata.selected = Some(false);
    let mut after = before.clone();
    after.metadata.selected = Some(true);
    assert_eq!(
        changes(&[before], &[after]),
        vec![
            Change::Property(
                7,
                UIA_SelectionItemIsSelectedPropertyId,
                Value::Flag(false),
                Value::Flag(true)
            ),
            Change::Event(7, UIA_SelectionItem_ElementSelectedEventId)
        ]
    );
}

#[test]
fn clearing_status_does_not_announce_empty_text() {
    let mut before = snapshot(AccessibilityRole::StaticText);
    before.metadata.live = true;
    before.metadata.name = "No matches".into();
    let mut after = before.clone();
    after.metadata.name = "".into();
    assert!(
        !changes(&[before], &[after])
            .iter()
            .any(|change| matches!(change, Change::Event(_, _)))
    );
}

#[test]
fn closing_a_preceding_tab_updates_native_position_and_total() {
    let mut before = snapshot(AccessibilityRole::Tab);
    before.metadata.set_position = Some((2, 3));
    let mut after = before.clone();
    after.metadata.set_position = Some((1, 2));
    assert_eq!(
        changes(&[before], &[after]),
        vec![
            Change::Property(
                7,
                UIA_PositionInSetPropertyId,
                Value::Number(2),
                Value::Number(1)
            ),
            Change::Property(
                7,
                UIA_SizeOfSetPropertyId,
                Value::Number(3),
                Value::Number(2)
            )
        ]
    );
}
