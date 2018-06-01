/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DomRefCell;
use dom::bindings::conversions::{ToJSValConvertible, root_from_handleobject};
use dom::bindings::error::{Error, throw_dom_exception};
use dom::bindings::inheritance::Castable;
use dom::bindings::proxyhandler::{fill_property_descriptor, get_property_descriptor};
use dom::bindings::reflector::{DomObject, Reflector};
use dom::bindings::root::{Dom, DomRoot, RootedReference};
use dom::bindings::str::DOMString;
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::{WindowProxyHandler, get_array_index_from_id, AsVoidPtr};
use dom::dissimilaroriginwindow::DissimilarOriginWindow;
use dom::document::Document;
use dom::element::Element;
use dom::globalscope::GlobalScope;
use dom::window::Window;
use dom_struct::dom_struct;
use embedder_traits::EmbedderMsg;
use ipc_channel::ipc;
use js::JSCLASS_IS_GLOBAL;
use js::glue::{CreateWrapperProxyHandler, ProxyTraps};
use js::glue::{GetProxyPrivate, SetProxyReservedSlot, GetProxyReservedSlot};
use js::jsapi::{JSAutoCompartment, JSContext, JSErrNum, JSFreeOp, JSObject};
use js::jsapi::{JSPROP_ENUMERATE, JSPROP_READONLY, JSTracer, JS_DefinePropertyById};
use js::jsapi::{JS_ForwardGetPropertyTo, JS_ForwardSetPropertyTo};
use js::jsapi::{JS_HasPropertyById, JS_HasOwnPropertyById};
use js::jsapi::{JS_IsExceptionPending, JS_GetOwnPropertyDescriptorById};
use js::jsapi::{ObjectOpResult, PropertyDescriptor};
use js::jsapi::Handle as RawHandle;
use js::jsapi::HandleId as RawHandleId;
use js::jsapi::HandleObject as RawHandleObject;
use js::jsapi::HandleValue as RawHandleValue;
use js::jsapi::MutableHandle as RawMutableHandle;
use js::jsapi::MutableHandleObject as RawMutableHandleObject;
use js::jsapi::MutableHandleValue as RawMutableHandleValue;
use js::jsval::{JSVal, NullValue, UndefinedValue, PrivateValue};
use js::rust::{Handle, MutableHandle};
use js::rust::get_object_class;
use js::rust::wrappers::{NewWindowProxy, SetWindowProxy, JS_TransplantObject};
use msg::constellation_msg::BrowsingContextId;
use msg::constellation_msg::PipelineId;
use msg::constellation_msg::TopLevelBrowsingContextId;
use script_thread::ScriptThread;
use script_traits::{AuxiliaryBrowsingContextLoadInfo, LoadData, NewLayoutInfo, ScriptMsg};
use servo_config::prefs::PREFS;
use servo_url::ServoUrl;
use std::cell::Cell;
use std::ptr;

#[dom_struct]
// NOTE: the browsing context for a window is managed in two places:
// here, in script, but also in the constellation. The constellation
// manages the session history, which in script is accessed through
// History objects, messaging the constellation.
pub struct WindowProxy {
    /// The JS WindowProxy object.
    /// Unlike other reflectors, we mutate this field because
    /// we have to brain-transplant the reflector when the WindowProxy
    /// changes Window.
    reflector: Reflector,

    /// The id of the browsing context.
    /// In the case that this is a nested browsing context, this is the id
    /// of the container.
    browsing_context_id: BrowsingContextId,

    // https://html.spec.whatwg.org/multipage/#opener-browsing-context
    opener: Option<BrowsingContextId>,

    /// The frame id of the top-level ancestor browsing context.
    /// In the case that this is a top-level window, this is our id.
    top_level_browsing_context_id: TopLevelBrowsingContextId,

    /// The name of the browsing context
    name: DomRefCell<DOMString>,
    /// The pipeline id of the currently active document.
    /// May be None, when the currently active document is in another script thread.
    /// We do not try to keep the pipeline id for documents in other threads,
    /// as this would require the constellation notifying many script threads about
    /// the change, which could be expensive.
    currently_active: Cell<Option<PipelineId>>,

    /// Has the browsing context been discarded?
    discarded: Cell<bool>,

    /// Has the browsing context been disowned?
    disowned: Cell<bool>,

    /// The containing iframe element, if this is a same-origin iframe
    frame_element: Option<Dom<Element>>,

    /// The parent browsing context's window proxy, if this is a nested browsing context
    parent: Option<Dom<WindowProxy>>,
}

impl WindowProxy {
    pub fn new_inherited(browsing_context_id: BrowsingContextId,
                         top_level_browsing_context_id: TopLevelBrowsingContextId,
                         currently_active: Option<PipelineId>,
                         frame_element: Option<&Element>,
                         parent: Option<&WindowProxy>,
                         opener: Option<BrowsingContextId>)
                         -> WindowProxy
    {
        let name = frame_element.map_or(DOMString::new(), |e| e.get_string_attribute(&local_name!("name")));
        WindowProxy {
            reflector: Reflector::new(),
            browsing_context_id: browsing_context_id,
            top_level_browsing_context_id: top_level_browsing_context_id,
            name: DomRefCell::new(name),
            currently_active: Cell::new(currently_active),
            discarded: Cell::new(false),
            disowned: Cell::new(false),
            frame_element: frame_element.map(Dom::from_ref),
            parent: parent.map(Dom::from_ref),
            opener,
        }
    }

    #[allow(unsafe_code)]
    pub fn new(window: &Window,
               browsing_context_id: BrowsingContextId,
               top_level_browsing_context_id: TopLevelBrowsingContextId,
               frame_element: Option<&Element>,
               parent: Option<&WindowProxy>,
               opener: Option<BrowsingContextId>)
               -> DomRoot<WindowProxy>
    {
        unsafe {
            let WindowProxyHandler(handler) = window.windowproxy_handler();
            assert!(!handler.is_null());

            let cx = window.get_cx();
            let window_jsobject = window.reflector().get_jsobject();
            assert!(!window_jsobject.get().is_null());
            assert_ne!(((*get_object_class(window_jsobject.get())).flags & JSCLASS_IS_GLOBAL), 0);
            let _ac = JSAutoCompartment::new(cx, window_jsobject.get());

            // Create a new window proxy.
            rooted!(in(cx) let js_proxy = NewWindowProxy(cx, window_jsobject, handler));
            assert!(!js_proxy.is_null());

            // Create a new browsing context.
            let current = Some(window.global().pipeline_id());
            let mut window_proxy = Box::new(WindowProxy::new_inherited(
                browsing_context_id,
                top_level_browsing_context_id,
                current,
                frame_element,
                parent,
                opener,
            ));

            // The window proxy owns the browsing context.
            // When we finalize the window proxy, it drops the browsing context it owns.
            SetProxyReservedSlot(js_proxy.get(), 0, &PrivateValue((&*window_proxy).as_void_ptr()));

            // Notify the JS engine about the new window proxy binding.
            SetWindowProxy(cx, window_jsobject, js_proxy.handle());

            // Set the reflector.
            debug!("Initializing reflector of {:p} to {:p}.", window_proxy, js_proxy.get());
            window_proxy.reflector.set_jsobject(js_proxy.get());
            DomRoot::from_ref(&*Box::into_raw(window_proxy))
        }
    }

    #[allow(unsafe_code)]
    pub fn new_dissimilar_origin(global_to_clone_from: &GlobalScope,
                                 browsing_context_id: BrowsingContextId,
                                 top_level_browsing_context_id: TopLevelBrowsingContextId,
                                 parent: Option<&WindowProxy>,
                                 opener: Option<BrowsingContextId>)
                                 -> DomRoot<WindowProxy>
    {
        unsafe {
            let handler = CreateWrapperProxyHandler(&XORIGIN_PROXY_HANDLER);
            assert!(!handler.is_null());

            let cx = global_to_clone_from.get_cx();

            // Create a new browsing context.
            let mut window_proxy = Box::new(WindowProxy::new_inherited(
                browsing_context_id,
                top_level_browsing_context_id,
                None,
                None,
                parent,
                opener
            ));

            // Create a new dissimilar-origin window.
            let window = DissimilarOriginWindow::new(global_to_clone_from, &*window_proxy);
            let window_jsobject = window.reflector().get_jsobject();
            assert!(!window_jsobject.get().is_null());
            assert_ne!(((*get_object_class(window_jsobject.get())).flags & JSCLASS_IS_GLOBAL), 0);
            let _ac = JSAutoCompartment::new(cx, window_jsobject.get());

            // Create a new window proxy.
            rooted!(in(cx) let js_proxy = NewWindowProxy(cx, window_jsobject, handler));
            assert!(!js_proxy.is_null());

            // The window proxy owns the browsing context.
            // When we finalize the window proxy, it drops the browsing context it owns.
            SetProxyReservedSlot(js_proxy.get(), 0, &PrivateValue((&*window_proxy).as_void_ptr()));

            // Notify the JS engine about the new window proxy binding.
            SetWindowProxy(cx, window_jsobject, js_proxy.handle());

            // Set the reflector.
            debug!("Initializing reflector of {:p} to {:p}.", window_proxy, js_proxy.get());
            window_proxy.reflector.set_jsobject(js_proxy.get());
            DomRoot::from_ref(&*Box::into_raw(window_proxy))
        }
    }

    // https://html.spec.whatwg.org/multipage/#auxiliary-browsing-context
    fn create_auxiliary_browsing_context(&self, name: DOMString, noopener: bool) -> Option<DomRoot<WindowProxy>> {
        let (chan, port) = ipc::channel().unwrap();
        let window = self.currently_active.get()
                    .and_then(|id| ScriptThread::find_document(id))
                    .and_then(|doc| Some(DomRoot::from_ref(doc.window())))
                    .unwrap();
        let msg = EmbedderMsg::AllowOpeningBrowser(chan);
        window.send_to_embedder(msg);
        if port.recv().unwrap() {
            let new_top_level_browsing_context_id = TopLevelBrowsingContextId::new();
            let new_browsing_context_id = BrowsingContextId::from(new_top_level_browsing_context_id);
            let new_pipeline_id = PipelineId::new();
            let load_info = AuxiliaryBrowsingContextLoadInfo {
                opener_pipeline_id: self.currently_active.get().unwrap(),
                new_browsing_context_id: new_browsing_context_id,
                new_top_level_browsing_context_id: new_top_level_browsing_context_id,
                new_pipeline_id: new_pipeline_id,
            };
            let document = self.currently_active.get()
                .and_then(|id| ScriptThread::find_document(id))
                .unwrap();
            let blank_url = ServoUrl::parse("about:blank").ok().unwrap();
            let load_data = LoadData::new(blank_url,
                                          None,
                                          document.get_referrer_policy(),
                                          Some(document.url().clone()));
            let (pipeline_sender, pipeline_receiver) = ipc::channel().unwrap();
            let new_layout_info = NewLayoutInfo {
                parent_info: None,
                new_pipeline_id: new_pipeline_id,
                browsing_context_id: new_browsing_context_id,
                top_level_browsing_context_id: new_top_level_browsing_context_id,
                opener: Some(self.browsing_context_id),
                load_data: load_data,
                pipeline_port: pipeline_receiver,
                content_process_shutdown_chan: None,
                window_size: None,
                layout_threads: PREFS.get("layout.threads").as_u64().expect("count") as usize,
            };
            let constellation_msg = ScriptMsg::ScriptNewAuxiliary(load_info, pipeline_sender);
            window.send_to_constellation(constellation_msg);
            ScriptThread::process_attach_layout(new_layout_info, document.origin().clone());
            let msg = EmbedderMsg::BrowserCreated(new_top_level_browsing_context_id);
            window.send_to_embedder(msg);
            // TODO: if noopener is false, copy the sessionStorage storage area of the creator origin.
            // See step 14 of https://html.spec.whatwg.org/multipage/#creating-a-new-browsing-context
            let auxiliary = ScriptThread::find_document(new_pipeline_id).and_then(|doc| doc.browsing_context());
            if let Some(proxy) = auxiliary {
                if name.to_lowercase() != "_blank" {
                    proxy.set_name(name);
                }
                if noopener {
                    proxy.disown();
                }
                return Some(proxy)
            }
        }
        None
    }

    // https://html.spec.whatwg.org/multipage/#disowned-its-opener
    pub fn disown(&self) {
        self.disowned.set(true);
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-opener
    pub unsafe fn opener(&self, cx: *mut JSContext) -> JSVal {
        if self.disowned.get() {
            return NullValue()
        }
        let opener_id = match self.opener {
            Some(opener_browsing_context_id) => opener_browsing_context_id,
            None => return NullValue()
        };
        let opener_proxy = match ScriptThread::find_window_proxy(opener_id) {
            Some(window_proxy) => window_proxy,
            None => {
                let sender_pipeline_id = self.currently_active().unwrap();
                match ScriptThread::get_top_level_for_browsing_context(sender_pipeline_id, opener_id) {
                    Some(opener_top_id) => {
                        let global_to_clone_from = GlobalScope::from_context(cx);
                        WindowProxy::new_dissimilar_origin(
                            &*global_to_clone_from,
                            opener_id,
                            opener_top_id,
                            None,
                            None
                        )
                    },
                    None => return NullValue()
                }
            }
        };
        if opener_proxy.is_browsing_context_discarded() {
            return NullValue()
        }
        rooted!(in(cx) let mut val = UndefinedValue());
        opener_proxy.to_jsval(cx, val.handle_mut());
        return val.get()
    }

    // https://html.spec.whatwg.org/multipage/#window-open-steps
    pub fn open(&self,
                url: DOMString,
                target: DOMString,
                features: DOMString)
                -> Option<DomRoot<WindowProxy>> {
        // Step 3.
        let non_empty_target = match target.as_ref() {
            "" => DOMString::from("_blank"),
            _ => target
        };
        // TODO Step 4, properly tokenize features.
        // Step 5
        let noopener = features.contains("noopener");
        // Step 6, 7
        let (chosen, new) = match self.choose_browsing_context(non_empty_target, noopener) {
            (Some(chosen), new) => (chosen, new),
            (None, _) => return None
        };
        // TODO Step 8, set up browsing context features.
        let target_document = match chosen.document() {
            Some(target_document) => target_document,
            None => return None
        };
        let target_window = target_document.window();
        // Step 9, and 10.2, will have happened elsewhere,
        // since we've created a new browsing context and loaded it with about:blank.
        if !url.is_empty() {
            let existing_document = self.currently_active.get()
                        .and_then(|id| ScriptThread::find_document(id)).unwrap();
            // Step 10.1
            let url = match existing_document.url().join(&url) {
                Ok(url) => url,
                Err(_) => return None, // TODO: throw a  "SyntaxError" DOMException.
            };
            // Step 10.3
            target_window.load_url(url, new, false, target_document.get_referrer_policy());
        }
        if noopener {
            // Step 11 (Dis-owning has been done in create_auxiliary_browsing_context).
            return None
        }
        // Step 12.
        return target_document.browsing_context()
    }

    // https://html.spec.whatwg.org/multipage/#the-rules-for-choosing-a-browsing-context-given-a-browsing-context-name
    pub fn choose_browsing_context(&self, name: DOMString, noopener: bool) -> (Option<DomRoot<WindowProxy>>, bool) {
        match name.to_lowercase().as_ref() {
            "" | "_self" => {
                // Step 3.
                (Some(DomRoot::from_ref(self)), false)
            },
            "_parent" => {
                // Step 4
                if let Some(parent) = self.parent() {
                    return (Some(DomRoot::from_ref(parent)), false)
                }
                (None, false)

            },
            "_top" => {
                // Step 5
                (Some(DomRoot::from_ref(self.top())), false)
            },
            "_blank" => {
                (self.create_auxiliary_browsing_context(name, noopener), true)
            },
            _ => {
                // Step 6.
                // TODO: expand the search to all 'familiar' bc,
                // including auxiliaries familiar by way of their opener.
                // See https://html.spec.whatwg.org/multipage/#familiar-with
                match ScriptThread::find_window_proxy_by_name(&name) {
                    Some(proxy) => (Some(proxy), false),
                    None => (self.create_auxiliary_browsing_context(name, noopener), true)
                }
            }
        }
    }

    pub fn is_auxiliary(&self) -> bool {
        self.opener.is_some()
    }

    pub fn discard_browsing_context(&self) {
        self.discarded.set(true);
    }

    pub fn is_browsing_context_discarded(&self) -> bool {
        self.discarded.get()
    }

    pub fn browsing_context_id(&self) -> BrowsingContextId {
        self.browsing_context_id
    }

    pub fn top_level_browsing_context_id(&self) -> TopLevelBrowsingContextId {
        self.top_level_browsing_context_id
    }

    pub fn frame_element(&self) -> Option<&Element> {
        self.frame_element.r()
    }

    pub fn document(&self) -> Option<DomRoot<Document>> {
        self.currently_active.get()
            .and_then(|id| ScriptThread::find_document(id))
    }

    pub fn parent(&self) -> Option<&WindowProxy> {
        self.parent.r()
    }

    pub fn top(&self) -> &WindowProxy {
        let mut result = self;
        while let Some(parent) = result.parent() {
            result = parent;
        }
        result
    }

    #[allow(unsafe_code)]
    /// Change the Window that this WindowProxy resolves to.
    // TODO: support setting the window proxy to a dummy value,
    // to handle the case when the active document is in another script thread.
    fn set_window(&self, window: &GlobalScope, traps: &ProxyTraps) {
        unsafe {
            debug!("Setting window of {:p}.", self);
            let handler = CreateWrapperProxyHandler(traps);
            assert!(!handler.is_null());

            let cx = window.get_cx();
            let window_jsobject = window.reflector().get_jsobject();
            let old_js_proxy = self.reflector.get_jsobject();
            assert!(!window_jsobject.get().is_null());
            assert_ne!(((*get_object_class(window_jsobject.get())).flags & JSCLASS_IS_GLOBAL), 0);
            let _ac = JSAutoCompartment::new(cx, window_jsobject.get());

            // The old window proxy no longer owns this browsing context.
            SetProxyReservedSlot(old_js_proxy.get(), 0, &PrivateValue(ptr::null_mut()));

            // Brain transpant the window proxy.
            // We need to do this, because the Window and WindowProxy
            // objects need to be in the same compartment.
            // JS_TransplantObject does this by copying the contents
            // of the old window proxy to the new window proxy, then
            // making the old window proxy a cross-compartment wrapper
            // pointing to the new window proxy.
            rooted!(in(cx) let new_js_proxy = NewWindowProxy(cx, window_jsobject, handler));
            debug!("Transplanting proxy from {:p} to {:p}.", old_js_proxy.get(), new_js_proxy.get());
            rooted!(in(cx) let new_js_proxy = JS_TransplantObject(cx, old_js_proxy, new_js_proxy.handle()));
            debug!("Transplanted proxy is {:p}.", new_js_proxy.get());

            // Transfer ownership of this browsing context from the old window proxy to the new one.
            SetProxyReservedSlot(new_js_proxy.get(), 0, &PrivateValue(self.as_void_ptr()));

            // Notify the JS engine about the new window proxy binding.
            SetWindowProxy(cx, window_jsobject, new_js_proxy.handle());

            // Update the reflector.
            debug!("Setting reflector of {:p} to {:p}.", self, new_js_proxy.get());
            self.reflector.rootable().set(new_js_proxy.get());
        }
    }

    pub fn set_currently_active(&self, window: &Window) {
        let globalscope = window.upcast();
        self.set_window(&*globalscope, &PROXY_HANDLER);
        self.currently_active.set(Some(globalscope.pipeline_id()));
    }

    pub fn unset_currently_active(&self) {
        let globalscope = self.global();
        let window = DissimilarOriginWindow::new(&*globalscope, self);
        self.set_window(&*window.upcast(), &XORIGIN_PROXY_HANDLER);
        self.currently_active.set(None);
    }

    pub fn currently_active(&self) -> Option<PipelineId> {
        self.currently_active.get()
    }

    pub fn get_name(&self) -> DOMString {
        self.name.borrow().clone()
    }

    pub fn set_name(&self, name: DOMString) {
        *self.name.borrow_mut() = name;
    }
}

// This is only called from extern functions,
// there's no use using the lifetimed handles here.
// https://html.spec.whatwg.org/multipage/#accessing-other-browsing-contexts
#[allow(unsafe_code)]
unsafe fn GetSubframeWindowProxy(
    cx: *mut JSContext,
    proxy: RawHandleObject,
    id: RawHandleId
) -> Option<(DomRoot<WindowProxy>, u32)> {
    let index = get_array_index_from_id(cx, Handle::from_raw(id));
    if let Some(index) = index {
        let ref mut slot = UndefinedValue();
        GetProxyPrivate(*proxy, slot);
        rooted!(in(cx) let target = slot.to_object());
        if let Ok(win) = root_from_handleobject::<Window>(target.handle()) {
            let browsing_context_id = win.window_proxy().browsing_context_id();
            let (result_sender, result_receiver) = ipc::channel().unwrap();

            let _ = win.upcast::<GlobalScope>().script_to_constellation_chan().send(
                ScriptMsg::GetChildBrowsingContextId(
                    browsing_context_id,
                    index as usize,
                    result_sender
                )
            );
            return result_receiver.recv().ok()
                .and_then(|maybe_bcid| maybe_bcid)
                .and_then(ScriptThread::find_window_proxy)
                .map(|proxy| (proxy, (JSPROP_ENUMERATE | JSPROP_READONLY) as u32));
        } else if let Ok(win) = root_from_handleobject::<DissimilarOriginWindow>(target.handle()) {
            let browsing_context_id = win.window_proxy().browsing_context_id();
            let (result_sender, result_receiver) = ipc::channel().unwrap();

            let _ = win.global().script_to_constellation_chan().send(ScriptMsg::GetChildBrowsingContextId(
                browsing_context_id,
                index as usize,
                result_sender
            ));
            return result_receiver.recv().ok()
                .and_then(|maybe_bcid| maybe_bcid)
                .and_then(ScriptThread::find_window_proxy)
                .map(|proxy| (proxy, JSPROP_READONLY as u32));
        }
    }

    None
}

#[allow(unsafe_code)]
unsafe extern "C" fn getOwnPropertyDescriptor(cx: *mut JSContext,
                                              proxy: RawHandleObject,
                                              id: RawHandleId,
                                              mut desc: RawMutableHandle<PropertyDescriptor>)
                                              -> bool {
    let window = GetSubframeWindowProxy(cx, proxy, id);
    if let Some((window, attrs)) = window {
        rooted!(in(cx) let mut val = UndefinedValue());
        window.to_jsval(cx, val.handle_mut());
        desc.value = val.get();
        fill_property_descriptor(MutableHandle::from_raw(desc), proxy.get(), attrs);
        return true;
    }

    let ref mut slot = UndefinedValue();
    GetProxyPrivate(proxy.get(), slot);
    rooted!(in(cx) let target = slot.to_object());
    if !JS_GetOwnPropertyDescriptorById(cx, target.handle().into(), id, desc) {
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
                                    proxy: RawHandleObject,
                                    id: RawHandleId,
                                    desc: RawHandle<PropertyDescriptor>,
                                    res: *mut ObjectOpResult)
                                    -> bool {
    if get_array_index_from_id(cx, Handle::from_raw(id)).is_some() {
        // Spec says to Reject whether this is a supported index or not,
        // since we have no indexed setter or indexed creator.  That means
        // throwing in strict mode (FIXME: Bug 828137), doing nothing in
        // non-strict mode.
        (*res).code_ = JSErrNum::JSMSG_CANT_DEFINE_WINDOW_ELEMENT as ::libc::uintptr_t;
        return true;
    }

    let ref mut slot = UndefinedValue();
    GetProxyPrivate(*proxy.ptr, slot);
    rooted!(in(cx) let target = slot.to_object());
    JS_DefinePropertyById(cx, target.handle().into(), id, desc, res)
}

#[allow(unsafe_code)]
unsafe extern "C" fn has(cx: *mut JSContext,
                         proxy: RawHandleObject,
                         id: RawHandleId,
                         bp: *mut bool)
                         -> bool {
    let window = GetSubframeWindowProxy(cx, proxy, id);
    if window.is_some() {
        *bp = true;
        return true;
    }

    let ref mut slot = UndefinedValue();
    GetProxyPrivate(*proxy.ptr, slot);
    rooted!(in(cx) let target = slot.to_object());
    let mut found = false;
    if !JS_HasPropertyById(cx, target.handle().into(), id, &mut found) {
        return false;
    }

    *bp = found;
    true
}

#[allow(unsafe_code)]
unsafe extern "C" fn get(cx: *mut JSContext,
                         proxy: RawHandleObject,
                         receiver: RawHandleValue,
                         id: RawHandleId,
                         vp: RawMutableHandleValue)
                         -> bool {
    let window = GetSubframeWindowProxy(cx, proxy, id);
    if let Some((window, _attrs)) = window {
        window.to_jsval(cx, MutableHandle::from_raw(vp));
        return true;
    }

    let ref mut slot = UndefinedValue();
    GetProxyPrivate(*proxy.ptr, slot);
    rooted!(in(cx) let target = slot.to_object());
    JS_ForwardGetPropertyTo(cx, target.handle().into(), id, receiver, vp)
}

#[allow(unsafe_code)]
unsafe extern "C" fn set(cx: *mut JSContext,
                         proxy: RawHandleObject,
                         id: RawHandleId,
                         v: RawHandleValue,
                         receiver: RawHandleValue,
                         res: *mut ObjectOpResult)
                         -> bool {
    if get_array_index_from_id(cx, Handle::from_raw(id)).is_some() {
        // Reject (which means throw if and only if strict) the set.
        (*res).code_ = JSErrNum::JSMSG_READ_ONLY as ::libc::uintptr_t;
        return true;
    }

    let ref mut slot = UndefinedValue();
    GetProxyPrivate(*proxy.ptr, slot);
    rooted!(in(cx) let target = slot.to_object());
    JS_ForwardSetPropertyTo(cx,
                            target.handle().into(),
                            id,
                            v,
                            receiver,
                            res)
}

#[allow(unsafe_code)]
unsafe extern "C" fn get_prototype_if_ordinary(_: *mut JSContext,
                                               _: RawHandleObject,
                                               is_ordinary: *mut bool,
                                               _: RawMutableHandleObject)
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
pub fn new_window_proxy_handler() -> WindowProxyHandler {
    unsafe {
        WindowProxyHandler(CreateWrapperProxyHandler(&PROXY_HANDLER))
    }
}

// The proxy traps for cross-origin windows.
// These traps often throw security errors, and only pass on calls to methods
// defined in the DissimilarOriginWindow IDL.

#[allow(unsafe_code)]
unsafe fn throw_security_error(cx: *mut JSContext) -> bool {
    if !JS_IsExceptionPending(cx) {
        let global = GlobalScope::from_context(cx);
        throw_dom_exception(cx, &*global, Error::Security);
    }
    false
}

#[allow(unsafe_code)]
unsafe extern "C" fn has_xorigin(cx: *mut JSContext,
                                 proxy: RawHandleObject,
                                 id: RawHandleId,
                                 bp: *mut bool)
                                 -> bool
{
    let ref mut slot = UndefinedValue();
    GetProxyPrivate(*proxy.ptr, slot);
    rooted!(in(cx) let target = slot.to_object());
    let mut found = false;
    JS_HasOwnPropertyById(cx, target.handle().into(), id, &mut found);
    if found {
        *bp = true;
        true
    } else {
        throw_security_error(cx)
    }
}

#[allow(unsafe_code)]
unsafe extern "C" fn get_xorigin(cx: *mut JSContext,
                                 proxy: RawHandleObject,
                                 receiver: RawHandleValue,
                                 id: RawHandleId,
                                 vp: RawMutableHandleValue)
                                 -> bool
{
    let mut found = false;
    has_xorigin(cx, proxy, id, &mut found);
    found && get(cx, proxy, receiver, id, vp)
}

#[allow(unsafe_code)]
unsafe extern "C" fn set_xorigin(cx: *mut JSContext,
                                 _: RawHandleObject,
                                 _: RawHandleId,
                                 _: RawHandleValue,
                                 _: RawHandleValue,
                                 _: *mut ObjectOpResult)
                                 -> bool
{
    throw_security_error(cx)
}

#[allow(unsafe_code)]
unsafe extern "C" fn delete_xorigin(cx: *mut JSContext,
                                    _: RawHandleObject,
                                    _: RawHandleId,
                                    _: *mut ObjectOpResult)
                                    -> bool
{
    throw_security_error(cx)
}

#[allow(unsafe_code)]
unsafe extern "C" fn getOwnPropertyDescriptor_xorigin(cx: *mut JSContext,
                                                      proxy: RawHandleObject,
                                                      id: RawHandleId,
                                                      desc: RawMutableHandle<PropertyDescriptor>)
                                                      -> bool
{
    let mut found = false;
    has_xorigin(cx, proxy, id, &mut found);
    found && getOwnPropertyDescriptor(cx, proxy, id, desc)
}

#[allow(unsafe_code)]
unsafe extern "C" fn defineProperty_xorigin(cx: *mut JSContext,
                                            _: RawHandleObject,
                                            _: RawHandleId,
                                            _: RawHandle<PropertyDescriptor>,
                                            _: *mut ObjectOpResult)
                                            -> bool
{
    throw_security_error(cx)
}

#[allow(unsafe_code)]
unsafe extern "C" fn preventExtensions_xorigin(cx: *mut JSContext,
                                               _: RawHandleObject,
                                               _: *mut ObjectOpResult)
                                               -> bool
{
    throw_security_error(cx)
}

static XORIGIN_PROXY_HANDLER: ProxyTraps = ProxyTraps {
    enter: None,
    getOwnPropertyDescriptor: Some(getOwnPropertyDescriptor_xorigin),
    defineProperty: Some(defineProperty_xorigin),
    ownPropertyKeys: None,
    delete_: Some(delete_xorigin),
    enumerate: None,
    getPrototypeIfOrdinary: None,
    preventExtensions: Some(preventExtensions_xorigin),
    isExtensible: None,
    has: Some(has_xorigin),
    get: Some(get_xorigin),
    set: Some(set_xorigin),
    call: None,
    construct: None,
    getPropertyDescriptor: Some(getOwnPropertyDescriptor_xorigin),
    hasOwn: Some(has_xorigin),
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

// How WindowProxy objects are garbage collected.

#[allow(unsafe_code)]
unsafe extern fn finalize(_fop: *mut JSFreeOp, obj: *mut JSObject) {
    let ref mut slot = UndefinedValue();
    GetProxyReservedSlot(obj, 0, slot);
    let this = slot.to_private() as *mut WindowProxy;
    if this.is_null() {
        // GC during obj creation or after transplanting.
        return;
    }
    let jsobject = (*this).reflector.get_jsobject().get();
    debug!("WindowProxy finalize: {:p}, with reflector {:p} from {:p}.", this, jsobject, obj);
    let _ = Box::from_raw(this);
}

#[allow(unsafe_code)]
unsafe extern fn trace(trc: *mut JSTracer, obj: *mut JSObject) {
    let ref mut slot = UndefinedValue();
    GetProxyReservedSlot(obj, 0, slot);
    let this = slot.to_private() as *const WindowProxy;
    if this.is_null() {
        // GC during obj creation or after transplanting.
        return;
    }
    (*this).trace(trc);
}
