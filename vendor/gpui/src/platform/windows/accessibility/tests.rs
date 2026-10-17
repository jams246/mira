use super::*;

fn bridge() -> AccessibilityBridge {
    let bridge = AccessibilityBridge::default();
    bridge.0.nodes.write().unwrap().push(AccessibilitySnapshot {
        id: 7,
        metadata: crate::accessibility::Accessibility {
            name: "Find query".into(),
            role: AccessibilityRole::TextInput,
            disabled: false,
            selected: Some(true),
            value: Some("東京".into()),
        },
        bounds: Default::default(),
        focused: true,
        focusable: true,
        invokable: true,
        writable: true,
    });
    bridge
}

#[test]
fn missing_and_closed_nodes_reject_native_requests() {
    let bridge = bridge();
    let missing = bridge.provider(HWND::default(), Some(9)).unwrap();
    let stale = bridge.provider(HWND::default(), Some(7)).unwrap();
    assert_eq!(
        missing
            .request(AccessibilityAction::Invoke)
            .unwrap_err()
            .code(),
        HRESULT(UIA_E_ELEMENTNOTAVAILABLE as i32)
    );
    bridge.close();
    assert_eq!(
        stale
            .request(AccessibilityAction::Focus)
            .unwrap_err()
            .code(),
        HRESULT(UIA_E_ELEMENTNOTAVAILABLE as i32)
    );
}

#[test]
fn disabled_control_cannot_enqueue_invoke() {
    let bridge = bridge();
    bridge.0.nodes.write().unwrap()[0].metadata.disabled = true;
    let provider = bridge.provider(HWND::default(), Some(7)).unwrap();
    assert_eq!(
        provider
            .request(AccessibilityAction::Invoke)
            .unwrap_err()
            .code(),
        HRESULT(UIA_E_ELEMENTNOTENABLED as i32)
    );
    assert!(bridge.take_requests().is_empty());
}

#[test]
fn stale_provider_cannot_invoke_replacement_node() {
    let bridge = bridge();
    let provider = bridge.provider(HWND::default(), Some(7)).unwrap();
    bridge.0.nodes.write().unwrap()[0].id = 8;
    assert!(provider.request(AccessibilityAction::Invoke).is_err());
    assert!(bridge.take_requests().is_empty());
}

#[test]
fn native_value_pattern_exposes_unicode_and_writability() {
    let bridge = bridge();
    let provider = bridge
        .provider(HWND::default(), Some(7))
        .unwrap()
        .cast::<IRawElementProviderSimple>()
        .unwrap();
    let value = unsafe { provider.GetPatternProvider(UIA_ValuePatternId) }
        .unwrap()
        .cast::<IValueProvider>()
        .unwrap();
    assert_eq!(unsafe { value.Value() }.unwrap().to_string(), "東京");
    assert!(!unsafe { value.IsReadOnly() }.unwrap().as_bool());
}

#[test]
fn native_selection_and_toggle_patterns_follow_roles_and_state() {
    let bridge = bridge();
    bridge.0.nodes.write().unwrap()[0].metadata.role = AccessibilityRole::Tab;
    let provider = bridge
        .provider(HWND::default(), Some(7))
        .unwrap()
        .cast::<IRawElementProviderSimple>()
        .unwrap();
    let selection = unsafe { provider.GetPatternProvider(UIA_SelectionItemPatternId) }
        .unwrap()
        .cast::<ISelectionItemProvider>()
        .unwrap();
    assert!(unsafe { selection.IsSelected() }.unwrap().as_bool());
    bridge.0.nodes.write().unwrap()[0].metadata.role = AccessibilityRole::CheckBox;
    let toggle = unsafe { provider.GetPatternProvider(UIA_TogglePatternId) }
        .unwrap()
        .cast::<IToggleProvider>()
        .unwrap();
    assert_eq!(unsafe { toggle.ToggleState() }.unwrap(), ToggleState_On);
    bridge.0.nodes.write().unwrap()[0].metadata.selected = Some(false);
    assert_eq!(unsafe { toggle.ToggleState() }.unwrap(), ToggleState_Off);
}

#[test]
fn fragment_root_preserves_its_com_identity() {
    let bridge = bridge();
    let root = bridge
        .provider(HWND::default(), None)
        .unwrap()
        .cast::<IRawElementProviderFragment>()
        .unwrap();
    let returned = unsafe { root.FragmentRoot() }.unwrap();
    assert_eq!(
        root.cast::<IUnknown>().unwrap(),
        returned.cast::<IUnknown>().unwrap()
    );
}

#[test]
fn fragment_focus_follows_current_snapshot_and_reports_keyboard_focus() {
    let bridge = bridge();
    let root = bridge
        .provider(HWND::default(), None)
        .unwrap()
        .cast::<IRawElementProviderFragmentRoot>()
        .unwrap();
    let focused = unsafe { root.GetFocus() }.unwrap();
    let focused = focused.cast::<IRawElementProviderSimple>().unwrap();
    let value = unsafe { focused.GetPropertyValue(UIA_HasKeyboardFocusPropertyId) }.unwrap();
    assert!(bool::try_from(&value).unwrap());
    bridge.0.nodes.write().unwrap()[0].focused = false;
    assert!(unsafe { root.GetFocus() }.is_err());
}

#[test]
fn closed_root_rejects_hwnd_reads_and_fragment_access() {
    let bridge = bridge();
    let root = bridge
        .provider(HWND::default(), None)
        .unwrap()
        .cast::<IRawElementProviderSimple>()
        .unwrap();
    bridge.close();
    let expected = HRESULT(UIA_E_ELEMENTNOTAVAILABLE as i32);
    assert_eq!(
        unsafe { root.GetPropertyValue(UIA_NamePropertyId) }
            .unwrap_err()
            .code(),
        expected
    );
    assert_eq!(
        unsafe { root.HostRawElementProvider() }.unwrap_err().code(),
        expected
    );
    let fragment = root.cast::<IRawElementProviderFragment>().unwrap();
    assert_eq!(
        unsafe { fragment.BoundingRectangle() }.unwrap_err().code(),
        expected
    );
    assert_eq!(
        unsafe { fragment.FragmentRoot() }.unwrap_err().code(),
        expected
    );
}

#[test]
fn navigation_and_focus_reuse_the_registered_root_and_child() {
    let bridge = bridge();
    let first = bridge.provider(HWND::default(), None).unwrap();
    let next = bridge.provider(HWND::default(), None).unwrap();
    assert_eq!(
        first.cast::<IUnknown>().unwrap(),
        next.cast::<IUnknown>().unwrap()
    );
    let root = first.cast::<IRawElementProviderFragmentRoot>().unwrap();
    let focused = unsafe { root.GetFocus() }.unwrap();
    let child = unsafe {
        first
            .cast::<IRawElementProviderFragment>()
            .unwrap()
            .Navigate(NavigateDirection_FirstChild)
    }
    .unwrap();
    assert_eq!(
        focused.cast::<IUnknown>().unwrap(),
        child.cast::<IUnknown>().unwrap()
    );
    assert_eq!(
        unsafe { child.FragmentRoot() }
            .unwrap()
            .cast::<IUnknown>()
            .unwrap(),
        first.cast::<IUnknown>().unwrap()
    );
}

#[test]
fn external_provider_references_do_not_keep_the_window_state_alive() {
    let bridge = bridge();
    let state = Arc::downgrade(&bridge.0);
    let provider = bridge
        .provider(HWND::default(), None)
        .unwrap()
        .cast::<IRawElementProviderSimple>()
        .unwrap();
    drop(bridge);
    assert!(state.upgrade().is_none());
    assert_eq!(
        unsafe { provider.GetPropertyValue(UIA_NamePropertyId) }
            .unwrap_err()
            .code(),
        HRESULT(UIA_E_ELEMENTNOTAVAILABLE as i32)
    );
}

#[test]
fn unpainted_children_are_pruned_from_the_identity_cache() {
    let bridge = bridge();
    let _root = bridge.provider(HWND::default(), None).unwrap();
    let child = bridge.provider(HWND::default(), Some(7)).unwrap();
    bridge.update(HWND::default(), Vec::new());
    assert_eq!(bridge.0.providers.lock().unwrap().len(), 1);
    assert!(child.node().is_err());
}

#[test]
fn child_provider_exposes_fragment_but_not_fragment_root() {
    let bridge = bridge();
    let child = bridge.provider(HWND::default(), Some(7)).unwrap().simple();
    assert!(child.cast::<IRawElementProviderFragment>().is_ok());
    assert_eq!(
        child
            .cast::<IRawElementProviderFragmentRoot>()
            .unwrap_err()
            .code(),
        E_NOINTERFACE
    );
}

#[test]
fn unsupported_pattern_returns_success_without_a_provider() {
    let bridge = bridge();
    for id in [None, Some(7)] {
        let provider = bridge.provider(HWND::default(), id).unwrap().simple();
        let mut result = std::ptr::null_mut();
        let status = unsafe {
            (provider.vtable().GetPatternProvider)(
                provider.as_raw(),
                UIA_GridPatternId,
                &mut result,
            )
        };
        assert!(status.is_ok());
        assert!(result.is_null());
    }
}

#[test]
fn root_and_virtual_children_report_the_owning_process() {
    let bridge = bridge();
    for id in [None, Some(7)] {
        let provider = bridge.provider(HWND::default(), id).unwrap().simple();
        let value = unsafe { provider.GetPropertyValue(UIA_ProcessIdPropertyId) }.unwrap();
        assert_eq!(i32::try_from(&value).unwrap(), std::process::id() as i32);
    }
}

#[test]
fn virtual_child_runtime_id_uses_append_prefix_and_all_stable_id_bits() {
    use windows::Win32::System::Ole::{
        SafeArrayGetDim, SafeArrayGetElement, SafeArrayGetLBound, SafeArrayGetUBound,
    };
    let bridge = bridge();
    let id = 0x1234_5678_9abc_def0;
    bridge.0.nodes.write().unwrap()[0].id = id;
    let child = bridge
        .provider(HWND::default(), Some(id))
        .unwrap()
        .cast::<IRawElementProviderFragment>()
        .unwrap();
    let array = unsafe { child.GetRuntimeId() }.unwrap();
    let dimensions = unsafe { SafeArrayGetDim(array) };
    let lower = unsafe { SafeArrayGetLBound(array, 1) }.unwrap();
    let upper = unsafe { SafeArrayGetUBound(array, 1) }.unwrap();
    let mut values = [0i32; 3];
    for (index, value) in values.iter_mut().enumerate() {
        unsafe { SafeArrayGetElement(array, &(index as i32), value as *mut _ as _) }.unwrap();
    }
    unsafe { SafeArrayDestroy(array) }.unwrap();
    assert_eq!((dimensions, lower, upper), (1, 0, 2));
    assert_eq!(
        values,
        [UiaAppendRuntimeId as i32, id as i32, (id >> 32) as i32]
    );
}
