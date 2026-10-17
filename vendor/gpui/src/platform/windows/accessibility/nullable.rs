use super::*;
use std::{ffi::c_void, ptr::NonNull};

// windows-rs models nullable interface returns as Result<I>; initialize their ABI output explicitly.
unsafe fn output(result: *mut *mut c_void, call: impl FnOnce() -> HRESULT) -> HRESULT {
    if result.is_null() {
        return E_POINTER;
    }
    unsafe { result.write(std::ptr::null_mut()) };
    let status = call();
    if status == S_FALSE { S_OK } else { status }
}

const fn simple<I: IRawElementProviderSimple_Impl, const OFFSET: isize>()
-> IRawElementProviderSimple_Vtbl {
    unsafe extern "system" fn pattern<I: IRawElementProviderSimple_Impl, const OFFSET: isize>(
        this: *mut c_void,
        id: UIA_PATTERN_ID,
        result: *mut *mut c_void,
    ) -> HRESULT {
        unsafe {
            output(result, || {
                (IRawElementProviderSimple_Vtbl::new::<I, OFFSET>().GetPatternProvider)(
                    this, id, result,
                )
            })
        }
    }
    unsafe extern "system" fn host<I: IRawElementProviderSimple_Impl, const OFFSET: isize>(
        this: *mut c_void,
        result: *mut *mut c_void,
    ) -> HRESULT {
        unsafe {
            output(result, || {
                (IRawElementProviderSimple_Vtbl::new::<I, OFFSET>().HostRawElementProvider)(
                    this, result,
                )
            })
        }
    }
    let mut table = IRawElementProviderSimple_Vtbl::new::<I, OFFSET>();
    table.GetPatternProvider = pattern::<I, OFFSET>;
    table.HostRawElementProvider = host::<I, OFFSET>;
    table
}

const fn fragment<I: IRawElementProviderFragment_Impl, const OFFSET: isize>()
-> IRawElementProviderFragment_Vtbl {
    unsafe extern "system" fn navigate<I: IRawElementProviderFragment_Impl, const OFFSET: isize>(
        this: *mut c_void,
        direction: NavigateDirection,
        result: *mut *mut c_void,
    ) -> HRESULT {
        unsafe {
            output(result, || {
                (IRawElementProviderFragment_Vtbl::new::<I, OFFSET>().Navigate)(
                    this, direction, result,
                )
            })
        }
    }
    unsafe extern "system" fn root<I: IRawElementProviderFragment_Impl, const OFFSET: isize>(
        this: *mut c_void,
        result: *mut *mut c_void,
    ) -> HRESULT {
        unsafe {
            output(result, || {
                (IRawElementProviderFragment_Vtbl::new::<I, OFFSET>().FragmentRoot)(this, result)
            })
        }
    }
    let mut table = IRawElementProviderFragment_Vtbl::new::<I, OFFSET>();
    table.Navigate = navigate::<I, OFFSET>;
    table.FragmentRoot = root::<I, OFFSET>;
    table
}

const fn fragment_root<I: IRawElementProviderFragmentRoot_Impl, const OFFSET: isize>()
-> IRawElementProviderFragmentRoot_Vtbl {
    unsafe extern "system" fn focus<
        I: IRawElementProviderFragmentRoot_Impl,
        const OFFSET: isize,
    >(
        this: *mut c_void,
        result: *mut *mut c_void,
    ) -> HRESULT {
        unsafe {
            output(result, || {
                (IRawElementProviderFragmentRoot_Vtbl::new::<I, OFFSET>().GetFocus)(this, result)
            })
        }
    }
    unsafe extern "system" fn point<
        I: IRawElementProviderFragmentRoot_Impl,
        const OFFSET: isize,
    >(
        this: *mut c_void,
        x: f64,
        y: f64,
        result: *mut *mut c_void,
    ) -> HRESULT {
        unsafe {
            output(result, || {
                (IRawElementProviderFragmentRoot_Vtbl::new::<I, OFFSET>().ElementProviderFromPoint)(
                    this, x, y, result,
                )
            })
        }
    }
    let mut table = IRawElementProviderFragmentRoot_Vtbl::new::<I, OFFSET>();
    table.GetFocus = focus::<I, OFFSET>;
    table.ElementProviderFromPoint = point::<I, OFFSET>;
    table
}

static CHILD_SIMPLE: IRawElementProviderSimple_Vtbl = simple::<Provider_Impl, -1>();
static CHILD_FRAGMENT: IRawElementProviderFragment_Vtbl = fragment::<Provider_Impl, -2>();
static ROOT_SIMPLE: IRawElementProviderSimple_Vtbl = simple::<RootProvider_Impl, -1>();
static ROOT_FRAGMENT: IRawElementProviderFragment_Vtbl = fragment::<RootProvider_Impl, -2>();
static ROOT_FRAGMENT_ROOT: IRawElementProviderFragmentRoot_Vtbl =
    fragment_root::<RootProvider_Impl, -3>();

pub(super) fn child(provider: Provider) -> ComObject<Provider> {
    let mut outer = provider.into_outer();
    outer.interface1_irawelementproviders = &CHILD_SIMPLE;
    outer.interface2_irawelementproviderf = &CHILD_FRAGMENT;
    // Keep the generated allocation/refcount layout while replacing only nullable-return callbacks.
    unsafe { ComObject::from_raw(NonNull::from(Box::leak(Box::new(outer)))) }
}

pub(super) fn root(provider: RootProvider) -> ComObject<RootProvider> {
    let mut outer = provider.into_outer();
    outer.interface1_irawelementproviders = &ROOT_SIMPLE;
    outer.interface2_irawelementproviderf = &ROOT_FRAGMENT;
    outer.interface3_irawelementproviderf = &ROOT_FRAGMENT_ROOT;
    unsafe { ComObject::from_raw(NonNull::from(Box::leak(Box::new(outer)))) }
}
