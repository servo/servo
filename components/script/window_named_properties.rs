/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ptr;

use js::conversions::jsstr_to_string;
use js::glue::{AppendToIdVector, CreateProxyHandler, NewProxyObject, ProxyTraps};
use js::jsapi::{
    GetWellKnownSymbol, Handle, HandleId, HandleObject, JSClass, JSClass_NON_NATIVE, JSContext,
    JSErrNum, JS_SetImmutablePrototype, MutableHandle, MutableHandleIdVector, MutableHandleObject,
    ObjectOpResult, PropertyDescriptor, ProxyClassExtension, ProxyClassOps, ProxyObjectOps,
    SymbolCode, UndefinedHandleValue, JSCLASS_DELAY_METADATA_BUILDER, JSCLASS_IS_PROXY,
    JSCLASS_RESERVED_SLOTS_MASK, JSCLASS_RESERVED_SLOTS_SHIFT, JSPROP_READONLY,
};
use js::jsid::SymbolId;
use js::jsval::UndefinedValue;
use js::rust::{
    Handle as RustHandle, HandleObject as RustHandleObject, IntoHandle,
    MutableHandle as RustMutableHandle, MutableHandleObject as RustMutableHandleObject,
};

use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::proxyhandler::set_property_descriptor;
use crate::dom::bindings::root::Root;
use crate::dom::bindings::utils::has_property_on_prototype;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use crate::js::conversions::ToJSValConvertible;
use crate::script_runtime::JSContext as SafeJSContext;

struct SyncWrapper(*const libc::c_void);
#[allow(unsafe_code)]
unsafe impl Sync for SyncWrapper {}

lazy_static::lazy_static! {
    static ref HANDLER: SyncWrapper = {
        let traps = ProxyTraps {
            enter: None,
            getOwnPropertyDescriptor: Some(get_own_property_descriptor),
            defineProperty: Some(define_property),
            ownPropertyKeys: Some(own_property_keys),
            delete_: Some(delete),
            enumerate: None,
            getPrototypeIfOrdinary: Some(get_prototype_if_ordinary),
            getPrototype: None,
            setPrototype: None,
            setImmutablePrototype: None,
            preventExtensions: Some(prevent_extensions),
            isExtensible: Some(is_extensible),
            has: None,
            get: None,
            set: None,
            call: None,
            construct: None,
            hasOwn: None,
            getOwnEnumerablePropertyKeys: None,
            nativeCall: None,
            objectClassIs: None,
            className: Some(class_name),
            fun_toString: None,
            boxedValue_unbox: None,
            defaultValue: None,
            trace: None,
            finalize: None,
            objectMoved: None,
            isCallable: None,
            isConstructor: None,
        };

        #[allow(unsafe_code)]
        unsafe {
            SyncWrapper(CreateProxyHandler(&traps, ptr::null()))
        }
    };
}

#[allow(unsafe_code)]
unsafe extern "C" fn get_own_property_descriptor(
    cx: *mut JSContext,
    proxy: HandleObject,
    id: HandleId,
    desc: MutableHandle<PropertyDescriptor>,
    is_none: *mut bool,
) -> bool {
    let cx = SafeJSContext::from_ptr(cx);

    if id.is_symbol() {
        if id.get().asBits_ == SymbolId(GetWellKnownSymbol(*cx, SymbolCode::toStringTag)).asBits_ {
            rooted!(in(*cx) let mut rval = UndefinedValue());
            "WindowProperties".to_jsval(*cx, rval.handle_mut());
            set_property_descriptor(
                RustMutableHandle::from_raw(desc),
                rval.handle(),
                JSPROP_READONLY.into(),
                &mut *is_none,
            );
        }
        return true;
    }

    let mut found = false;
    if !has_property_on_prototype(
        *cx,
        RustHandle::from_raw(proxy),
        RustHandle::from_raw(id),
        &mut found,
    ) {
        return false;
    }
    if found {
        return true;
    }

    let s = if id.is_string() {
        jsstr_to_string(*cx, id.to_string())
    } else if id.is_int() {
        // If the property key is an integer index, convert it to a String too.
        // For indexed access on the window object, which may shadow this, see
        // the getOwnPropertyDescriptor trap in dom/windowproxy.rs.
        id.to_int().to_string()
    } else if id.is_symbol() {
        // Symbol properties were already handled above.
        unreachable!()
    } else {
        unimplemented!()
    };
    if s.is_empty() {
        return true;
    }

    let window = Root::downcast::<Window>(GlobalScope::from_object(proxy.get()))
        .expect("global is not a window");
    if let Some(obj) = window.NamedGetter(cx, s.into()) {
        rooted!(in(*cx) let mut rval = UndefinedValue());
        obj.to_jsval(*cx, rval.handle_mut());
        set_property_descriptor(
            RustMutableHandle::from_raw(desc),
            rval.handle(),
            0,
            &mut *is_none,
        );
    }
    true
}

#[allow(unsafe_code)]
unsafe extern "C" fn own_property_keys(
    cx: *mut JSContext,
    _proxy: HandleObject,
    props: MutableHandleIdVector,
) -> bool {
    // TODO is this all we need to return? compare with gecko:
    // https://searchfox.org/mozilla-central/rev/af78418c4b5f2c8721d1a06486cf4cf0b33e1e8d/dom/base/WindowNamedPropertiesHandler.cpp#175-232
    // see also https://github.com/whatwg/html/issues/9068
    rooted!(in(cx) let mut rooted = SymbolId(GetWellKnownSymbol(cx, SymbolCode::toStringTag)));
    AppendToIdVector(props, rooted.handle().into());
    true
}

#[allow(unsafe_code)]
unsafe extern "C" fn define_property(
    _cx: *mut JSContext,
    _proxy: HandleObject,
    _id: HandleId,
    _desc: Handle<PropertyDescriptor>,
    result: *mut ObjectOpResult,
) -> bool {
    (*result).code_ = JSErrNum::JSMSG_CANT_DEFINE_WINDOW_NAMED_PROPERTY as usize;
    true
}

#[allow(unsafe_code)]
unsafe extern "C" fn delete(
    _cx: *mut JSContext,
    _proxy: HandleObject,
    _id: HandleId,
    result: *mut ObjectOpResult,
) -> bool {
    (*result).code_ = JSErrNum::JSMSG_CANT_DELETE_WINDOW_NAMED_PROPERTY as usize;
    true
}

#[allow(unsafe_code)]
unsafe extern "C" fn get_prototype_if_ordinary(
    _cx: *mut JSContext,
    proxy: HandleObject,
    is_ordinary: *mut bool,
    proto: MutableHandleObject,
) -> bool {
    *is_ordinary = true;
    proto.set(js::jsapi::GetStaticPrototype(proxy.get()));
    true
}

#[allow(unsafe_code)]
unsafe extern "C" fn prevent_extensions(
    _cx: *mut JSContext,
    _proxy: HandleObject,
    result: *mut ObjectOpResult,
) -> bool {
    (*result).code_ = JSErrNum::JSMSG_CANT_PREVENT_EXTENSIONS as usize;
    true
}

#[allow(unsafe_code)]
unsafe extern "C" fn is_extensible(
    _cx: *mut JSContext,
    _proxy: HandleObject,
    extensible: *mut bool,
) -> bool {
    *extensible = true;
    true
}

#[allow(unsafe_code)]
unsafe extern "C" fn class_name(_cx: *mut JSContext, _proxy: HandleObject) -> *const libc::c_char {
    c"WindowProperties".as_ptr()
}

// Maybe this should be a DOMJSClass. See https://bugzilla.mozilla.org/show_bug.cgi?id=787070
#[allow(unsafe_code)]
static CLASS: JSClass = JSClass {
    name: c"WindowProperties".as_ptr(),
    flags: JSClass_NON_NATIVE |
        JSCLASS_IS_PROXY |
        JSCLASS_DELAY_METADATA_BUILDER |
        ((1 & JSCLASS_RESERVED_SLOTS_MASK) << JSCLASS_RESERVED_SLOTS_SHIFT), /* JSCLASS_HAS_RESERVED_SLOTS(1) */
    cOps: unsafe { &ProxyClassOps },
    spec: ptr::null(),
    ext: unsafe { &ProxyClassExtension },
    oOps: unsafe { &ProxyObjectOps },
};

#[allow(unsafe_code)]
pub fn create(
    cx: SafeJSContext,
    proto: RustHandleObject,
    mut properties_obj: RustMutableHandleObject,
) {
    unsafe {
        properties_obj.set(NewProxyObject(
            *cx,
            HANDLER.0,
            UndefinedHandleValue,
            proto.get(),
            &CLASS,
            false,
        ));
        assert!(!properties_obj.get().is_null());
        let mut succeeded = false;
        assert!(JS_SetImmutablePrototype(
            *cx,
            properties_obj.handle().into_handle(),
            &mut succeeded
        ));
        assert!(succeeded);
    }
}
