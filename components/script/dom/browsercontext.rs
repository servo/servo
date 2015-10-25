/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::conversions::{ToJSValConvertible, root_from_handleobject};
use dom::bindings::js::{JS, Root};
use dom::bindings::proxyhandler::{fill_property_descriptor, get_property_descriptor};
use dom::bindings::utils::get_array_index_from_id;
use dom::bindings::utils::{Reflectable, WindowProxyHandler};
use dom::document::Document;
use dom::element::Element;
use dom::window::Window;
use js::glue::{CreateWrapperProxyHandler, ProxyTraps, WrapperNew};
use js::glue::{GetProxyPrivate};
use js::jsapi::{Handle, Heap, JS_ForwardSetPropertyTo, ObjectOpResult, RootedObject, RootedValue};
use js::jsapi::{HandleId, HandleObject, MutableHandle, MutableHandleValue};
use js::jsapi::{JSAutoCompartment, JSAutoRequest};
use js::jsapi::{JSContext, JSErrNum, JSObject, JSPropertyDescriptor};
use js::jsapi::{JS_AlreadyHasOwnPropertyById, JS_ForwardGetPropertyTo};
use js::jsapi::{JS_DefinePropertyById6, JS_GetPropertyDescriptorById};
use js::jsval::{ObjectValue, UndefinedValue};
use std::default::Default;
use std::ptr;

#[derive(JSTraceable, HeapSizeOf)]
#[privatize]
#[allow(raw_pointer_derive)]
#[must_root]
pub struct BrowsingContext {
    history: Vec<SessionHistoryEntry>,
    active_index: usize,
    window_proxy: Heap<*mut JSObject>,
    frame_element: Option<JS<Element>>,
}

impl BrowsingContext {
    pub fn new(document: &Document, frame_element: Option<&Element>) -> BrowsingContext {
        BrowsingContext {
            history: vec!(SessionHistoryEntry::new(document)),
            active_index: 0,
            window_proxy: Heap::default(),
            frame_element: frame_element.map(JS::from_ref),
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
        assert!(!self.window_proxy.get().is_null());
        self.window_proxy.get()
    }

    #[allow(unsafe_code)]
    pub fn create_window_proxy(&mut self) {
        // We inline self.active_window() because we can't borrow *self here.
        let win = self.history[self.active_index].document.window();

        let WindowProxyHandler(handler) = win.windowproxy_handler();
        assert!(!handler.is_null());

        let cx = win.get_cx();
        let _ar = JSAutoRequest::new(cx);
        let parent = win.reflector().get_jsobject();
        let _ac = JSAutoCompartment::new(cx, parent.get());
        let wrapper = unsafe { WrapperNew(cx, parent, handler, ptr::null(), false) };
        assert!(!wrapper.is_null());
        self.window_proxy.set(wrapper);
    }
}

// This isn't a DOM struct, just a convenience struct
// without a reflector, so we don't mark this as #[dom_struct]
#[must_root]
#[privatize]
#[derive(JSTraceable, HeapSizeOf)]
pub struct SessionHistoryEntry {
    document: JS<Document>,
    children: Vec<BrowsingContext>
}

impl SessionHistoryEntry {
    fn new(document: &Document) -> SessionHistoryEntry {
        SessionHistoryEntry {
            document: JS::from_ref(document),
            children: vec!()
        }
    }
}

#[allow(unsafe_code)]
unsafe fn GetSubframeWindow(cx: *mut JSContext, proxy: HandleObject, id: HandleId) -> Option<Root<Window>> {
    let index = get_array_index_from_id(cx, id);
    if let Some(index) = index {
        let target = RootedObject::new(cx, GetProxyPrivate(*proxy.ptr).to_object());
        let win = root_from_handleobject::<Window>(target.handle()).unwrap();
        let mut found = false;
        return win.r().IndexedGetter(index, &mut found);
    }

    None
}

#[allow(unsafe_code)]
unsafe extern fn getOwnPropertyDescriptor(cx: *mut JSContext, proxy: HandleObject, id: HandleId,
                                          desc: MutableHandle<JSPropertyDescriptor>) -> bool {
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
unsafe extern fn defineProperty(cx: *mut JSContext,
                                proxy: HandleObject,
                                id: HandleId,
                                desc: Handle<JSPropertyDescriptor>,
                                res: *mut ObjectOpResult) -> bool {
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
unsafe extern fn hasOwn(cx: *mut JSContext, proxy: HandleObject, id: HandleId, bp: *mut bool) -> bool {
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
unsafe extern fn get(cx: *mut JSContext,
                     proxy: HandleObject,
                     receiver: HandleObject,
                     id: HandleId,
                     vp: MutableHandleValue) -> bool {
    let window = GetSubframeWindow(cx, proxy, id);
    if let Some(window) = window {
        window.to_jsval(cx, vp);
        return true;
    }

    let target = RootedObject::new(cx, GetProxyPrivate(*proxy.ptr).to_object());
    JS_ForwardGetPropertyTo(cx, target.handle(), id, receiver, vp)
}

#[allow(unsafe_code)]
unsafe extern fn set(cx: *mut JSContext,
                     proxy: HandleObject,
                     receiver: HandleObject,
                     id: HandleId,
                     vp: MutableHandleValue,
                     res: *mut ObjectOpResult) -> bool {
    if get_array_index_from_id(cx, id).is_some() {
        // Reject (which means throw if and only if strict) the set.
        (*res).code_ = JSErrNum::JSMSG_READ_ONLY as ::libc::uintptr_t;
        return true;
    }

    let target = RootedObject::new(cx, GetProxyPrivate(*proxy.ptr).to_object());
    let receiver = RootedValue::new(cx, ObjectValue(&**receiver.ptr));
    JS_ForwardSetPropertyTo(cx, target.handle(), id, vp.to_handle(), receiver.handle(), res)
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
