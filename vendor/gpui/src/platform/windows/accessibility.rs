use crate::{AccessibilityAction, AccessibilityRole, AccessibilitySnapshot};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{
        Arc, Mutex, RwLock, Weak,
        atomic::{AtomicBool, Ordering},
    },
};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::ClientToScreen,
        System::{
            Com::SAFEARRAY,
            Ole::{SafeArrayCreateVector, SafeArrayDestroy, SafeArrayPutElement},
            Variant::{VARIANT, VT_I4},
        },
        UI::{Accessibility::*, HiDpi::GetDpiForWindow, WindowsAndMessaging::*},
    },
    core::*,
};

pub(crate) const WM_GPUI_ACCESSIBILITY: u32 = WM_USER + 8;

#[derive(Default)]
struct Shared {
    nodes: RwLock<Vec<AccessibilitySnapshot>>,
    requests: Mutex<VecDeque<(u64, AccessibilityAction)>>,
    providers: Mutex<HashMap<Option<u64>, CachedProvider>>,
    closed: AtomicBool,
}

#[derive(Clone, Default)]
pub(crate) struct AccessibilityBridge(Arc<Shared>);

impl AccessibilityBridge {
    pub fn update(&self, hwnd: HWND, nodes: Vec<AccessibilitySnapshot>) {
        if self.0.closed.load(Ordering::Acquire) {
            return;
        }
        let old_focus = self
            .0
            .nodes
            .read()
            .ok()
            .and_then(|nodes| nodes.iter().find(|n| n.focused).map(|n| n.id));
        let new_focus = nodes.iter().find(|n| n.focused).map(|n| n.id);
        let ids: HashSet<_> = nodes.iter().map(|n| n.id).collect();
        if let Ok(mut current) = self.0.nodes.write() {
            *current = nodes;
        }
        if let Ok(mut providers) = self.0.providers.lock() {
            providers.retain(|id, _| id.is_none_or(|id| ids.contains(&id)));
        }
        if old_focus != new_focus
            && let Some(id) = new_focus
            && let Ok(provider) = self.provider(hwnd, Some(id))
        {
            let provider = provider.simple();
            crate::measurement::measure("accessibility focus event duration", || unsafe {
                let _ = UiaRaiseAutomationEvent(&provider, UIA_AutomationFocusChangedEventId);
            });
        }
    }

    pub fn object(&self, hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> Option<isize> {
        if lparam.0 as i32 != UiaRootObjectId || self.0.closed.load(Ordering::Acquire) {
            return None;
        }
        let provider = self.provider(hwnd, None).ok()?.simple();
        Some(unsafe { UiaReturnRawElementProvider(hwnd, wparam, lparam, &provider).0 })
    }

    fn provider(&self, hwnd: HWND, id: Option<u64>) -> Result<CachedProvider> {
        let mut providers = self.0.providers.lock().map_err(|_| unavailable())?;
        if self.0.closed.load(Ordering::Acquire) {
            return Err(unavailable());
        }
        Ok(providers
            .entry(id)
            .or_insert_with(|| {
                let provider = Provider::new(hwnd, self, id);
                if id.is_none() {
                    CachedProvider::Root(RootProvider(provider).into_object())
                } else {
                    CachedProvider::Child(provider.into_object())
                }
            })
            .clone())
    }

    pub fn take_requests(&self) -> Vec<(u64, AccessibilityAction)> {
        self.0
            .requests
            .lock()
            .map(|mut pending| pending.drain(..).collect())
            .unwrap_or_default()
    }

    pub fn close(&self) {
        self.0.closed.store(true, Ordering::Release);
        if let Ok(mut providers) = self.0.providers.lock() {
            providers.clear();
        }
    }
}

#[implement(
    IRawElementProviderSimple,
    IRawElementProviderFragment,
    IInvokeProvider,
    IValueProvider,
    ISelectionItemProvider,
    IToggleProvider
)]
struct Provider {
    hwnd: usize,
    shared: Weak<Shared>,
    id: Option<u64>,
}

#[implement(
    IRawElementProviderSimple,
    IRawElementProviderFragment,
    IRawElementProviderFragmentRoot
)]
struct RootProvider(Provider);

impl std::ops::Deref for RootProvider {
    type Target = Provider;
    fn deref(&self) -> &Provider {
        &self.0
    }
}

#[derive(Clone)]
enum CachedProvider {
    Root(ComObject<RootProvider>),
    Child(ComObject<Provider>),
}

impl CachedProvider {
    fn simple(&self) -> IRawElementProviderSimple {
        match self {
            Self::Root(provider) => provider.to_interface(),
            Self::Child(provider) => provider.to_interface(),
        }
    }
    fn cast<I: Interface>(&self) -> Result<I> {
        self.simple().cast()
    }
}

impl std::ops::Deref for CachedProvider {
    type Target = Provider;
    fn deref(&self) -> &Provider {
        match self {
            Self::Root(provider) => &provider.0,
            Self::Child(provider) => provider,
        }
    }
}

impl Provider {
    fn new(hwnd: HWND, bridge: &AccessibilityBridge, id: Option<u64>) -> Self {
        Self {
            hwnd: hwnd.0 as usize,
            shared: Arc::downgrade(&bridge.0),
            id,
        }
    }
    fn hwnd(&self) -> HWND {
        HWND(self.hwnd as *mut _)
    }
    fn node(&self) -> Result<AccessibilitySnapshot> {
        self.shared()?
            .nodes
            .read()
            .ok()
            .and_then(|nodes| nodes.iter().find(|n| Some(n.id) == self.id).cloned())
            .ok_or_else(unavailable)
    }
    fn ensure_open(&self) -> Result<()> {
        self.shared().map(drop)
    }
    fn shared(&self) -> Result<Arc<Shared>> {
        self.shared
            .upgrade()
            .filter(|shared| !shared.closed.load(Ordering::Acquire))
            .ok_or_else(unavailable)
    }
    fn provider(&self, id: Option<u64>) -> Result<CachedProvider> {
        AccessibilityBridge(self.shared()?).provider(self.hwnd(), id)
    }
    fn fragment(&self, id: Option<u64>) -> Result<IRawElementProviderFragment> {
        self.provider(id)?.cast()
    }
    fn request(&self, action: AccessibilityAction) -> Result<()> {
        let node = self.node()?;
        if node.metadata.disabled {
            return Err(Error::from_hresult(HRESULT(UIA_E_ELEMENTNOTENABLED as i32)));
        }
        self.shared()?
            .requests
            .lock()
            .map_err(|_| unavailable())?
            .push_back((node.id, action));
        unsafe {
            PostMessageW(
                Some(self.hwnd()),
                WM_GPUI_ACCESSIBILITY,
                WPARAM(0),
                LPARAM(0),
            )
        }
    }
    fn property(&self, property: UIA_PROPERTY_ID) -> Result<VARIANT> {
        self.ensure_open()?;
        if property == UIA_ProcessIdPropertyId {
            return Ok(VARIANT::from(std::process::id() as i32));
        }
        if self.id.is_none() {
            return Ok(if property == UIA_NamePropertyId {
                let len = unsafe { GetWindowTextLengthW(self.hwnd()) }.max(0) as usize;
                let mut title = vec![0; len + 1];
                let count = unsafe { GetWindowTextW(self.hwnd(), &mut title) }.max(0) as usize;
                VARIANT::from(String::from_utf16_lossy(&title[..count]).as_str())
            } else if property == UIA_ControlTypePropertyId {
                VARIANT::from(UIA_PaneControlTypeId.0)
            } else if property == UIA_FrameworkIdPropertyId {
                VARIANT::from("GPUI")
            } else if property == UIA_IsControlElementPropertyId
                || property == UIA_IsContentElementPropertyId
            {
                VARIANT::from(true)
            } else {
                VARIANT::default()
            });
        }
        let node = self.node()?;
        let role = match node.metadata.role {
            AccessibilityRole::Button => UIA_ButtonControlTypeId,
            AccessibilityRole::Tab => UIA_TabItemControlTypeId,
            AccessibilityRole::ListItem => UIA_ListItemControlTypeId,
            AccessibilityRole::TextInput => UIA_EditControlTypeId,
            AccessibilityRole::CheckBox => UIA_CheckBoxControlTypeId,
            AccessibilityRole::StaticText => UIA_TextControlTypeId,
        };
        Ok(
            if property == UIA_NamePropertyId || property == UIA_HelpTextPropertyId {
                VARIANT::from(node.metadata.name.as_ref())
            } else if property == UIA_ControlTypePropertyId {
                VARIANT::from(role.0)
            } else if property == UIA_FrameworkIdPropertyId {
                VARIANT::from("GPUI")
            } else if property == UIA_AutomationIdPropertyId {
                VARIANT::from(format!("gpui-{}", node.id).as_str())
            } else if property == UIA_IsEnabledPropertyId {
                VARIANT::from(!node.metadata.disabled)
            } else if property == UIA_HasKeyboardFocusPropertyId {
                VARIANT::from(node.focused)
            } else if property == UIA_IsKeyboardFocusablePropertyId {
                VARIANT::from(node.focusable && !node.metadata.disabled)
            } else if property == UIA_IsControlElementPropertyId
                || property == UIA_IsContentElementPropertyId
            {
                VARIANT::from(true)
            } else if property == UIA_IsOffscreenPropertyId {
                VARIANT::from(false)
            } else if property == UIA_ValueValuePropertyId {
                node.metadata
                    .value
                    .as_ref()
                    .map_or_else(VARIANT::default, |v| VARIANT::from(v.as_ref()))
            } else if property == UIA_ValueIsReadOnlyPropertyId {
                VARIANT::from(!node.writable)
            } else if property == UIA_SelectionItemIsSelectedPropertyId {
                VARIANT::from(node.metadata.selected.unwrap_or(false))
            } else {
                VARIANT::default()
            },
        )
    }
    fn navigate(&self, direction: NavigateDirection) -> Result<IRawElementProviderFragment> {
        self.ensure_open()?;
        let shared = self.shared()?;
        let nodes = shared.nodes.read().map_err(|_| unavailable())?;
        if self.id.is_none() {
            let id = if direction == NavigateDirection_FirstChild {
                nodes.first()
            } else if direction == NavigateDirection_LastChild {
                nodes.last()
            } else {
                None
            }
            .map(|n| n.id);
            return id.map_or_else(absent, |id| self.fragment(Some(id)));
        }
        if direction == NavigateDirection_Parent {
            return self.fragment(None);
        }
        let index = nodes
            .iter()
            .position(|n| Some(n.id) == self.id)
            .ok_or_else(unavailable)?;
        let next = if direction == NavigateDirection_NextSibling {
            index.checked_add(1)
        } else if direction == NavigateDirection_PreviousSibling {
            index.checked_sub(1)
        } else {
            None
        };
        next.and_then(|i| nodes.get(i))
            .map_or_else(absent, |n| self.fragment(Some(n.id)))
    }
    fn rectangle(&self) -> Result<UiaRect> {
        self.ensure_open()?;
        let mut origin = POINT::default();
        unsafe {
            ClientToScreen(self.hwnd(), &mut origin).ok()?;
        }
        if self.id.is_none() {
            let mut bounds = RECT::default();
            unsafe {
                GetClientRect(self.hwnd(), &mut bounds)?;
            }
            return Ok(UiaRect {
                left: origin.x as f64,
                top: origin.y as f64,
                width: (bounds.right - bounds.left) as f64,
                height: (bounds.bottom - bounds.top) as f64,
            });
        }
        let node = self.node()?;
        let scale = unsafe { GetDpiForWindow(self.hwnd()) } as f64 / 96.;
        Ok(UiaRect {
            left: origin.x as f64 + f32::from(node.bounds.left()) as f64 * scale,
            top: origin.y as f64 + f32::from(node.bounds.top()) as f64 * scale,
            width: f32::from(node.bounds.size.width) as f64 * scale,
            height: f32::from(node.bounds.size.height) as f64 * scale,
        })
    }
}

fn unavailable() -> Error {
    Error::from_hresult(HRESULT(UIA_E_ELEMENTNOTAVAILABLE as i32))
}
fn absent<T>() -> Result<T> {
    Err(Error::from_hresult(S_FALSE))
}

#[allow(non_snake_case)]
impl IRawElementProviderSimple_Impl for Provider_Impl {
    fn ProviderOptions(&self) -> Result<ProviderOptions> {
        Ok(ProviderOptions_ServerSideProvider | ProviderOptions_UseComThreading)
    }
    fn GetPatternProvider(&self, pattern: UIA_PATTERN_ID) -> Result<IUnknown> {
        self.ensure_open()?;
        if self.id.is_none() {
            return absent();
        }
        let node = self.node()?;
        if pattern == UIA_InvokePatternId && node.invokable {
            return Ok(self.to_interface::<IInvokeProvider>().into());
        }
        if pattern == UIA_ValuePatternId && node.metadata.value.is_some() {
            return Ok(self.to_interface::<IValueProvider>().into());
        }
        if pattern == UIA_SelectionItemPatternId
            && matches!(
                node.metadata.role,
                AccessibilityRole::Tab | AccessibilityRole::ListItem
            )
        {
            return Ok(self.to_interface::<ISelectionItemProvider>().into());
        }
        if pattern == UIA_TogglePatternId && node.metadata.role == AccessibilityRole::CheckBox {
            return Ok(self.to_interface::<IToggleProvider>().into());
        }
        absent()
    }
    fn GetPropertyValue(&self, property: UIA_PROPERTY_ID) -> Result<VARIANT> {
        self.property(property)
    }
    fn HostRawElementProvider(&self) -> Result<IRawElementProviderSimple> {
        self.ensure_open()?;
        if self.id.is_none() {
            unsafe { UiaHostProviderFromHwnd(self.hwnd()) }
        } else {
            absent()
        }
    }
}

#[allow(non_snake_case)]
impl IRawElementProviderFragment_Impl for Provider_Impl {
    fn Navigate(&self, direction: NavigateDirection) -> Result<IRawElementProviderFragment> {
        self.navigate(direction)
    }
    fn GetRuntimeId(&self) -> Result<*mut SAFEARRAY> {
        self.ensure_open()?;
        let Some(id) = self.id else {
            return Ok(std::ptr::null_mut());
        };
        let values = [UiaAppendRuntimeId as i32, id as i32, (id >> 32) as i32];
        let array = unsafe { SafeArrayCreateVector(VT_I4, 0, 3) };
        if array.is_null() {
            return Err(E_OUTOFMEMORY.into());
        }
        for (index, value) in values.iter().enumerate() {
            if let Err(error) =
                unsafe { SafeArrayPutElement(array, &(index as i32), value as *const _ as _) }
            {
                unsafe {
                    let _ = SafeArrayDestroy(array);
                }
                return Err(error);
            }
        }
        Ok(array)
    }
    fn BoundingRectangle(&self) -> Result<UiaRect> {
        self.rectangle()
    }
    fn GetEmbeddedFragmentRoots(&self) -> Result<*mut SAFEARRAY> {
        self.ensure_open()?;
        Ok(std::ptr::null_mut())
    }
    fn SetFocus(&self) -> Result<()> {
        self.request(AccessibilityAction::Focus)
    }
    fn FragmentRoot(&self) -> Result<IRawElementProviderFragmentRoot> {
        self.ensure_open()?;
        self.provider(None)?.cast()
    }
}

#[allow(non_snake_case)]
impl IRawElementProviderSimple_Impl for RootProvider_Impl {
    fn ProviderOptions(&self) -> Result<ProviderOptions> {
        Ok(ProviderOptions_ServerSideProvider | ProviderOptions_UseComThreading)
    }
    fn GetPatternProvider(&self, _: UIA_PATTERN_ID) -> Result<IUnknown> {
        self.ensure_open()?;
        absent()
    }
    fn GetPropertyValue(&self, property: UIA_PROPERTY_ID) -> Result<VARIANT> {
        self.property(property)
    }
    fn HostRawElementProvider(&self) -> Result<IRawElementProviderSimple> {
        self.ensure_open()?;
        unsafe { UiaHostProviderFromHwnd(self.hwnd()) }
    }
}

#[allow(non_snake_case)]
impl IRawElementProviderFragment_Impl for RootProvider_Impl {
    fn Navigate(&self, direction: NavigateDirection) -> Result<IRawElementProviderFragment> {
        self.navigate(direction)
    }
    fn GetRuntimeId(&self) -> Result<*mut SAFEARRAY> {
        self.ensure_open()?;
        Ok(std::ptr::null_mut())
    }
    fn BoundingRectangle(&self) -> Result<UiaRect> {
        self.rectangle()
    }
    fn GetEmbeddedFragmentRoots(&self) -> Result<*mut SAFEARRAY> {
        self.ensure_open()?;
        Ok(std::ptr::null_mut())
    }
    fn SetFocus(&self) -> Result<()> {
        self.ensure_open()
    }
    fn FragmentRoot(&self) -> Result<IRawElementProviderFragmentRoot> {
        self.ensure_open()?;
        Ok(self.to_interface())
    }
}

#[allow(non_snake_case)]
impl IRawElementProviderFragmentRoot_Impl for RootProvider_Impl {
    fn ElementProviderFromPoint(&self, x: f64, y: f64) -> Result<IRawElementProviderFragment> {
        self.ensure_open()?;
        let nodes = self
            .shared()?
            .nodes
            .read()
            .map_err(|_| unavailable())?
            .clone();
        for node in nodes.iter().rev() {
            let provider = self.provider(Some(node.id))?;
            let bounds = provider.rectangle()?;
            if x >= bounds.left
                && x < bounds.left + bounds.width
                && y >= bounds.top
                && y < bounds.top + bounds.height
            {
                return provider.cast();
            }
        }
        self.fragment(None)
    }
    fn GetFocus(&self) -> Result<IRawElementProviderFragment> {
        self.ensure_open()?;
        let shared = self.shared()?;
        let nodes = shared.nodes.read().map_err(|_| unavailable())?;
        let focused = nodes.iter().find(|node| node.focused);
        focused.map_or_else(absent, |node| self.fragment(Some(node.id)))
    }
}

#[allow(non_snake_case)]
impl IInvokeProvider_Impl for Provider_Impl {
    fn Invoke(&self) -> Result<()> {
        self.request(AccessibilityAction::Invoke)
    }
}
#[allow(non_snake_case)]
impl IValueProvider_Impl for Provider_Impl {
    fn SetValue(&self, value: &PCWSTR) -> Result<()> {
        if !self.node()?.writable {
            return Err(E_ACCESSDENIED.into());
        }
        if value.is_null() {
            return Err(E_INVALIDARG.into());
        }
        self.request(AccessibilityAction::SetValue(unsafe { value.to_string()? }))
    }
    fn Value(&self) -> Result<BSTR> {
        Ok(BSTR::from(
            self.node()?
                .metadata
                .value
                .as_ref()
                .map_or("", |v| v.as_ref()),
        ))
    }
    fn IsReadOnly(&self) -> Result<BOOL> {
        Ok((!self.node()?.writable).into())
    }
}
#[allow(non_snake_case)]
impl ISelectionItemProvider_Impl for Provider_Impl {
    fn Select(&self) -> Result<()> {
        self.request(AccessibilityAction::Invoke)
    }
    fn AddToSelection(&self) -> Result<()> {
        self.Select()
    }
    fn RemoveFromSelection(&self) -> Result<()> {
        Err(Error::from_hresult(HRESULT(UIA_E_INVALIDOPERATION as i32)))
    }
    fn IsSelected(&self) -> Result<BOOL> {
        Ok(self.node()?.metadata.selected.unwrap_or(false).into())
    }
    fn SelectionContainer(&self) -> Result<IRawElementProviderSimple> {
        Ok(self.provider(None)?.simple())
    }
}
#[allow(non_snake_case)]
impl IToggleProvider_Impl for Provider_Impl {
    fn Toggle(&self) -> Result<()> {
        self.request(AccessibilityAction::Invoke)
    }
    fn ToggleState(&self) -> Result<ToggleState> {
        Ok(if self.node()?.metadata.selected.unwrap_or(false) {
            ToggleState_On
        } else {
            ToggleState_Off
        })
    }
}

#[cfg(test)]
#[path = "accessibility/tests.rs"]
mod tests;
