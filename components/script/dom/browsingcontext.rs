/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::conversions::{ToJSValConvertible, root_from_handleobject};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableHeap, Root, RootedReference};
use dom::bindings::proxyhandler::{fill_property_descriptor, get_property_descriptor};
use dom::bindings::reflector::{Reflectable, MutReflectable, Reflector};
use dom::bindings::str::DOMString;
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::WindowProxyHandler;
use dom::bindings::utils::get_array_index_from_id;
use dom::document::Document;
use dom::element::Element;
use dom::globalscope::GlobalScope;
use dom::window::Window;
use js::JSCLASS_IS_GLOBAL;
use js::glue::{CreateWrapperProxyHandler, ProxyTraps, NewWindowProxy};
use js::glue::{GetProxyPrivate, SetProxyExtra, GetProxyExtra};
use js::jsapi::{Handle, HandleId, HandleObject, HandleValue, Heap};
use js::jsapi::{JSAutoCompartment, JSContext, JSErrNum, JSFreeOp, JSObject};
use js::jsapi::{JSPROP_READONLY, JSTracer, JS_DefinePropertyById};
use js::jsapi::{JS_ForwardGetPropertyTo, JS_ForwardSetPropertyTo, JS_GetClass};
use js::jsapi::{JS_GetOwnPropertyDescriptorById, JS_HasPropertyById};
use js::jsapi::{MutableHandle, MutableHandleObject, MutableHandleValue};
use js::jsapi::{ObjectOpResult, PropertyDescriptor};
use js::jsval::{JSVal, PrivateValue, UndefinedValue};
use msg::constellation_msg::{HistoryStateId, PipelineId};
use std::cell::Cell;
use std::collections::HashMap;
use url::Url;

#[dom_struct]
// NOTE: the browsing context for a window is managed in two places:
// here, in script, but also in the constellation. The constellation
// manages the session history, which in script is accessed through
// History objects, messaging the constellation.
pub struct BrowsingContext {
    reflector: Reflector,

    /// Pipeline id associated with this context.
    id: PipelineId,

    /// Indicates if reflow is required when reloading.
    needs_reflow: Cell<bool>,

    /// Stores the child browsing contexts (ex. iframe browsing context)
    children: DOMRefCell<Vec<JS<BrowsingContext>>>,

    /// The current active document.
    /// Note that the session history is stored in the constellation,
    /// in the script thread we just track the current active document.
    active_document: MutNullableHeap<JS<Document>>,

    active_state: HistoryStateId,

    next_state_id: HistoryStateId,

    states: HashMap<HistoryStateId, HistoryState>,

    /// The containing iframe element, if this is a same-origin iframe
    frame_element: Option<JS<Element>>,
}

impl BrowsingContext {
    pub fn new_inherited(frame_element: Option<&Element>, id: PipelineId) -> BrowsingContext {
        let mut states = HashMap::new();
        states.insert(HistoryStateId(0), HistoryState::new());
        BrowsingContext {
            reflector: Reflector::new(),
            id: id,
            needs_reflow: Cell::new(true),
            children: DOMRefCell::new(vec![]),
            active_document: Default::default(),
            active_state: HistoryStateId(0),
            next_state_id: HistoryStateId(1),
            states: states,
            frame_element: frame_element.map(JS::from_ref),
        }
    }

    #[allow(unsafe_code)]
    pub fn new(window: &Window, frame_element: Option<&Element>, id: PipelineId) -> Root<BrowsingContext> {
        unsafe {
            let WindowProxyHandler(handler) = window.windowproxy_handler();
            assert!(!handler.is_null());

            let cx = window.get_cx();
            let parent = window.reflector().get_jsobject();
            assert!(!parent.get().is_null());
            assert!(((*JS_GetClass(parent.get())).flags & JSCLASS_IS_GLOBAL) != 0);
            let _ac = JSAutoCompartment::new(cx, parent.get());
            rooted!(in(cx) let window_proxy = NewWindowProxy(cx, parent, handler));
            assert!(!window_proxy.is_null());

            let object = box BrowsingContext::new_inherited(frame_element, id);

            let raw = Box::into_raw(object);
            SetProxyExtra(window_proxy.get(), 0, &PrivateValue(raw as *const _));

            (*raw).init_reflector(window_proxy.get());

            Root::from_ref(&*raw)
        }
    }

    pub fn set_active_document(&self, document: &Document) {
        self.active_document.set(Some(document))
    }

    pub fn replace_session_history_entry(&self,
                                         title: Option<DOMString>,
                                         url: Option<Url>,
                                         state: HandleValue) {
        // let document = &*self.active_document();
        // let mut history = self.history.borrow_mut();
        // let url = match url {
        //     Some(url) => url,
        //     None => document.url().clone(),
        // };
        // let title = match title {
        //     Some(title) => title,
        //     None => document.Title(),
        // };
        // // TODO(ConnorGBrewster):
        // // Set Document's Url to url
        // // see: https://html.spec.whatwg.org/multipage/browsers.html#dom-history-pushstate Step 10
        // // Currently you can't mutate document.url
        // history[self.active_index.get()] = SessionHistoryEntry::new(document, url, title, Some(state));
    }

    pub fn push_session_history_entry(&self,
                                        document: &Document,
                                        title: Option<DOMString>,
                                        url: Option<Url>,
                                        state: Option<HandleValue>) {
    }

    pub fn active_document(&self) -> Root<Document> {
        self.active_document.get().expect("No active document.")
    }

    pub fn maybe_active_document(&self) -> Option<Root<Document>> {
        self.active_document.get()
    }

    pub fn active_window(&self) -> Root<Window> {
        Root::from_ref(self.active_document().window())
    }

    pub fn state(&self) -> JSVal {
        self.states.get(&self.active_state).expect("No active state.").state.get()
    }

    pub fn frame_element(&self) -> Option<&Element> {
        self.frame_element.r()
    }

    pub fn window_proxy(&self) -> *mut JSObject {
        let window_proxy = self.reflector.get_jsobject();
        assert!(!window_proxy.get().is_null());
        window_proxy.get()
    }

    pub fn remove(&self, id: PipelineId) -> Option<Root<BrowsingContext>> {
        let remove_idx = self.children
                             .borrow()
                             .iter()
                             .position(|context| context.id == id);
        match remove_idx {
            Some(idx) => Some(Root::from_ref(&*self.children.borrow_mut().remove(idx))),
            None => {
                self.children
                    .borrow_mut()
                    .iter_mut()
                    .filter_map(|context| context.remove(id))
                    .next()
            }
        }
    }

    pub fn set_reflow_status(&self, status: bool) -> bool {
        let old = self.needs_reflow.get();
        self.needs_reflow.set(status);
        old
    }

    pub fn pipeline_id(&self) -> PipelineId {
        self.id
    }

    pub fn push_child_context(&self, context: &BrowsingContext) {
        self.children.borrow_mut().push(JS::from_ref(&context));
    }

    pub fn find_child_by_id(&self, pipeline_id: PipelineId) -> Option<Root<Window>> {
        self.children.borrow().iter().find(|context| {
            let window = context.active_window();
            window.upcast::<GlobalScope>().pipeline_id() == pipeline_id
        }).map(|context| context.active_window())
    }

    pub fn unset_active_document(&self) {
        self.active_document.set(None)
    }

    pub fn iter(&self) -> ContextIterator {
        ContextIterator {
            stack: vec!(Root::from_ref(self)),
        }
    }

    pub fn find(&self, id: PipelineId) -> Option<Root<BrowsingContext>> {
        if self.id == id {
            return Some(Root::from_ref(self));
        }

        self.children.borrow()
                     .iter()
                     .filter_map(|c| c.find(id))
                     .next()
    }
}

#[derive(JSTraceable, HeapSizeOf)]
struct HistoryState {
    title: Option<DOMString>,
    url: Option<Url>,
    state: Heap<JSVal>,
}

impl HistoryState {
    fn new() -> HistoryState {
        let mut jsval: Heap<JSVal> = Default::default();
        let state = HandleValue::null();
        jsval.set(state.get());
        HistoryState {
            title: None,
            url: None,
            state: jsval,
        }
    }
}

pub struct ContextIterator {
    stack: Vec<Root<BrowsingContext>>,
}

impl Iterator for ContextIterator {
    type Item = Root<BrowsingContext>;

    fn next(&mut self) -> Option<Root<BrowsingContext>> {
        let popped = self.stack.pop();
        if let Some(ref context) = popped {
            self.stack.extend(context.children.borrow()
                                              .iter()
                                              .map(|c| Root::from_ref(&**c)));
        }
        popped
    }
}

#[allow(unsafe_code)]
unsafe fn GetSubframeWindow(cx: *mut JSContext,
                            proxy: HandleObject,
                            id: HandleId)
                            -> Option<Root<Window>> {
    let index = get_array_index_from_id(cx, id);
    if let Some(index) = index {
        rooted!(in(cx) let target = GetProxyPrivate(*proxy.ptr).to_object());
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
                                              mut desc: MutableHandle<PropertyDescriptor>)
                                              -> bool {
    let window = GetSubframeWindow(cx, proxy, id);
    if let Some(window) = window {
        rooted!(in(cx) let mut val = UndefinedValue());
        window.to_jsval(cx, val.handle_mut());
        desc.value = val.get();
        fill_property_descriptor(desc, proxy.get(), JSPROP_READONLY);
        return true;
    }

    rooted!(in(cx) let target = GetProxyPrivate(proxy.get()).to_object());
    if !JS_GetOwnPropertyDescriptorById(cx, target.handle(), id, desc) {
        return false;
    }

    assert!(desc.obj.is_null() || desc.obj == target.get());
    if desc.obj == target.get() {
        // FIXME(#11868) Should assign to desc.obj, desc.get() is a copy.
        desc.get().obj = proxy.get();
    }

    true
}

#[allow(unsafe_code)]
unsafe extern "C" fn defineProperty(cx: *mut JSContext,
                                    proxy: HandleObject,
                                    id: HandleId,
                                    desc: Handle<PropertyDescriptor>,
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

    rooted!(in(cx) let target = GetProxyPrivate(*proxy.ptr).to_object());
    JS_DefinePropertyById(cx, target.handle(), id, desc, res)
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

    rooted!(in(cx) let target = GetProxyPrivate(*proxy.ptr).to_object());
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
                         receiver: HandleValue,
                         id: HandleId,
                         vp: MutableHandleValue)
                         -> bool {
    let window = GetSubframeWindow(cx, proxy, id);
    if let Some(window) = window {
        window.to_jsval(cx, vp);
        return true;
    }

    rooted!(in(cx) let target = GetProxyPrivate(*proxy.ptr).to_object());
    JS_ForwardGetPropertyTo(cx, target.handle(), id, receiver, vp)
}

#[allow(unsafe_code)]
unsafe extern "C" fn set(cx: *mut JSContext,
                         proxy: HandleObject,
                         id: HandleId,
                         v: HandleValue,
                         receiver: HandleValue,
                         res: *mut ObjectOpResult)
                         -> bool {
    if get_array_index_from_id(cx, id).is_some() {
        // Reject (which means throw if and only if strict) the set.
        (*res).code_ = JSErrNum::JSMSG_READ_ONLY as ::libc::uintptr_t;
        return true;
    }

    rooted!(in(cx) let target = GetProxyPrivate(*proxy.ptr).to_object());
    JS_ForwardSetPropertyTo(cx,
                            target.handle(),
                            id,
                            v,
                            receiver,
                            res)
}

#[allow(unsafe_code)]
unsafe extern "C" fn get_prototype_if_ordinary(_: *mut JSContext,
                                               _: HandleObject,
                                               is_ordinary: *mut bool,
                                               _: MutableHandleObject)
                                               -> bool {
    // Window's [[GetPrototypeOf]] trap isn't the ordinary definition:
    //
    //   https://html.spec.whatwg.org/multipage/#windowproxy-getprototypeof
    //
    // We nonetheless can implement it with a static [[Prototype]], because
    // wrapper-class handlers (particularly, XOW in FilteringWrapper.cpp) supply
    // all non-ordinary behavior.
    //
    // But from a spec point of view, it's the exact same object in both cases --
    // only the observer's changed.  So this getPrototypeIfOrdinary trap on the
    // non-wrapper object *must* report non-ordinary, even if static [[Prototype]]
    // usually means ordinary.
    *is_ordinary = false;
    return true;
}

static PROXY_HANDLER: ProxyTraps = ProxyTraps {
    enter: None,
    getOwnPropertyDescriptor: Some(getOwnPropertyDescriptor),
    defineProperty: Some(defineProperty),
    ownPropertyKeys: None,
    delete_: None,
    enumerate: None,
    getPrototypeIfOrdinary: Some(get_prototype_if_ordinary),
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
    trace: Some(trace),
    finalize: Some(finalize),
    objectMoved: None,
    isCallable: None,
    isConstructor: None,
};

#[allow(unsafe_code)]
unsafe extern fn finalize(_fop: *mut JSFreeOp, obj: *mut JSObject) {
    let this = GetProxyExtra(obj, 0).to_private() as *mut BrowsingContext;
    assert!(!this.is_null());
    let _ = Box::from_raw(this);
    debug!("BrowsingContext finalize: {:p}", this);
}

#[allow(unsafe_code)]
unsafe extern fn trace(trc: *mut JSTracer, obj: *mut JSObject) {
    let this = GetProxyExtra(obj, 0).to_private() as *const BrowsingContext;
    if this.is_null() {
        // GC during obj creation
        return;
    }
    (*this).trace(trc);
}

#[allow(unsafe_code)]
pub fn new_window_proxy_handler() -> WindowProxyHandler {
    unsafe {
        WindowProxyHandler(CreateWrapperProxyHandler(&PROXY_HANDLER))
    }
}
