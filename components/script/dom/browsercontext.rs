/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::conversions::native_from_reflector_jsmanaged;
use dom::bindings::conversions::{ToJSValConvertible};
use dom::bindings::js::{JS, JSRef, Temporary, Root};
use dom::bindings::js::{OptionalRootable, OptionalOptionalRootable};
use dom::bindings::js::{ResultRootable, Rootable};
use dom::bindings::proxyhandler::{get_property_descriptor, fill_property_descriptor};
use dom::bindings::utils::{Reflectable, WindowProxyHandler};
use dom::bindings::utils::get_array_index_from_id;
use dom::document::{Document, DocumentHelpers};
use dom::element::Element;
use dom::window::Window;
use dom::window::WindowHelpers;

use js::jsapi::{JSContext, JSObject, JSPropertyDescriptor, JSErrNum};
use js::jsapi::{HandleObject, HandleId, MutableHandle, MutableHandleValue};
use js::jsapi::{JS_AlreadyHasOwnPropertyById, JS_ForwardGetPropertyTo};
use js::jsapi::{JS_GetPropertyDescriptorById, JS_DefinePropertyById6};
use js::jsapi::{JS_ForwardSetPropertyTo, ObjectOpResult, RootedObject, RootedValue, Handle, HandleValue};
use js::jsapi::{JSAutoRequest, JSAutoCompartment};
use js::jsval::ObjectValue;
use js::glue::{GetProxyPrivate};
use js::glue::{WrapperNew, CreateWrapperProxyHandler, ProxyTraps};

use std::ptr;

#[allow(raw_pointer_derive)]
#[jstraceable]
#[privatize]
pub struct BrowserContext {
    history: Vec<SessionHistoryEntry>,
    active_index: usize,
    window_proxy: *mut JSObject,
    frame_element: Option<JS<Element>>,
}

impl BrowserContext {
    pub fn new(document: JSRef<Document>, frame_element: Option<JSRef<Element>>) -> BrowserContext {
        let mut context = BrowserContext {
            history: vec!(SessionHistoryEntry::new(document)),
            active_index: 0,
            window_proxy: ptr::null_mut(),
            frame_element: frame_element.map(JS::from_rooted),
        };
        context.create_window_proxy();
        context
    }

    pub fn active_document(&self) -> Temporary<Document> {
        Temporary::from_rooted(self.history[self.active_index].document.clone())
    }

    pub fn active_window(&self) -> Temporary<Window> {
        let doc = self.active_document().root();
        doc.r().window()
    }

    pub fn frame_element(&self) -> Option<Temporary<Element>> {
        self.frame_element.map(Temporary::from_rooted)
    }

    pub fn window_proxy(&self) -> *mut JSObject {
        assert!(!self.window_proxy.is_null());
        self.window_proxy
    }

    #[allow(unsafe_code)]
    fn create_window_proxy(&mut self) {
        let win = self.active_window().root();
        let win = win.r();

        let WindowProxyHandler(handler) = win.windowproxy_handler();
        assert!(!handler.is_null());

        let cx = win.get_cx();
        let _ar = JSAutoRequest::new(cx);
        let parent = RootedObject::new(cx, win.reflector().get_jsobject());
        let _ac = JSAutoCompartment::new(cx, parent.ptr);
        let wrapper = unsafe { WrapperNew(cx, parent.handle(), handler) };
        assert!(!wrapper.is_null());
        self.window_proxy = wrapper;
    }
}

// This isn't a DOM struct, just a convenience struct
// without a reflector, so we don't mark this as #[dom_struct]
#[must_root]
#[privatize]
#[jstraceable]
pub struct SessionHistoryEntry {
    document: JS<Document>,
    children: Vec<BrowserContext>
}

impl SessionHistoryEntry {
    fn new(document: JSRef<Document>) -> SessionHistoryEntry {
        SessionHistoryEntry {
            document: JS::from_rooted(document),
            children: vec!()
        }
    }
}

#[allow(unsafe_code)]
unsafe fn GetSubframeWindow(cx: *mut JSContext, proxy: HandleObject, id: HandleId) -> Option<Temporary<Window>> {
    let index = get_array_index_from_id(cx, id);
    if let Some(index) = index {
        let target = RootedObject::new(cx, GetProxyPrivate(*proxy.ptr).to_object());
        let win: Root<Window> = native_from_reflector_jsmanaged(target.ptr).unwrap().root();
        let mut found = false;
        return win.r().IndexedGetter(index, &mut found);
    }

    None
}

#[allow(unsafe_code)]
unsafe extern fn getOwnPropertyDescriptor(cx: *mut JSContext, proxy: HandleObject, id: HandleId, desc: MutableHandle<JSPropertyDescriptor>) -> u8 {
    let window = GetSubframeWindow(cx, proxy, id);
    if let Some(window) = window {
        let window = window.root();
        (*desc.ptr).value = window.to_jsval(cx);
        fill_property_descriptor(&mut *desc.ptr, *proxy.ptr, true);
        return true as u8;
    }

    let target = RootedObject::new(cx, GetProxyPrivate(*proxy.ptr).to_object());
    // XXX This should be JS_GetOwnPropertyDescriptorById
    if JS_GetPropertyDescriptorById(cx, target.handle(), id, desc) == 0 {
        return false as u8;
    }

    if (*desc.ptr).obj != target.ptr {
        // Not an own property
        (*desc.ptr).obj = ptr::null_mut();
    } else {
        (*desc.ptr).obj = *proxy.ptr;
    }

    true as u8
}

#[allow(unsafe_code)]
unsafe extern fn defineProperty(cx: *mut JSContext,
                                proxy: HandleObject,
                                id: HandleId,
                                desc: Handle<JSPropertyDescriptor>,
                                res: *mut ObjectOpResult) -> u8 {
    if get_array_index_from_id(cx, id).is_some() {
        // Spec says to Reject whether this is a supported index or not,
        // since we have no indexed setter or indexed creator.  That means
        // throwing in strict mode (FIXME: Bug 828137), doing nothing in
        // non-strict mode.
       (*res).code_ = JSErrNum::JSMSG_CANT_DEFINE_WINDOW_ELEMENT as u32;
       return true as u8;
    }

    let target = RootedObject::new(cx, GetProxyPrivate(*proxy.ptr).to_object());
    JS_DefinePropertyById6(cx, target.handle(), id, desc, res)
}

#[allow(unsafe_code)]
unsafe extern fn hasOwn(cx: *mut JSContext, proxy: HandleObject, id: HandleId, bp: *mut u8) -> u8 {
    let window = GetSubframeWindow(cx, proxy, id);
    if window.is_some() {
        *bp = true as u8;
        return true as u8;
    }

    let target = RootedObject::new(cx, GetProxyPrivate(*proxy.ptr).to_object());
    let mut found = 0;
    if JS_AlreadyHasOwnPropertyById(cx, target.handle(), id, &mut found) == 0 {
        return false as u8;
    }

    *bp = (found != 0) as u8;
    return true as u8;
}

#[allow(unsafe_code)]
unsafe extern fn get(cx: *mut JSContext,
                     proxy: HandleObject,
                     receiver: HandleObject,
                     id: HandleId,
                     vp: MutableHandleValue) -> u8 {
    let window = GetSubframeWindow(cx, proxy, id);
    if let Some(window) = window {
        let window = window.root();
        *vp.ptr = window.to_jsval(cx);
        return true as u8;
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
                     res: *mut ObjectOpResult) -> u8 {
    if get_array_index_from_id(cx, id).is_some() {
        // Reject (which means throw if and only if strict) the set.
        (*res).code_ = JSErrNum::JSMSG_READ_ONLY as u32;
        return true as u8;
    }

    let target = RootedObject::new(cx, GetProxyPrivate(*proxy.ptr).to_object());
    let receiver = RootedValue::new(cx, ObjectValue(&**receiver.ptr));
    JS_ForwardSetPropertyTo(cx, target.handle(), id, HandleValue { ptr: vp.ptr }, receiver.handle(), res)
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
