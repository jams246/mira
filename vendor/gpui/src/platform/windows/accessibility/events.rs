use super::*;
use crate::SharedString;

#[derive(Debug, PartialEq)]
pub(super) enum Value {
    Text(SharedString),
    Flag(bool),
    Number(i32),
}

impl Value {
    fn variant(&self) -> VARIANT {
        match self {
            Self::Text(value) => VARIANT::from(value.as_ref()),
            Self::Flag(value) => VARIANT::from(*value),
            Self::Number(value) => VARIANT::from(*value),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(super) enum Change {
    Property(u64, UIA_PROPERTY_ID, Value, Value),
    Event(u64, UIA_EVENT_ID),
}

impl Change {
    pub fn id(&self) -> u64 {
        match self {
            Self::Property(id, ..) | Self::Event(id, _) => *id,
        }
    }
    pub fn raise(self, provider: &IRawElementProviderSimple) -> Result<()> {
        unsafe {
            match self {
                Self::Property(_, property, old, new) => UiaRaiseAutomationPropertyChangedEvent(
                    provider,
                    property,
                    &old.variant(),
                    &new.variant(),
                ),
                Self::Event(_, event) => UiaRaiseAutomationEvent(provider, event),
            }
        }
    }
}

pub(super) fn changes(
    before: &[AccessibilitySnapshot],
    after: &[AccessibilitySnapshot],
) -> Vec<Change> {
    let previous: HashMap<_, _> = before.iter().map(|node| (node.id, node)).collect();
    let mut changes = Vec::new();
    for node in after {
        let Some(old) = previous.get(&node.id) else {
            continue;
        };
        let id = node.id;
        let mut property = |property, old: Value, new: Value| {
            if old != new {
                changes.push(Change::Property(id, property, old, new));
            }
        };
        property(
            UIA_NamePropertyId,
            Value::Text(old.metadata.name.clone()),
            Value::Text(node.metadata.name.clone()),
        );
        property(
            UIA_IsEnabledPropertyId,
            Value::Flag(!old.metadata.disabled),
            Value::Flag(!node.metadata.disabled),
        );
        if old.metadata.value != node.metadata.value {
            property(
                UIA_ValueValuePropertyId,
                Value::Text(old.metadata.value.clone().unwrap_or_default()),
                Value::Text(node.metadata.value.clone().unwrap_or_default()),
            );
        }
        if node.metadata.role == AccessibilityRole::CheckBox {
            property(
                UIA_ToggleToggleStatePropertyId,
                Value::Number(i32::from(old.metadata.selected.unwrap_or(false))),
                Value::Number(i32::from(node.metadata.selected.unwrap_or(false))),
            );
        } else if matches!(
            node.metadata.role,
            AccessibilityRole::Tab | AccessibilityRole::ListItem
        ) {
            property(
                UIA_SelectionItemIsSelectedPropertyId,
                Value::Flag(old.metadata.selected.unwrap_or(false)),
                Value::Flag(node.metadata.selected.unwrap_or(false)),
            );
        }
        property(
            UIA_LiveSettingPropertyId,
            Value::Number(i32::from(old.metadata.live)),
            Value::Number(i32::from(node.metadata.live)),
        );
        let old_position = set_position(old);
        let new_position = set_position(node);
        property(
            UIA_PositionInSetPropertyId,
            Value::Number(old_position.0),
            Value::Number(new_position.0),
        );
        property(
            UIA_SizeOfSetPropertyId,
            Value::Number(old_position.1),
            Value::Number(new_position.1),
        );
        if node.metadata.selected == Some(true)
            && old.metadata.selected != Some(true)
            && matches!(
                node.metadata.role,
                AccessibilityRole::Tab | AccessibilityRole::ListItem
            )
        {
            changes.push(Change::Event(id, UIA_SelectionItem_ElementSelectedEventId));
        }
        if node.metadata.live
            && old.metadata.name != node.metadata.name
            && !node.metadata.name.is_empty()
        {
            changes.push(Change::Event(id, UIA_LiveRegionChangedEventId));
        }
    }
    changes
}

#[cfg(test)]
#[path = "event_tests.rs"]
mod tests;
