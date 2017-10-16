/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::conversions::{ToJSValConvertible, root_from_handleobject};
use dom::bindings::error::{Error, throw_dom_exception};
use dom::bindings::inheritance::Castable;
use dom::bindings::proxyhandler::{fill_property_descriptor, get_property_descriptor};
use dom::bindings::reflector::{DomObject, Reflector};
use dom::bindings::root::{Dom, DomRoot, RootedReference};
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::{WindowProxyHandler, get_array_index_from_id, AsVoidPtr};
use dom::dissimilaroriginwindow::DissimilarOriginWindow;
use dom::element::Element;
use dom::globalscope::GlobalScope;
use dom::window::Window;
use dom_struct::dom_struct;
use js::JSCLASS_IS_GLOBAL;
use js::glue::{CreateWrapperProxyHandler, ProxyTraps, NewWindowProxy};
use js::glue::{GetProxyPrivate, SetProxyExtra, GetProxyExtra};
use js::jsapi::{Handle, HandleId, HandleObject, HandleValue};
use js::jsapi::{JSAutoCompartment, JSContext, JSErrNum, JSFreeOp, JSObject};
use js::jsapi::{JSPROP_READONLY, JSTracer, JS_DefinePropertyById};
use js::jsapi::{JS_ForwardGetPropertyTo, JS_ForwardSetPropertyTo};
use js::jsapi::{JS_GetOwnPropertyDescriptorById, JS_HasPropertyById, JS_HasOwnPropertyById};
use js::jsapi::{JS_IsExceptionPending, JS_TransplantObject, SetWindowProxy};
use js::jsapi::{MutableHandle, MutableHandleObject, MutableHandleValue};
use js::jsapi::{ObjectOpResult, PropertyDescriptor};
use js::jsval::{UndefinedValue, PrivateValue};
use js::rust::get_object_class;
use msg::constellation_msg::BrowsingContextId;
use msg::constellation_msg::PipelineId;
use msg::constellation_msg::TopLevelBrowsingContextId;
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

    /// The frame id of the top-level ancestor browsing context.
    /// In the case that this is a top-level window, this is our id.
    top_level_browsing_context_id: TopLevelBrowsingContextId,

    /// The pipeline id of the currently active document.
    /// May be None, when the currently active document is in another script thread.
    /// We do not try to keep the pipeline id for documents in other threads,
    /// as this would require the constellation notifying many script threads about
    /// the change, which could be expensive.
    currently_active: Cell<Option<PipelineId>>,

    /// Has the browsing context been discarded?
    discarded: Cell<bool>,

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
                         parent: Option<&WindowProxy>)
                         -> WindowProxy
    {
        WindowProxy {
            reflector: Reflector::new(),
            browsing_context_id: browsing_context_id,
            top_level_browsing_context_id: top_level_browsing_context_id,
            currently_active: Cell::new(currently_active),
            discarded: Cell::new(false),
            frame_element: frame_element.map(Dom::from_ref),
            parent: parent.map(Dom::from_ref),
        }
    }

    #[allow(unsafe_code)]
    pub fn new(window: &Window,
               browsing_context_id: BrowsingContextId,
               top_level_browsing_context_id: TopLevelBrowsingContextId,
               frame_element: Option<&Element>,
               parent: Option<&WindowProxy>)
               -> DomRoot<WindowProxy>
    {
        unsafe {
            let WindowProxyHandler(handler) = window.windowproxy_handler();
            assert!(!handler.is_null());

            let cx = window.get_cx();
            let window_jsobject = window.reflector().get_jsobject();
            assert!(!window_jsobject.get().is_null());
            assert!(((*get_object_class(window_jsobject.get())).flags & JSCLASS_IS_GLOBAL) != 0);
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
                parent
            ));

            // The window proxy owns the browsing context.
            // When we finalize the window proxy, it drops the browsing context it owns.
            SetProxyExtra(js_proxy.get(), 0, &PrivateValue((&*window_proxy).as_void_ptr()));

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
                                 parent: Option<&WindowProxy>)
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
                parent
            ));

            // Create a new dissimilar-origin window.
            let window = DissimilarOriginWindow::new(global_to_clone_from, &*window_proxy);
            let window_jsobject = window.reflector().get_jsobject();
            assert!(!window_jsobject.get().is_null());
            assert!(((*get_object_class(window_jsobject.get())).flags & JSCLASS_IS_GLOBAL) != 0);
            let _ac = JSAutoCompartment::new(cx, window_jsobject.get());

            // Create a new window proxy.
            rooted!(in(cx) let js_proxy = NewWindowProxy(cx, window_jsobject, handler));
            assert!(!js_proxy.is_null());

            // The window proxy owns the browsing context.
            // When we finalize the window proxy, it drops the browsing context it owns.
            SetProxyExtra(js_proxy.get(), 0, &PrivateValue((&*window_proxy).as_void_ptr()));

            // Notify the JS engine about the new window proxy binding.
            SetWindowProxy(cx, window_jsobject, js_proxy.handle());

            // Set the reflector.
            debug!("Initializing reflector of {:p} to {:p}.", window_proxy, js_proxy.get());
            window_proxy.reflector.set_jsobject(js_proxy.get());
            DomRoot::from_ref(&*Box::into_raw(window_proxy))
        }
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
            assert!(((*get_object_class(window_jsobject.get())).flags & JSCLASS_IS_GLOBAL) != 0);
            let _ac = JSAutoCompartment::new(cx, window_jsobject.get());

            // The old window proxy no longer owns this browsing context.
            SetProxyExtra(old_js_proxy.get(), 0, &PrivateValue(ptr::null_mut()));

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
            SetProxyExtra(new_js_proxy.get(), 0, &PrivateValue(self.as_void_ptr()));

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
}

#[allow(unsafe_code)]
unsafe fn GetSubframeWindow(cx: *mut JSContext,
                            proxy: HandleObject,
                            id: HandleId)
                            -> Option<DomRoot<Window>> {
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
                                 proxy: HandleObject,
                                 id: HandleId,
                                 bp: *mut bool)
                                 -> bool
{
    rooted!(in(cx) let target = GetProxyPrivate(*proxy.ptr).to_object());
    let mut found = false;
    JS_HasOwnPropertyById(cx, target.handle(), id, &mut found);
    if found {
        *bp = true;
        true
    } else {
        throw_security_error(cx)
    }
}

#[allow(unsafe_code)]
unsafe extern "C" fn get_xorigin(cx: *mut JSContext,
                                 proxy: HandleObject,
                                 receiver: HandleValue,
                                 id: HandleId,
                                 vp: MutableHandleValue)
                                 -> bool
{
    let mut found = false;
    has_xorigin(cx, proxy, id, &mut found);
    found && get(cx, proxy, receiver, id, vp)
}

#[allow(unsafe_code)]
unsafe extern "C" fn set_xorigin(cx: *mut JSContext,
                                 _: HandleObject,
                                 _: HandleId,
                                 _: HandleValue,
                                 _: HandleValue,
                                 _: *mut ObjectOpResult)
                                 -> bool
{
    throw_security_error(cx)
}

#[allow(unsafe_code)]
unsafe extern "C" fn delete_xorigin(cx: *mut JSContext,
                                    _: HandleObject,
                                    _: HandleId,
                                    _: *mut ObjectOpResult)
                                    -> bool
{
    throw_security_error(cx)
}

#[allow(unsafe_code)]
unsafe extern "C" fn getOwnPropertyDescriptor_xorigin(cx: *mut JSContext,
                                                      proxy: HandleObject,
                                                      id: HandleId,
                                                      desc: MutableHandle<PropertyDescriptor>)
                                                      -> bool
{
    let mut found = false;
    has_xorigin(cx, proxy, id, &mut found);
    found && getOwnPropertyDescriptor(cx, proxy, id, desc)
}

#[allow(unsafe_code)]
unsafe extern "C" fn defineProperty_xorigin(cx: *mut JSContext,
                                            _: HandleObject,
                                            _: HandleId,
                                            _: Handle<PropertyDescriptor>,
                                            _: *mut ObjectOpResult)
                                            -> bool
{
    throw_security_error(cx)
}

#[allow(unsafe_code)]
unsafe extern "C" fn preventExtensions_xorigin(cx: *mut JSContext,
                                               _: HandleObject,
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
    let this = GetProxyExtra(obj, 0).to_private() as *mut WindowProxy;
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
    let this = GetProxyExtra(obj, 0).to_private() as *const WindowProxy;
    if this.is_null() {
        // GC during obj creation or after transplanting.
        return;
    }
    (*this).trace(trc);
}
