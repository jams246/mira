use std::rc::Rc;

use crate::{App, Bounds, ClickEvent, FocusHandle, Pixels, SharedString, Window};

/// A native role exposed to assistive technology.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AccessibilityRole {
    /// An invokable command.
    #[default]
    Button,
    /// A document tab.
    Tab,
    /// An item within a list.
    ListItem,
    /// An editable text field.
    TextInput,
    /// A checkable option.
    CheckBox,
    /// Noninteractive status or descriptive text.
    StaticText,
}

#[derive(Clone, Default)]
pub(crate) struct Accessibility {
    pub name: SharedString,
    pub role: AccessibilityRole,
    pub disabled: bool,
    pub selected: Option<bool>,
    pub value: Option<SharedString>,
    pub live: bool,
    pub set_position: Option<(usize, usize)>,
}

/// An immutable native accessibility node from the current frame.
#[derive(Clone)]
pub struct AccessibilitySnapshot {
    pub(crate) id: u64,
    pub(crate) metadata: Accessibility,
    pub(crate) bounds: Bounds<Pixels>,
    pub(crate) focused: bool,
    pub(crate) focusable: bool,
    pub(crate) invokable: bool,
    pub(crate) writable: bool,
}

/// An action requested through the native accessibility provider.
#[derive(Clone)]
pub enum AccessibilityAction {
    /// Move keyboard focus without activating the control.
    Focus,
    /// Activate the control's existing click handler.
    Invoke,
    /// Replace an editable field's value.
    SetValue(String),
}

#[derive(Clone)]
pub(crate) struct PaintedAccessibility {
    pub snapshot: AccessibilitySnapshot,
    pub focus: Option<FocusHandle>,
    pub invoke: Vec<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
    pub set_value: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>,
}

impl Window {
    pub(crate) fn accessibility_action(
        &mut self,
        id: u64,
        action: AccessibilityAction,
        cx: &mut App,
    ) {
        let Some(node) = self
            .rendered_frame
            .accessibility
            .iter()
            .find(|node| node.snapshot.id == id)
            .cloned()
        else {
            return;
        };
        if node.snapshot.metadata.disabled {
            return;
        }
        if let Some(focus) = node.focus {
            self.focus(&focus);
        }
        match action {
            AccessibilityAction::Focus => {}
            AccessibilityAction::Invoke => {
                let position = node.snapshot.bounds.center();
                let event = ClickEvent::Mouse(crate::MouseClickEvent {
                    down: crate::MouseDownEvent {
                        position,
                        ..Default::default()
                    },
                    up: crate::MouseUpEvent {
                        position,
                        ..Default::default()
                    },
                });
                for callback in node.invoke {
                    callback(&event, self, cx);
                }
            }
            AccessibilityAction::SetValue(value) => {
                if let Some(callback) = node.set_value {
                    callback(&value, self, cx);
                }
            }
        }
    }
}
