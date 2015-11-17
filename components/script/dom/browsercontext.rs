/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::conversions::{ToJSValConvertible, root_from_handleobject};
use dom::bindings::js::{JS, Root};
use dom::bindings::proxyhandler::{fill_property_descriptor, get_property_descriptor};
use dom::bindings::reflector::{Reflectable, Reflector};
use dom::bindings::utils::WindowProxyHandler;
use dom::bindings::utils::get_array_index_from_id;
use dom::document::Document;
use dom::element::Element;
use dom::window::Window;
use js::JSCLASS_IS_GLOBAL;
use js::glue::{CreateWrapperProxyHandler, ProxyTraps, WrapperNew};
use js::glue::{GetProxyPrivate, SetProxyExtra};
use js::jsapi::{Handle, JS_ForwardSetPropertyTo, ObjectOpResult, RootedObject, RootedValue};
use js::jsapi::{HandleId, HandleObject, MutableHandle, MutableHandleValue};
use js::jsapi::{JSAutoCompartment, JSAutoRequest, JS_GetClass};
use js::jsapi::{JSContext, JSErrNum, JSObject, JSPropertyDescriptor};
use js::jsapi::{JS_AlreadyHasOwnPropertyById, JS_ForwardGetPropertyTo};
use js::jsapi::{JS_DefinePropertyById6, JS_GetPropertyDescriptorById};
use js::jsval::{ObjectValue, UndefinedValue, PrivateValue};
use std::ptr;

#[dom_struct]
pub struct BrowsingContext {
    reflector: Reflector,
    history: Vec<SessionHistoryEntry>,
    active_index: usize,
    frame_element: Option<JS<Element>>,
}

impl BrowsingContext {
    pub fn new_inherited(document: &Document, frame_element: Option<&Element>) -> BrowsingContext {
        BrowsingContext {
            reflector: Reflector::new(),
            history: vec![SessionHistoryEntry::new(document)],
            active_index: 0,
            frame_element: frame_element.map(JS::from_ref),
        }
    }

    #[allow(unsafe_code)]
    pub fn new(document: &Document, frame_element: Option<&Element>) -> Root<BrowsingContext> {
        unsafe {
            let window = document.window();

            let WindowProxyHandler(handler) = window.windowproxy_handler();
            assert!(!handler.is_null());

            let cx = window.get_cx();
            let _ar = JSAutoRequest::new(cx);
            let parent = window.reflector().get_jsobject();
            assert!(!parent.get().is_null());
            assert!(((*JS_GetClass(parent.get())).flags & JSCLASS_IS_GLOBAL) != 0);
            let _ac = JSAutoCompartment::new(cx, parent.get());
            let window_proxy = RootedObject::new(cx,
                                                 WrapperNew(cx,
                                                            parent,
                                                            handler,
                                                            ptr::null(),
                                                            true));
            assert!(!window_proxy.ptr.is_null());

            let object = box BrowsingContext::new_inherited(document, frame_element);

            let raw = Box::into_raw(object);
            SetProxyExtra(window_proxy.ptr, 0, PrivateValue(raw as *const _));

            (*raw).init_reflector(window_proxy.ptr);

            Root::from_ref(&*raw)
        }
    }

    pub fn active_document(&self) -> &Document {
        &*self.history[self.active_index].document
    }

    pub fn active_window(&self) -> &Window {
        self.active_document().window()
    }

    pub fn frame_element(&self) -> Option<&Element> {
        self.frame_element.as_ref().map(|element| &**element)
    }

    pub fn window_proxy(&self) -> *mut JSObject {
        let window_proxy = self.reflector.get_jsobject();
        assert!(!window_proxy.get().is_null());
        window_proxy.get()
    }
}

// This isn't a DOM struct, just a convenience struct
// without a reflector, so we don't mark this as #[dom_struct]
#[must_root]
#[privatize]
#[derive(JSTraceable, HeapSizeOf)]
pub struct SessionHistoryEntry {
    document: JS<Document>,
    children: Vec<JS<BrowsingContext>>,
}

impl SessionHistoryEntry {
    fn new(document: &Document) -> SessionHistoryEntry {
        SessionHistoryEntry {
            document: JS::from_ref(document),
            children: vec![],
        }
    }
}

#[allow(unsafe_code)]
unsafe fn GetSubframeWindow(cx: *mut JSContext,
                            proxy: HandleObject,
                            id: HandleId)
                            -> Option<Root<Window>> {
    let index = get_array_index_from_id(cx, id);
    if let Some(index) = index {
        let target = RootedObject::new(cx, GetProxyPrivate(*proxy.ptr).to_object());
        let win = root_from_handleobject::<Window>(target.handle()).unwrap();
        let mut found = false;
        return win.IndexedGetter(index, &mut found);
    }

    None
}

#[allow(unsafe_code)]
unsafe extern "C" fn getOwnPropertyDescriptor(cx: *mut JSContext,
                                              proxy: HandleObject,
                                              id: HandleId,
                                              desc: MutableHandle<JSPropertyDescriptor>)
                                              -> bool {
    let window = GetSubframeWindow(cx, proxy, id);
    if let Some(window) = window {
        let mut val = RootedValue::new(cx, UndefinedValue());
        window.to_jsval(cx, val.handle_mut());
        (*desc.ptr).value = val.ptr;
        fill_property_descriptor(&mut *desc.ptr, *proxy.ptr, true);
        return true;
    }

    let target = RootedObject::new(cx, GetProxyPrivate(*proxy.ptr).to_object());
    // XXX This should be JS_GetOwnPropertyDescriptorById
    if !JS_GetPropertyDescriptorById(cx, target.handle(), id, desc) {
        return false;
    }

    if (*desc.ptr).obj != target.ptr {
        // Not an own property
        (*desc.ptr).obj = ptr::null_mut();
    } else {
        (*desc.ptr).obj = *proxy.ptr;
    }

    true
}

#[allow(unsafe_code)]
unsafe extern "C" fn defineProperty(cx: *mut JSContext,
                                    proxy: HandleObject,
                                    id: HandleId,
                                    desc: Handle<JSPropertyDescriptor>,
                                    res: *mut ObjectOpResult)
                                    -> bool {
    if get_array_index_from_id(cx, id).is_some() {
        // Spec says to Reject whether this is a supported index or not,
        // since we have no indexed setter or indexed creator.  That means
        // throwing in strict mode (FIXME: Bug 828137), doing nothing in
        // non-strict mode.
        (*res).code_ = JSErrNum::JSMSG_CANT_DEFINE_WINDOW_ELEMENT as ::libc::uintptr_t;
        return true;
    }

    let target = RootedObject::new(cx, GetProxyPrivate(*proxy.ptr).to_object());
    JS_DefinePropertyById6(cx, target.handle(), id, desc, res)
}

#[allow(unsafe_code)]
unsafe extern "C" fn hasOwn(cx: *mut JSContext,
                            proxy: HandleObject,
                            id: HandleId,
                            bp: *mut bool)
                            -> bool {
    let window = GetSubframeWindow(cx, proxy, id);
    if window.is_some() {
        *bp = true;
        return true;
    }

    let target = RootedObject::new(cx, GetProxyPrivate(*proxy.ptr).to_object());
    let mut found = false;
    if !JS_AlreadyHasOwnPropertyById(cx, target.handle(), id, &mut found) {
        return false;
    }

    *bp = found;
    true
}

#[allow(unsafe_code)]
unsafe extern "C" fn get(cx: *mut JSContext,
                         proxy: HandleObject,
                         receiver: HandleObject,
                         id: HandleId,
                         vp: MutableHandleValue)
                         -> bool {
    let window = GetSubframeWindow(cx, proxy, id);
    if let Some(window) = window {
        window.to_jsval(cx, vp);
        return true;
    }

    let target = RootedObject::new(cx, GetProxyPrivate(*proxy.ptr).to_object());
    JS_ForwardGetPropertyTo(cx, target.handle(), id, receiver, vp)
}

#[allow(unsafe_code)]
unsafe extern "C" fn set(cx: *mut JSContext,
                         proxy: HandleObject,
                         receiver: HandleObject,
                         id: HandleId,
                         vp: MutableHandleValue,
                         res: *mut ObjectOpResult)
                         -> bool {
    if get_array_index_from_id(cx, id).is_some() {
        // Reject (which means throw if and only if strict) the set.
        (*res).code_ = JSErrNum::JSMSG_READ_ONLY as ::libc::uintptr_t;
        return true;
    }

    let target = RootedObject::new(cx, GetProxyPrivate(*proxy.ptr).to_object());
    let receiver = RootedValue::new(cx, ObjectValue(&**receiver.ptr));
    JS_ForwardSetPropertyTo(cx,
                            target.handle(),
                            id,
                            vp.to_handle(),
                            receiver.handle(),
                            res)
}

static PROXY_HANDLER: ProxyTraps = ProxyTraps {
    enter: None,
    getOwnPropertyDescriptor: Some(getOwnPropertyDescriptor),
    defineProperty: Some(defineProperty),
    ownPropertyKeys: None,
    delete_: None,
    enumerate: None,
    preventExtensions: None,
    isExtensible: None,
    has: None,
    get: Some(get),
    set: Some(set),
    call: None,
    construct: None,
    getPropertyDescriptor: Some(get_property_descriptor),
    hasOwn: Some(hasOwn),
    getOwnEnumerablePropertyKeys: None,
    nativeCall: None,
    hasInstance: None,
    objectClassIs: None,
    className: None,
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
pub fn new_window_proxy_handler() -> WindowProxyHandler {
    unsafe {
        WindowProxyHandler(CreateWrapperProxyHandler(&PROXY_HANDLER))
    }
}
