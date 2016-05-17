/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::LocationBinding::LocationMethods;
use dom::bindings::conversions::{ToJSValConvertible, root_from_handleobject};
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root, RootedReference};
use dom::bindings::proxyhandler::{fill_property_descriptor, get_property_descriptor};
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{Reflectable, Reflector};
use dom::bindings::structuredclone::StructuredCloneData;
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::WindowProxyHandler;
use dom::bindings::utils::get_array_index_from_id;
use dom::document::Document;
use dom::element::Element;
use dom::popstateevent::PopStateEvent;
use dom::urlhelper::UrlHelper;
use dom::window::Window;
use js::JSCLASS_IS_GLOBAL;
use js::glue::{CreateWrapperProxyHandler, ProxyTraps, NewWindowProxy};
use js::glue::{GetProxyPrivate, SetProxyExtra, GetProxyExtra};
use js::jsapi::{Handle, HandleId, HandleObject, HandleValue, JSAutoCompartment};
use js::jsapi::{Heap, MutableHandleValue, ObjectOpResult, RootedObject, RootedValue};
use js::jsapi::{JSContext, JSPROP_READONLY, JSErrNum, JSObject, PropertyDescriptor, JS_DefinePropertyById};
use js::jsapi::{JS_ForwardGetPropertyTo, JS_ForwardSetPropertyTo, JS_GetClass, JSTracer, FreeOp};
use js::jsapi::{JS_GetOwnPropertyDescriptorById, JS_HasPropertyById, MutableHandle};
use js::jsval::{JSVal, UndefinedValue, PrivateValue, NullValue};
use msg::constellation_msg::ConstellationChan;
use msg::constellation_msg::{PipelineId, SubpageId};
use script_thread::Runnable;
use script_traits::ScriptMsg as ConstellationMsg;
use std::cell::Cell;
use task_source::history_traversal::HistoryTraversalTask;
use url::Url;
use util::str::DOMString;

#[dom_struct]
pub struct BrowsingContext {
    reflector_: Reflector,

    /// Pipeline id associated with this context.
    id: PipelineId,

    /// Indicates if reflow is required when reloading.
    needs_reflow: Cell<bool>,

    /// Stores this context's session history
    history: DOMRefCell<Vec<SessionHistoryEntry>>,

    /// The index of the active session history entry
    active_index: Cell<usize>,

    /// Stores the child browsing contexts (ex. iframe browsing context)
    children: DOMRefCell<Vec<JS<BrowsingContext>>>,

    frame_element: Option<JS<Element>>,
}

impl BrowsingContext {
    pub fn new_inherited(frame_element: Option<&Element>, id: PipelineId) -> BrowsingContext {
        BrowsingContext {
            reflector_: Reflector::new(),
            id: id,
            needs_reflow: Cell::new(true),
            history: DOMRefCell::new(vec![]),
            active_index: Cell::new(0),
            children: DOMRefCell::new(vec![]),
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
            let window_proxy = RootedObject::new(cx,
                NewWindowProxy(cx, parent, handler));
            assert!(!window_proxy.ptr.is_null());

            let object = box BrowsingContext::new_inherited(frame_element, id);

            let raw = Box::into_raw(object);
            SetProxyExtra(window_proxy.ptr, 0, &PrivateValue(raw as *const _));

            (*raw).init_reflector(window_proxy.ptr);

            Root::from_ref(&*raw)
        }
    }

    pub fn init(&self, document: &Document) {
        assert!(self.history.borrow().is_empty());
        assert_eq!(self.active_index.get(), 0);
        self.history.borrow_mut().push(SessionHistoryEntry::new(document,
                                                                document.url().clone(),
                                                                document.Title(),
                                                                None));
    }

    pub fn push_history(&self, document: &Document) {
        let mut history = self.history.borrow_mut();
        self.remove_forward_history();
        history.push(SessionHistoryEntry::new(document, document.url().clone(), document.Title(), None));
        self.active_index.set(self.active_index.get() + 1);
        assert_eq!(self.active_index.get(), history.len() - 1);
    }

    pub fn session_history_length(&self) -> usize {
        self.history.borrow().len()
    }

    pub fn active_document(&self) -> Root<Document> {
        Root::from_ref(&self.history.borrow()[self.active_index.get()].document)
    }

    pub fn active_window(&self) -> Root<Window> {
        Root::from_ref(self.active_document().window())
    }

    pub fn set_active_entry(&self, active_index: usize, trigger_event: bool) {
        assert!(active_index < self.history.borrow().len());
        self.active_index.set(active_index);

        // let active_document = self.active_document();
        // let entry = &self.history.borrow()[active_index];
        // active_document.GetLocation().map(|location| location.SetHash(UrlHelper::Hash(&entry.url)));

        if trigger_event {
            let active_window = &*self.active_window();
            let trusted_window = Trusted::new(active_window);
            let task_source = active_window.history_traversal_task_source();
            let runnable = box PopstateNotificationRunnable {
                window: trusted_window,
                state: self.state(),
            };
            let _ = task_source.queue(HistoryTraversalTask::FireNavigationEvent(runnable));
        }
    }

    pub fn state(&self) -> StructuredCloneData {
        let active_window = &*self.active_window();
        let global = GlobalRef::Window(active_window);
        let _ac = JSAutoCompartment::new(global.get_cx(), self.reflector_.get_jsobject().get());

        let state_js = RootedValue::new(global.get_cx(), self.history.borrow()[self.active_index.get()].state());
        StructuredCloneData::write(global.get_cx(), state_js.handle()).unwrap()
    }

    pub fn push_state(&self, state: StructuredCloneData, title: DOMString, url: Option<DOMString>) -> ErrorResult {
        // TODO: update URL after we can set the Url of a doc after creation
        self.remove_forward_history();
        let active_document = self.active_document();

        let active_window = &*self.active_window();
        let global = GlobalRef::Window(active_window);
        let _ac = JSAutoCompartment::new(global.get_cx(), self.reflector_.get_jsobject().get());
        let mut state_js = RootedValue::new(global.get_cx(), NullValue());
        state.read(global, state_js.handle_mut());

        let url = match url {
            Some(url) => {
                match active_document.url().join(&url) {
                    Ok(url) => url,
                    Err(_) => return Err(Error::Security),
                }
            },
            None => active_document.url().clone(),
        };

        if !UrlHelper::SameOrigin(&url, active_document.url()) {
            return Err(Error::Security);
        }

        self.history.borrow_mut().push(SessionHistoryEntry::new(&*active_document,
                                                                url,
                                                                title,
                                                                Some(state_js.handle())));
        let new_index = self.active_index.get() + 1;
        self.set_active_entry(new_index, false);

        let active_window = self.active_window();
        let pipeline_info = active_window.parent_info();
        let ConstellationChan(ref chan) = *active_window.constellation_chan();
        let msg = ConstellationMsg::HistoryStatePushed(pipeline_info, new_index);
        chan.send(msg).unwrap();

        Ok(())
    }

    pub fn replace_state(&self, state: StructuredCloneData, title: DOMString, url: Option<DOMString>) -> ErrorResult {
        // TODO: update URL after we can set the Url of a doc after creation
        let active_index = self.active_index.get();
        assert!(active_index < self.session_history_length());
        let active_document = self.active_document();

        let active_window = &*self.active_window();
        let global = GlobalRef::Window(active_window);
        let _ac = JSAutoCompartment::new(global.get_cx(), self.reflector_.get_jsobject().get());
        let mut state_js = RootedValue::new(global.get_cx(), NullValue());
        state.read(global, state_js.handle_mut());

        let url = match url {
            Some(url) => {
                match active_document.url().join(&url) {
                    Ok(url) => url,
                    Err(_) => return Err(Error::Security),
                }
            },
            None => active_document.url().clone(),
        };

        if !UrlHelper::SameOrigin(&url, active_document.url()) {
            return Err(Error::Security);
        }

        self.history.borrow_mut()[active_index] = SessionHistoryEntry::new(&*active_document,
                                                                           url,
                                                                           title,
                                                                           Some(state_js.handle()));
        self.set_active_entry(active_index, false);

        Ok(())
    }

    // Clear all session history entries after the active index
    pub fn remove_forward_history(&self) {
        let mut history = self.history.borrow_mut();
        history.drain((self.active_index.get() + 1)..);
    }

    pub fn frame_element(&self) -> Option<&Element> {
        self.frame_element.r()
    }

    pub fn window_proxy(&self) -> *mut JSObject {
        let window_proxy = self.reflector_.get_jsobject();
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

    pub fn pipeline(&self) -> PipelineId {
        self.id
    }

    pub fn push_child_context(&self, context: &BrowsingContext) {
        self.children.borrow_mut().push(JS::from_ref(&context));
    }

    pub fn find_child_by_subpage(&self, subpage_id: SubpageId) -> Option<Root<Window>> {
        self.children.borrow().iter().find(|context| {
            let window = context.active_window();
            window.subpage() == Some(subpage_id)
        }).map(|context| context.active_window())
    }

    pub fn clear_session_history(&self) {
        self.active_index.set(0);
        self.history.borrow_mut().clear();
    }

    pub fn handle_popstate(window: Trusted<Window>, state: StructuredCloneData) {
        let window = window.root();
        let global = GlobalRef::Window(&window);
        let target = window.upcast();
        let _ac = JSAutoCompartment::new(global.get_cx(), target.reflector().get_jsobject().get());
        let mut state_js = RootedValue::new(global.get_cx(), UndefinedValue());
        state.read(global, state_js.handle_mut());
        PopStateEvent::dispatch_jsval(target, global, state_js.handle());
    }
}

pub struct PopstateNotificationRunnable {
    window: Trusted<Window>,
    state: StructuredCloneData,
}

impl Runnable for PopstateNotificationRunnable {
    fn handler(self: Box<PopstateNotificationRunnable>) {
        let this = *self;
        BrowsingContext::handle_popstate(this.window, this.state);
    }
}

pub struct ContextIterator {
    stack: Vec<Root<BrowsingContext>>,
}

pub trait IterableContext {
    fn iter(&self) -> ContextIterator;
    fn find(&self, id: PipelineId) -> Option<Root<BrowsingContext>>;
}

impl IterableContext for BrowsingContext {
    fn iter(&self) -> ContextIterator {
        ContextIterator {
            stack: vec!(Root::from_ref(self)),
        }
    }

    fn find(&self, id: PipelineId) -> Option<Root<BrowsingContext>> {
        if self.id == id {
            return Some(Root::from_ref(self));
        }

        self.children.borrow()
                     .iter()
                     .filter_map(|c| c.find(id))
                     .next()
    }
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

// This isn't a DOM struct, just a convenience struct
// without a reflector, so we don't mark this as #[dom_struct]
#[must_root]
#[privatize]
#[derive(JSTraceable, HeapSizeOf)]
pub struct SessionHistoryEntry {
    document: JS<Document>,
    url: Url,
    title: DOMString,
    state: Heap<JSVal>,
}

impl SessionHistoryEntry {
    fn new(document: &Document, url: Url, title: DOMString, state: Option<HandleValue>) -> SessionHistoryEntry {
        let mut jsval: Heap<JSVal> = Default::default();
        let state = match state {
            Some(state) => state,
            None => HandleValue::null()
        };
        jsval.set(state.get());
        SessionHistoryEntry {
            document: JS::from_ref(document),
            url: url,
            title: title,
            state: jsval,
        }
    }

    pub fn state(&self) -> JSVal {
        self.state.get()
    }

    pub fn title(&self) -> DOMString {
        self.title.clone()
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
                                              desc: MutableHandle<PropertyDescriptor>)
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

    let target = RootedObject::new(cx, GetProxyPrivate(*proxy.ptr).to_object());
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
                         receiver: HandleValue,
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

    let target = RootedObject::new(cx, GetProxyPrivate(*proxy.ptr).to_object());
    JS_ForwardSetPropertyTo(cx,
                            target.handle(),
                            id,
                            v,
                            receiver,
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
    trace: Some(trace),
    finalize: Some(finalize),
    objectMoved: None,
    isCallable: None,
    isConstructor: None,
};

#[allow(unsafe_code)]
unsafe extern fn finalize(_fop: *mut FreeOp, obj: *mut JSObject) {
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
