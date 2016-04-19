/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::conversions::{ToJSValConvertible, root_from_handleobject};
use dom::bindings::js::{JS, Root, RootedReference};
use dom::bindings::proxyhandler::{fill_property_descriptor, get_property_descriptor};
use dom::bindings::reflector::{Reflectable, Reflector};
use dom::bindings::utils::WindowProxyHandler;
use dom::bindings::utils::get_array_index_from_id;
use dom::document::Document;
use dom::element::Element;
use dom::window::Window;
use js::glue::{CreateWrapperProxyHandler, ProxyTraps, NewWindowProxy};
use js::glue::{GetProxyPrivate, SetProxyExtra};
use js::jsapi::{Handle, HandleId, HandleObject, JSAutoCompartment, JSAutoRequest, JSContext};
use js::jsapi::{JSErrNum, JSObject, JSPropertyDescriptor, JS_DefinePropertyById6};
use js::jsapi::{JS_ForwardGetPropertyTo, JS_ForwardSetPropertyTo, JS_GetClass};
use js::jsapi::{JS_GetOwnPropertyDescriptorById, JS_HasPropertyById, MutableHandle};
use js::jsapi::{MutableHandleValue, ObjectOpResult, RootedObject, RootedValue};
use js::jsval::{ObjectValue, UndefinedValue, PrivateValue};
use js::{JSCLASS_IS_GLOBAL, JSPROP_READONLY};
use msg::constellation_msg::PipelineId;
use script_runtime::ScriptChan;
use std::marker::PhantomData;

#[dom_struct]
pub struct BrowsingContext {
    reflector: Reflector,
    history: DOMRefCell<Vec<SessionHistoryEntry>>,
    active_index: usize,
    frame_element: Option<JS<Element>>,
}

impl BrowsingContext {
    pub fn new_inherited(frame_element: Option<&Element>) -> BrowsingContext {
        BrowsingContext {
            reflector: Reflector::new(),
            history: DOMRefCell::new(vec![]),
            active_index: 0,
            frame_element: frame_element.map(JS::from_ref),
        }
    }

    #[allow(unsafe_code)]
    pub fn new(window: &Window, frame_element: Option<&Element>) -> Root<BrowsingContext> {
        unsafe {
            let WindowProxyHandler(handler) = window.windowproxy_handler();
            assert!(!handler.is_null());

            let cx = window.get_cx();
            let _ar = JSAutoRequest::new(cx);
            let parent = window.reflector().get_jsobject();
            assert!(!parent.get().is_null());
            assert!(((*JS_GetClass(parent.get())).flags & JSCLASS_IS_GLOBAL) != 0);
            let _ac = JSAutoCompartment::new(cx, parent.get());
            let window_proxy = RootedObject::new(cx,
                NewWindowProxy(cx, parent, handler));
            assert!(!window_proxy.ptr.is_null());

            let object = box BrowsingContext::new_inherited(frame_element);

            let raw = Box::into_raw(object);
            SetProxyExtra(window_proxy.ptr, 0, PrivateValue(raw as *const _));

            (*raw).init_reflector(window_proxy.ptr);

            Root::from_ref(&*raw)
        }
    }

    pub fn init(&self, document: &Document) {
        assert!(self.history.borrow().is_empty());
        assert_eq!(self.active_index, 0);
        self.history.borrow_mut().push(SessionHistoryEntry::new(document));
    }

    pub fn active_document(&self) -> DocumentRoot {
        self.history.borrow()[self.active_index].document.root()
    }

    pub fn active_window(&self) -> WindowRoot {
        self.active_document().r().window().root()
    }

    pub fn frame_element(&self) -> Option<&Element> {
        self.frame_element.r()
    }

    pub fn window_proxy(&self) -> *mut JSObject {
        let window_proxy = self.reflector.get_jsobject();
        assert!(!window_proxy.get().is_null());
        window_proxy.get()
    }
}

#[derive(JSTraceable)]
pub struct RemoteDOMObject<T> {
    pipeline: PipelineId,
    event_loop: Box<ScriptChan + Send>,
    object: PhantomData<T>,
}

impl<T> RemoteDOMObject<T> {
    fn clone(&self) -> RemoteDOMObject<T> {
        RemoteDOMObject {
            pipeline: self.pipeline,
            event_loop: self.event_loop.clone(),
            object: PhantomData,
        }
    }
}

#[must_root]
#[derive(JSTraceable)]
pub enum DocumentField {
    Local(JS<Document>),
    Remote(RemoteDOMObject<Document>),
}

impl DocumentField {
    pub fn as_local(&self) -> &Document {
        match *self {
            DocumentField::Local(ref document) => &**document,
            DocumentField::Remote(_) => panic!("unexpected remote document"),
        }
    }

    pub fn root(&self) -> DocumentRoot {
        match *self {
            DocumentField::Local(ref document) => DocumentRoot::Local(Root::from_ref(&**document)),
            DocumentField::Remote(ref document) => DocumentRoot::Remote(document.clone()),
        }
    }

    pub fn r(&self) -> DocumentRef {
        match *self {
            DocumentField::Local(ref document) => DocumentRef::Local(&**document),
            DocumentField::Remote(ref document) => DocumentRef::Remote(document.clone()),
        }
    }
}

#[derive(JSTraceable)]
#[allow(unrooted_must_root)]
pub enum DocumentRoot {
    Local(Root<Document>),
    Remote(RemoteDOMObject<Document>),
}

impl DocumentRoot {
    pub fn as_local(self) -> Root<Document> {
        match self {
            DocumentRoot::Local(document) => document,
            DocumentRoot::Remote(_) => panic!("unexpected remote document"),
        }
    }

    pub fn r(&self) -> DocumentRef {
        match *self {
            DocumentRoot::Local(ref document) => DocumentRef::Local(&**document),
            DocumentRoot::Remote(ref document) => DocumentRef::Remote(document.clone()),
        }
    }
}

#[allow(unrooted_must_root)]
pub enum DocumentRef<'a> {
    Local(&'a Document),
    Remote(RemoteDOMObject<Document>),
}

impl<'a> DocumentRef<'a> {
    pub fn as_local(&self) -> &'a Document {
        match *self {
            DocumentRef::Local(document) => document,
            DocumentRef::Remote(_) => panic!("unexpected remote document"),
        }
    }

    pub fn window(&self) -> WindowRef<'a> {
        match *self {
            DocumentRef::Local(ref document) => WindowRef::Local(document.window()),
            DocumentRef::Remote(ref document) => {
                WindowRef::Remote(RemoteDOMObject {
                    pipeline: document.pipeline.clone(),
                    event_loop: document.event_loop.clone(),
                    object: PhantomData,
                })
            }
        }
    }
}

#[allow(unrooted_must_root)]
pub enum WindowRoot {
    Local(Root<Window>),
    Remote(RemoteDOMObject<Window>),
}

impl WindowRoot {
    pub fn as_local(self) -> Root<Window> {
        match self {
            WindowRoot::Local(window) => window,
            WindowRoot::Remote(_) => panic!("unexpected remote window"),
        }
    }
}

#[allow(unrooted_must_root)]
pub enum WindowRef<'a> {
    Local(&'a Window),
    Remote(RemoteDOMObject<Window>),
}

impl<'a> WindowRef<'a> {
    pub fn as_local(&self) -> &'a Window {
        match *self {
            WindowRef::Local(window) => window,
            WindowRef::Remote(_) => panic!("unexpected remote window"),
        }
    }

    pub fn root(&self) -> WindowRoot {
        match *self {
            WindowRef::Local(ref window) => WindowRoot::Local(Root::from_ref(&**window)),
            WindowRef::Remote(ref window) => WindowRoot::Remote(window.clone()),
        }
    }
}

// This isn't a DOM struct, just a convenience struct
// without a reflector, so we don't mark this as #[dom_struct]
#[must_root]
#[privatize]
#[derive(JSTraceable, HeapSizeOf)]
pub struct SessionHistoryEntry {
    #[ignore_heap_size_of = "XXXjdm"]
    document: DocumentField,
    children: Vec<JS<BrowsingContext>>,
}

impl SessionHistoryEntry {
    fn new(document: &Document) -> SessionHistoryEntry {
        SessionHistoryEntry {
            document: DocumentField::Local(JS::from_ref(document)),
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
        fill_property_descriptor(&mut *desc.ptr, *proxy.ptr, JSPROP_READONLY);
        return true;
    }

    let target = RootedObject::new(cx, GetProxyPrivate(*proxy.ptr).to_object());
    if !JS_GetOwnPropertyDescriptorById(cx, target.handle(), id, desc) {
        return false;
    }

    assert!(desc.get().obj.is_null() || desc.get().obj == target.ptr);
    if desc.get().obj == target.ptr {
        desc.get().obj = *proxy.ptr;
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
unsafe extern "C" fn has(cx: *mut JSContext,
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
    if !JS_HasPropertyById(cx, target.handle(), id, &mut found) {
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
    has: Some(has),
    get: Some(get),
    set: Some(set),
    call: None,
    construct: None,
    getPropertyDescriptor: Some(get_property_descriptor),
    hasOwn: None,
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
