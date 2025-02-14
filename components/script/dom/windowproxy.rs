/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ptr;

use base::id::{BrowsingContextId, PipelineId, TopLevelBrowsingContextId};
use dom_struct::dom_struct;
use embedder_traits::EmbedderMsg;
use html5ever::local_name;
use indexmap::map::IndexMap;
use ipc_channel::ipc;
use js::glue::{
    CreateWrapperProxyHandler, DeleteWrapperProxyHandler, GetProxyPrivate, GetProxyReservedSlot,
    ProxyTraps, SetProxyReservedSlot,
};
use js::jsapi::{
    GCContext, Handle as RawHandle, HandleId as RawHandleId, HandleObject as RawHandleObject,
    HandleValue as RawHandleValue, JSAutoRealm, JSContext, JSErrNum, JSObject, JSTracer,
    JS_DefinePropertyById, JS_ForwardGetPropertyTo, JS_ForwardSetPropertyTo,
    JS_GetOwnPropertyDescriptorById, JS_HasOwnPropertyById, JS_HasPropertyById,
    JS_IsExceptionPending, MutableHandle as RawMutableHandle,
    MutableHandleObject as RawMutableHandleObject, MutableHandleValue as RawMutableHandleValue,
    ObjectOpResult, PropertyDescriptor, JSPROP_ENUMERATE, JSPROP_READONLY,
};
use js::jsval::{NullValue, PrivateValue, UndefinedValue};
use js::rust::wrappers::{JS_TransplantObject, NewWindowProxy, SetWindowProxy};
use js::rust::{get_object_class, Handle, MutableHandle, MutableHandleValue};
use js::JSCLASS_IS_GLOBAL;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use net_traits::request::Referrer;
use script_traits::{
    AuxiliaryBrowsingContextLoadInfo, LoadData, LoadOrigin, NavigationHistoryBehavior,
    NewLayoutInfo, ScriptMsg,
};
use serde::{Deserialize, Serialize};
use servo_url::{ImmutableOrigin, ServoUrl};
use style::attr::parse_integer;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::conversions::{root_from_handleobject, ToJSValConvertible};
use crate::dom::bindings::error::{throw_dom_exception, Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::proxyhandler::set_property_descriptor;
use crate::dom::bindings::reflector::{DomGlobal, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::bindings::utils::{get_array_index_from_id, AsVoidPtr};
use crate::dom::dissimilaroriginwindow::DissimilarOriginWindow;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use crate::realms::{enter_realm, AlreadyInRealm, InRealm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};
use crate::script_thread::ScriptThread;

#[dom_struct]
// NOTE: the browsing context for a window is managed in two places:
// here, in script, but also in the constellation. The constellation
// manages the session history, which in script is accessed through
// History objects, messaging the constellation.
pub(crate) struct WindowProxy {
    /// The JS WindowProxy object.
    /// Unlike other reflectors, we mutate this field because
    /// we have to brain-transplant the reflector when the WindowProxy
    /// changes Window.
    reflector: Reflector,

    /// The id of the browsing context.
    /// In the case that this is a nested browsing context, this is the id
    /// of the container.
    #[no_trace]
    browsing_context_id: BrowsingContextId,

    // https://html.spec.whatwg.org/multipage/#opener-browsing-context
    #[no_trace]
    opener: Option<BrowsingContextId>,

    /// The frame id of the top-level ancestor browsing context.
    /// In the case that this is a top-level window, this is our id.
    #[no_trace]
    top_level_browsing_context_id: TopLevelBrowsingContextId,

    /// The name of the browsing context (sometimes, but not always,
    /// equal to the name of a container element)
    name: DomRefCell<DOMString>,
    /// The pipeline id of the currently active document.
    /// May be None, when the currently active document is in another script thread.
    /// We do not try to keep the pipeline id for documents in other threads,
    /// as this would require the constellation notifying many script threads about
    /// the change, which could be expensive.
    #[no_trace]
    currently_active: Cell<Option<PipelineId>>,

    /// Has the browsing context been discarded?
    discarded: Cell<bool>,

    /// Has the browsing context been disowned?
    disowned: Cell<bool>,

    /// <https://html.spec.whatwg.org/multipage/#is-closing>
    is_closing: Cell<bool>,

    /// The containing iframe element, if this is a same-origin iframe
    frame_element: Option<Dom<Element>>,

    /// The parent browsing context's window proxy, if this is a nested browsing context
    parent: Option<Dom<WindowProxy>>,

    /// <https://html.spec.whatwg.org/multipage/#delaying-load-events-mode>
    delaying_load_events_mode: Cell<bool>,

    /// The creator browsing context's base url.
    #[no_trace]
    creator_base_url: Option<ServoUrl>,

    /// The creator browsing context's url.
    #[no_trace]
    creator_url: Option<ServoUrl>,

    /// The creator browsing context's origin.
    #[no_trace]
    creator_origin: Option<ImmutableOrigin>,
}

impl WindowProxy {
    fn new_inherited(
        browsing_context_id: BrowsingContextId,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        currently_active: Option<PipelineId>,
        frame_element: Option<&Element>,
        parent: Option<&WindowProxy>,
        opener: Option<BrowsingContextId>,
        creator: CreatorBrowsingContextInfo,
    ) -> WindowProxy {
        let name = frame_element.map_or(DOMString::new(), |e| {
            e.get_string_attribute(&local_name!("name"))
        });
        WindowProxy {
            reflector: Reflector::new(),
            browsing_context_id,
            top_level_browsing_context_id,
            name: DomRefCell::new(name),
            currently_active: Cell::new(currently_active),
            discarded: Cell::new(false),
            disowned: Cell::new(false),
            is_closing: Cell::new(false),
            frame_element: frame_element.map(Dom::from_ref),
            parent: parent.map(Dom::from_ref),
            delaying_load_events_mode: Cell::new(false),
            opener,
            creator_base_url: creator.base_url,
            creator_url: creator.url,
            creator_origin: creator.origin,
        }
    }

    #[allow(unsafe_code)]
    pub(crate) fn new(
        window: &Window,
        browsing_context_id: BrowsingContextId,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        frame_element: Option<&Element>,
        parent: Option<&WindowProxy>,
        opener: Option<BrowsingContextId>,
        creator: CreatorBrowsingContextInfo,
    ) -> DomRoot<WindowProxy> {
        unsafe {
            let handler = window.windowproxy_handler();

            let cx = GlobalScope::get_cx();
            let window_jsobject = window.reflector().get_jsobject();
            assert!(!window_jsobject.get().is_null());
            assert_ne!(
                ((*get_object_class(window_jsobject.get())).flags & JSCLASS_IS_GLOBAL),
                0
            );
            let _ac = JSAutoRealm::new(*cx, window_jsobject.get());

            // Create a new window proxy.
            rooted!(in(*cx) let js_proxy = handler.new_window_proxy(&cx, window_jsobject));
            assert!(!js_proxy.is_null());

            // Create a new browsing context.
            let current = Some(window.global().pipeline_id());
            let window_proxy = Box::new(WindowProxy::new_inherited(
                browsing_context_id,
                top_level_browsing_context_id,
                current,
                frame_element,
                parent,
                opener,
                creator,
            ));

            // The window proxy owns the browsing context.
            // When we finalize the window proxy, it drops the browsing context it owns.
            SetProxyReservedSlot(
                js_proxy.get(),
                0,
                &PrivateValue((*window_proxy).as_void_ptr()),
            );

            // Notify the JS engine about the new window proxy binding.
            SetWindowProxy(*cx, window_jsobject, js_proxy.handle());

            // Set the reflector.
            debug!(
                "Initializing reflector of {:p} to {:p}.",
                window_proxy,
                js_proxy.get()
            );
            window_proxy.reflector.set_jsobject(js_proxy.get());
            DomRoot::from_ref(&*Box::into_raw(window_proxy))
        }
    }

    #[allow(unsafe_code)]
    pub(crate) fn new_dissimilar_origin(
        global_to_clone_from: &GlobalScope,
        browsing_context_id: BrowsingContextId,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        parent: Option<&WindowProxy>,
        opener: Option<BrowsingContextId>,
        creator: CreatorBrowsingContextInfo,
    ) -> DomRoot<WindowProxy> {
        unsafe {
            let handler = WindowProxyHandler::x_origin_proxy_handler();

            let cx = GlobalScope::get_cx();

            // Create a new browsing context.
            let window_proxy = Box::new(WindowProxy::new_inherited(
                browsing_context_id,
                top_level_browsing_context_id,
                None,
                None,
                parent,
                opener,
                creator,
            ));

            // Create a new dissimilar-origin window.
            let window = DissimilarOriginWindow::new(global_to_clone_from, &window_proxy);
            let window_jsobject = window.reflector().get_jsobject();
            assert!(!window_jsobject.get().is_null());
            assert_ne!(
                ((*get_object_class(window_jsobject.get())).flags & JSCLASS_IS_GLOBAL),
                0
            );
            let _ac = JSAutoRealm::new(*cx, window_jsobject.get());

            // Create a new window proxy.
            rooted!(in(*cx) let js_proxy = handler.new_window_proxy(&cx, window_jsobject));
            assert!(!js_proxy.is_null());

            // The window proxy owns the browsing context.
            // When we finalize the window proxy, it drops the browsing context it owns.
            SetProxyReservedSlot(
                js_proxy.get(),
                0,
                &PrivateValue((*window_proxy).as_void_ptr()),
            );

            // Notify the JS engine about the new window proxy binding.
            SetWindowProxy(*cx, window_jsobject, js_proxy.handle());

            // Set the reflector.
            debug!(
                "Initializing reflector of {:p} to {:p}.",
                window_proxy,
                js_proxy.get()
            );
            window_proxy.reflector.set_jsobject(js_proxy.get());
            DomRoot::from_ref(&*Box::into_raw(window_proxy))
        }
    }

    // https://html.spec.whatwg.org/multipage/#auxiliary-browsing-context
    fn create_auxiliary_browsing_context(
        &self,
        name: DOMString,
        noopener: bool,
    ) -> Option<DomRoot<WindowProxy>> {
        let (chan, port) = ipc::channel().unwrap();
        let window = self
            .currently_active
            .get()
            .and_then(ScriptThread::find_document)
            .map(|doc| DomRoot::from_ref(doc.window()))
            .unwrap();
        let msg = EmbedderMsg::AllowOpeningWebView(window.webview_id(), chan);
        window.send_to_embedder(msg);
        if let Some(new_top_level_browsing_context_id) = port.recv().unwrap() {
            let new_browsing_context_id =
                BrowsingContextId::from(new_top_level_browsing_context_id);
            let new_pipeline_id = PipelineId::new();
            let document = self
                .currently_active
                .get()
                .and_then(ScriptThread::find_document)
                .expect("A WindowProxy creating an auxiliary to have an active document");

            let blank_url = ServoUrl::parse("about:blank").ok().unwrap();
            let load_data = LoadData::new(
                LoadOrigin::Script(document.origin().immutable().clone()),
                blank_url,
                None,
                document.global().get_referrer(),
                document.get_referrer_policy(),
                None, // Doesn't inherit secure context
                None,
            );
            let load_info = AuxiliaryBrowsingContextLoadInfo {
                load_data: load_data.clone(),
                opener_pipeline_id: self.currently_active.get().unwrap(),
                new_browsing_context_id,
                new_top_level_browsing_context_id,
                new_pipeline_id,
            };

            let new_layout_info = NewLayoutInfo {
                parent_info: None,
                new_pipeline_id,
                browsing_context_id: new_browsing_context_id,
                top_level_browsing_context_id: new_top_level_browsing_context_id,
                opener: Some(self.browsing_context_id),
                load_data,
                window_size: window.window_size(),
            };
            let constellation_msg = ScriptMsg::ScriptNewAuxiliary(load_info);
            window.send_to_constellation(constellation_msg);
            ScriptThread::process_attach_layout(new_layout_info, document.origin().clone());
            // TODO: if noopener is false, copy the sessionStorage storage area of the creator origin.
            // See step 14 of https://html.spec.whatwg.org/multipage/#creating-a-new-browsing-context
            let auxiliary =
                ScriptThread::find_document(new_pipeline_id).and_then(|doc| doc.browsing_context());
            if let Some(proxy) = auxiliary {
                if name.to_lowercase() != "_blank" {
                    proxy.set_name(name);
                }
                if noopener {
                    proxy.disown();
                }
                return Some(proxy);
            }
        }
        None
    }

    /// <https://html.spec.whatwg.org/multipage/#delaying-load-events-mode>
    pub(crate) fn is_delaying_load_events_mode(&self) -> bool {
        self.delaying_load_events_mode.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#delaying-load-events-mode>
    pub(crate) fn start_delaying_load_events_mode(&self) {
        self.delaying_load_events_mode.set(true);
    }

    /// <https://html.spec.whatwg.org/multipage/#delaying-load-events-mode>
    pub(crate) fn stop_delaying_load_events_mode(&self) {
        self.delaying_load_events_mode.set(false);
        if let Some(document) = self.document() {
            if !document.loader().events_inhibited() {
                ScriptThread::mark_document_with_no_blocked_loads(&document);
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#disowned-its-opener
    pub(crate) fn disown(&self) {
        self.disowned.set(true);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-window-close>
    /// Step 3.1, set BCs `is_closing` to true.
    pub(crate) fn close(&self) {
        self.is_closing.set(true);
    }

    /// <https://html.spec.whatwg.org/multipage/#is-closing>
    pub(crate) fn is_closing(&self) -> bool {
        self.is_closing.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#creator-base-url>
    pub(crate) fn creator_base_url(&self) -> Option<ServoUrl> {
        self.creator_base_url.clone()
    }

    pub(crate) fn has_creator_base_url(&self) -> bool {
        self.creator_base_url.is_some()
    }

    /// <https://html.spec.whatwg.org/multipage/#creator-url>
    pub(crate) fn creator_url(&self) -> Option<ServoUrl> {
        self.creator_url.clone()
    }

    pub(crate) fn has_creator_url(&self) -> bool {
        self.creator_base_url.is_some()
    }

    /// <https://html.spec.whatwg.org/multipage/#creator-origin>
    pub(crate) fn creator_origin(&self) -> Option<ImmutableOrigin> {
        self.creator_origin.clone()
    }

    pub(crate) fn has_creator_origin(&self) -> bool {
        self.creator_origin.is_some()
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-opener
    pub(crate) fn opener(
        &self,
        cx: *mut JSContext,
        in_realm_proof: InRealm,
        mut retval: MutableHandleValue,
    ) {
        if self.disowned.get() {
            return retval.set(NullValue());
        }
        let opener_id = match self.opener {
            Some(opener_browsing_context_id) => opener_browsing_context_id,
            None => return retval.set(NullValue()),
        };
        let parent_browsing_context = self.parent.as_deref();
        let opener_proxy = match ScriptThread::find_window_proxy(opener_id) {
            Some(window_proxy) => window_proxy,
            None => {
                let sender_pipeline_id = self.currently_active().unwrap();
                match ScriptThread::get_top_level_for_browsing_context(
                    sender_pipeline_id,
                    opener_id,
                ) {
                    Some(opener_top_id) => {
                        let global_to_clone_from =
                            unsafe { GlobalScope::from_context(cx, in_realm_proof) };
                        let creator =
                            CreatorBrowsingContextInfo::from(parent_browsing_context, None);
                        WindowProxy::new_dissimilar_origin(
                            &global_to_clone_from,
                            opener_id,
                            opener_top_id,
                            None,
                            None,
                            creator,
                        )
                    },
                    None => return retval.set(NullValue()),
                }
            },
        };
        if opener_proxy.is_browsing_context_discarded() {
            return retval.set(NullValue());
        }
        unsafe { opener_proxy.to_jsval(cx, retval) };
    }

    // https://html.spec.whatwg.org/multipage/#window-open-steps
    pub(crate) fn open(
        &self,
        url: USVString,
        target: DOMString,
        features: DOMString,
        can_gc: CanGc,
    ) -> Fallible<Option<DomRoot<WindowProxy>>> {
        // Step 4.
        let non_empty_target = match target.as_ref() {
            "" => DOMString::from("_blank"),
            _ => target,
        };
        // Step 5
        let tokenized_features = tokenize_open_features(features);
        // Step 7-9
        let noreferrer = parse_open_feature_boolean(&tokenized_features, "noreferrer");
        let noopener = if noreferrer {
            true
        } else {
            parse_open_feature_boolean(&tokenized_features, "noopener")
        };
        // Step 10, 11
        let (chosen, new) = match self.choose_browsing_context(non_empty_target, noopener) {
            (Some(chosen), new) => (chosen, new),
            (None, _) => return Ok(None),
        };
        // TODO Step 12, set up browsing context features.
        let target_document = match chosen.document() {
            Some(target_document) => target_document,
            None => return Ok(None),
        };
        let target_window = target_document.window();
        // Step 13, and 14.4, will have happened elsewhere,
        // since we've created a new browsing context and loaded it with about:blank.
        if !url.is_empty() {
            let existing_document = self
                .currently_active
                .get()
                .and_then(ScriptThread::find_document)
                .unwrap();
            // Step 14.1
            let url = match existing_document.url().join(&url) {
                Ok(url) => url,
                Err(_) => return Err(Error::Syntax),
            };
            // Step 14.3
            let referrer = if noreferrer {
                Referrer::NoReferrer
            } else {
                target_window.as_global_scope().get_referrer()
            };
            // Step 14.5
            let referrer_policy = target_document.get_referrer_policy();
            let pipeline_id = target_window.pipeline_id();
            let secure = target_window.as_global_scope().is_secure_context();
            let load_data = LoadData::new(
                LoadOrigin::Script(existing_document.origin().immutable().clone()),
                url,
                Some(pipeline_id),
                referrer,
                referrer_policy,
                Some(secure),
                Some(target_document.insecure_requests_policy()),
            );
            let history_handling = if new {
                NavigationHistoryBehavior::Replace
            } else {
                NavigationHistoryBehavior::Push
            };

            target_window.load_url(history_handling, false, load_data, can_gc);
        }
        if noopener {
            // Step 15 (Dis-owning has been done in create_auxiliary_browsing_context).
            return Ok(None);
        }
        // Step 17.
        Ok(target_document.browsing_context())
    }

    // https://html.spec.whatwg.org/multipage/#the-rules-for-choosing-a-browsing-context-given-a-browsing-context-name
    pub(crate) fn choose_browsing_context(
        &self,
        name: DOMString,
        noopener: bool,
    ) -> (Option<DomRoot<WindowProxy>>, bool) {
        match name.to_lowercase().as_ref() {
            "" | "_self" => {
                // Step 3.
                (Some(DomRoot::from_ref(self)), false)
            },
            "_parent" => {
                // Step 4
                if let Some(parent) = self.parent() {
                    return (Some(DomRoot::from_ref(parent)), false);
                }
                (None, false)
            },
            "_top" => {
                // Step 5
                (Some(DomRoot::from_ref(self.top())), false)
            },
            "_blank" => (self.create_auxiliary_browsing_context(name, noopener), true),
            _ => {
                // Step 6.
                // TODO: expand the search to all 'familiar' bc,
                // including auxiliaries familiar by way of their opener.
                // See https://html.spec.whatwg.org/multipage/#familiar-with
                match ScriptThread::find_window_proxy_by_name(&name) {
                    Some(proxy) => (Some(proxy), false),
                    None => (self.create_auxiliary_browsing_context(name, noopener), true),
                }
            },
        }
    }

    pub(crate) fn is_auxiliary(&self) -> bool {
        self.opener.is_some()
    }

    pub(crate) fn discard_browsing_context(&self) {
        self.discarded.set(true);
    }

    pub(crate) fn is_browsing_context_discarded(&self) -> bool {
        self.discarded.get()
    }

    pub(crate) fn browsing_context_id(&self) -> BrowsingContextId {
        self.browsing_context_id
    }

    pub(crate) fn top_level_browsing_context_id(&self) -> TopLevelBrowsingContextId {
        self.top_level_browsing_context_id
    }

    pub(crate) fn frame_element(&self) -> Option<&Element> {
        self.frame_element.as_deref()
    }

    pub(crate) fn document(&self) -> Option<DomRoot<Document>> {
        self.currently_active
            .get()
            .and_then(ScriptThread::find_document)
    }

    pub(crate) fn parent(&self) -> Option<&WindowProxy> {
        self.parent.as_deref()
    }

    pub(crate) fn top(&self) -> &WindowProxy {
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
    fn set_window(&self, window: &GlobalScope, handler: &WindowProxyHandler) {
        unsafe {
            debug!("Setting window of {:p}.", self);

            let cx = GlobalScope::get_cx();
            let window_jsobject = window.reflector().get_jsobject();
            let old_js_proxy = self.reflector.get_jsobject();
            assert!(!window_jsobject.get().is_null());
            assert_ne!(
                ((*get_object_class(window_jsobject.get())).flags & JSCLASS_IS_GLOBAL),
                0
            );
            let _ac = enter_realm(window);

            // The old window proxy no longer owns this browsing context.
            SetProxyReservedSlot(old_js_proxy.get(), 0, &PrivateValue(ptr::null_mut()));

            // Brain transplant the window proxy. Brain transplantation is
            // usually done to move a window proxy between compartments, but
            // that's not what we are doing here. We need to do this just
            // because we want to replace the wrapper's `ProxyTraps`, but we
            // don't want to update its identity.
            rooted!(in(*cx) let new_js_proxy = handler.new_window_proxy(&cx, window_jsobject));
            // Explicitly set this slot to a null pointer in case a GC occurs before we
            // are ready to set it to a real value.
            SetProxyReservedSlot(new_js_proxy.get(), 0, &PrivateValue(ptr::null_mut()));
            debug!(
                "Transplanting proxy from {:p} to {:p}.",
                old_js_proxy.get(),
                new_js_proxy.get()
            );
            rooted!(in(*cx) let new_js_proxy = JS_TransplantObject(*cx, old_js_proxy, new_js_proxy.handle()));
            debug!("Transplanted proxy is {:p}.", new_js_proxy.get());

            // Transfer ownership of this browsing context from the old window proxy to the new one.
            SetProxyReservedSlot(new_js_proxy.get(), 0, &PrivateValue(self.as_void_ptr()));

            // Notify the JS engine about the new window proxy binding.
            SetWindowProxy(*cx, window_jsobject, new_js_proxy.handle());

            // Update the reflector.
            debug!(
                "Setting reflector of {:p} to {:p}.",
                self,
                new_js_proxy.get()
            );
            self.reflector.rootable().set(new_js_proxy.get());
        }
    }

    pub(crate) fn set_currently_active(&self, window: &Window) {
        if let Some(pipeline_id) = self.currently_active() {
            if pipeline_id == window.pipeline_id() {
                return debug!(
                    "Attempt to set the currently active window to the currently active window."
                );
            }
        }

        let global_scope = window.as_global_scope();
        self.set_window(global_scope, WindowProxyHandler::proxy_handler());
        self.currently_active.set(Some(global_scope.pipeline_id()));
    }

    pub(crate) fn unset_currently_active(&self) {
        if self.currently_active().is_none() {
            return debug!("Attempt to unset the currently active window on a windowproxy that does not have one.");
        }
        let globalscope = self.global();
        let window = DissimilarOriginWindow::new(&globalscope, self);
        self.set_window(
            window.upcast(),
            WindowProxyHandler::x_origin_proxy_handler(),
        );
        self.currently_active.set(None);
    }

    pub(crate) fn currently_active(&self) -> Option<PipelineId> {
        self.currently_active.get()
    }

    pub(crate) fn get_name(&self) -> DOMString {
        self.name.borrow().clone()
    }

    pub(crate) fn set_name(&self, name: DOMString) {
        *self.name.borrow_mut() = name;
    }
}

/// A browsing context can have a creator browsing context, the browsing context that
/// was responsible for its creation. If a browsing context has a parent browsing context,
/// then that is its creator browsing context. Otherwise, if the browsing context has an
/// opener browsing context, then that is its creator browsing context. Otherwise, the
/// browsing context has no creator browsing context.
///
/// If a browsing context A has a creator browsing context, then the Document that was the
/// active document of that creator browsing context at the time A was created is the creator
/// Document.
///
/// See: <https://html.spec.whatwg.org/multipage/#creating-browsing-contexts>
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CreatorBrowsingContextInfo {
    /// Creator document URL.
    url: Option<ServoUrl>,

    /// Creator document base URL.
    base_url: Option<ServoUrl>,

    /// Creator document origin.
    origin: Option<ImmutableOrigin>,
}

impl CreatorBrowsingContextInfo {
    pub(crate) fn from(
        parent: Option<&WindowProxy>,
        opener: Option<&WindowProxy>,
    ) -> CreatorBrowsingContextInfo {
        let creator = match (parent, opener) {
            (Some(parent), _) => parent.document(),
            (None, Some(opener)) => opener.document(),
            (None, None) => None,
        };

        let base_url = creator.as_deref().map(|document| document.base_url());
        let url = creator.as_deref().map(|document| document.url());
        let origin = creator
            .as_deref()
            .map(|document| document.origin().immutable().clone());

        CreatorBrowsingContextInfo {
            base_url,
            url,
            origin,
        }
    }
}

// https://html.spec.whatwg.org/multipage/#concept-window-open-features-tokenize
fn tokenize_open_features(features: DOMString) -> IndexMap<String, String> {
    let is_feature_sep = |c: char| c.is_ascii_whitespace() || ['=', ','].contains(&c);
    // Step 1
    let mut tokenized_features = IndexMap::new();
    // Step 2
    let mut iter = features.chars();
    let mut cur = iter.next();

    // Step 3
    while cur.is_some() {
        // Step 3.1 & 3.2
        let mut name = String::new();
        let mut value = String::new();
        // Step 3.3
        while let Some(cur_char) = cur {
            if !is_feature_sep(cur_char) {
                break;
            }
            cur = iter.next();
        }
        // Step 3.4
        while let Some(cur_char) = cur {
            if is_feature_sep(cur_char) {
                break;
            }
            name.push(cur_char.to_ascii_lowercase());
            cur = iter.next();
        }
        // Step 3.5
        let normalized_name = String::from(match name.as_ref() {
            "screenx" => "left",
            "screeny" => "top",
            "innerwidth" => "width",
            "innerheight" => "height",
            _ => name.as_ref(),
        });
        // Step 3.6
        while let Some(cur_char) = cur {
            if cur_char == '=' || cur_char == ',' || !is_feature_sep(cur_char) {
                break;
            }
            cur = iter.next();
        }
        // Step 3.7
        if cur.is_some() && is_feature_sep(cur.unwrap()) {
            // Step 3.7.1
            while let Some(cur_char) = cur {
                if !is_feature_sep(cur_char) || cur_char == ',' {
                    break;
                }
                cur = iter.next();
            }
            // Step 3.7.2
            while let Some(cur_char) = cur {
                if is_feature_sep(cur_char) {
                    break;
                }
                value.push(cur_char.to_ascii_lowercase());
                cur = iter.next();
            }
        }
        // Step 3.8
        if !name.is_empty() {
            tokenized_features.insert(normalized_name, value);
        }
    }
    // Step 4
    tokenized_features
}

// https://html.spec.whatwg.org/multipage/#concept-window-open-features-parse-boolean
fn parse_open_feature_boolean(tokenized_features: &IndexMap<String, String>, name: &str) -> bool {
    if let Some(value) = tokenized_features.get(name) {
        // Step 1 & 2
        if value.is_empty() || value == "yes" {
            return true;
        }
        // Step 3 & 4
        if let Ok(int) = parse_integer(value.chars()) {
            return int != 0;
        }
    }
    // Step 5
    false
}

// This is only called from extern functions,
// there's no use using the lifetimed handles here.
// https://html.spec.whatwg.org/multipage/#accessing-other-browsing-contexts
#[allow(unsafe_code, non_snake_case)]
unsafe fn GetSubframeWindowProxy(
    cx: *mut JSContext,
    proxy: RawHandleObject,
    id: RawHandleId,
) -> Option<(DomRoot<WindowProxy>, u32)> {
    let index = get_array_index_from_id(cx, Handle::from_raw(id));
    if let Some(index) = index {
        let mut slot = UndefinedValue();
        GetProxyPrivate(*proxy, &mut slot);
        rooted!(in(cx) let target = slot.to_object());
        if let Ok(win) = root_from_handleobject::<Window>(target.handle(), cx) {
            let browsing_context_id = win.window_proxy().browsing_context_id();
            let (result_sender, result_receiver) = ipc::channel().unwrap();

            let _ = win.as_global_scope().script_to_constellation_chan().send(
                ScriptMsg::GetChildBrowsingContextId(
                    browsing_context_id,
                    index as usize,
                    result_sender,
                ),
            );
            return result_receiver
                .recv()
                .ok()
                .and_then(|maybe_bcid| maybe_bcid)
                .and_then(ScriptThread::find_window_proxy)
                .map(|proxy| (proxy, (JSPROP_ENUMERATE | JSPROP_READONLY) as u32));
        } else if let Ok(win) =
            root_from_handleobject::<DissimilarOriginWindow>(target.handle(), cx)
        {
            let browsing_context_id = win.window_proxy().browsing_context_id();
            let (result_sender, result_receiver) = ipc::channel().unwrap();

            let _ = win.global().script_to_constellation_chan().send(
                ScriptMsg::GetChildBrowsingContextId(
                    browsing_context_id,
                    index as usize,
                    result_sender,
                ),
            );
            return result_receiver
                .recv()
                .ok()
                .and_then(|maybe_bcid| maybe_bcid)
                .and_then(ScriptThread::find_window_proxy)
                .map(|proxy| (proxy, JSPROP_READONLY as u32));
        }
    }

    None
}

#[allow(unsafe_code, non_snake_case)]
unsafe extern "C" fn getOwnPropertyDescriptor(
    cx: *mut JSContext,
    proxy: RawHandleObject,
    id: RawHandleId,
    desc: RawMutableHandle<PropertyDescriptor>,
    is_none: *mut bool,
) -> bool {
    let window = GetSubframeWindowProxy(cx, proxy, id);
    if let Some((window, attrs)) = window {
        rooted!(in(cx) let mut val = UndefinedValue());
        window.to_jsval(cx, val.handle_mut());
        set_property_descriptor(
            MutableHandle::from_raw(desc),
            val.handle(),
            attrs,
            &mut *is_none,
        );
        return true;
    }

    let mut slot = UndefinedValue();
    GetProxyPrivate(proxy.get(), &mut slot);
    rooted!(in(cx) let target = slot.to_object());
    JS_GetOwnPropertyDescriptorById(cx, target.handle().into(), id, desc, is_none)
}

#[allow(unsafe_code, non_snake_case)]
unsafe extern "C" fn defineProperty(
    cx: *mut JSContext,
    proxy: RawHandleObject,
    id: RawHandleId,
    desc: RawHandle<PropertyDescriptor>,
    res: *mut ObjectOpResult,
) -> bool {
    if get_array_index_from_id(cx, Handle::from_raw(id)).is_some() {
        // Spec says to Reject whether this is a supported index or not,
        // since we have no indexed setter or indexed creator.  That means
        // throwing in strict mode (FIXME: Bug 828137), doing nothing in
        // non-strict mode.
        (*res).code_ = JSErrNum::JSMSG_CANT_DEFINE_WINDOW_ELEMENT as ::libc::uintptr_t;
        return true;
    }

    let mut slot = UndefinedValue();
    GetProxyPrivate(*proxy.ptr, &mut slot);
    rooted!(in(cx) let target = slot.to_object());
    JS_DefinePropertyById(cx, target.handle().into(), id, desc, res)
}

#[allow(unsafe_code)]
unsafe extern "C" fn has(
    cx: *mut JSContext,
    proxy: RawHandleObject,
    id: RawHandleId,
    bp: *mut bool,
) -> bool {
    let window = GetSubframeWindowProxy(cx, proxy, id);
    if window.is_some() {
        *bp = true;
        return true;
    }

    let mut slot = UndefinedValue();
    GetProxyPrivate(*proxy.ptr, &mut slot);
    rooted!(in(cx) let target = slot.to_object());
    let mut found = false;
    if !JS_HasPropertyById(cx, target.handle().into(), id, &mut found) {
        return false;
    }

    *bp = found;
    true
}

#[allow(unsafe_code)]
unsafe extern "C" fn get(
    cx: *mut JSContext,
    proxy: RawHandleObject,
    receiver: RawHandleValue,
    id: RawHandleId,
    vp: RawMutableHandleValue,
) -> bool {
    let window = GetSubframeWindowProxy(cx, proxy, id);
    if let Some((window, _attrs)) = window {
        window.to_jsval(cx, MutableHandle::from_raw(vp));
        return true;
    }

    let mut slot = UndefinedValue();
    GetProxyPrivate(*proxy.ptr, &mut slot);
    rooted!(in(cx) let target = slot.to_object());
    JS_ForwardGetPropertyTo(cx, target.handle().into(), id, receiver, vp)
}

#[allow(unsafe_code)]
unsafe extern "C" fn set(
    cx: *mut JSContext,
    proxy: RawHandleObject,
    id: RawHandleId,
    v: RawHandleValue,
    receiver: RawHandleValue,
    res: *mut ObjectOpResult,
) -> bool {
    if get_array_index_from_id(cx, Handle::from_raw(id)).is_some() {
        // Reject (which means throw if and only if strict) the set.
        (*res).code_ = JSErrNum::JSMSG_READ_ONLY as ::libc::uintptr_t;
        return true;
    }

    let mut slot = UndefinedValue();
    GetProxyPrivate(*proxy.ptr, &mut slot);
    rooted!(in(cx) let target = slot.to_object());
    JS_ForwardSetPropertyTo(cx, target.handle().into(), id, v, receiver, res)
}

#[allow(unsafe_code)]
unsafe extern "C" fn get_prototype_if_ordinary(
    _: *mut JSContext,
    _: RawHandleObject,
    is_ordinary: *mut bool,
    _: RawMutableHandleObject,
) -> bool {
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
    true
}

static PROXY_TRAPS: ProxyTraps = ProxyTraps {
    // TODO: These traps should change their behavior depending on
    //       `IsPlatformObjectSameOrigin(this.[[Window]])`
    enter: None,
    getOwnPropertyDescriptor: Some(getOwnPropertyDescriptor),
    defineProperty: Some(defineProperty),
    ownPropertyKeys: None,
    delete_: None,
    enumerate: None,
    getPrototypeIfOrdinary: Some(get_prototype_if_ordinary),
    getPrototype: None, // TODO: return `null` if cross origin-domain
    setPrototype: None,
    setImmutablePrototype: None,
    preventExtensions: None,
    isExtensible: None,
    has: Some(has),
    get: Some(get),
    set: Some(set),
    call: None,
    construct: None,
    hasOwn: None,
    getOwnEnumerablePropertyKeys: None,
    nativeCall: None,
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

/// Proxy handler for a WindowProxy.
/// Has ownership of the inner pointer and deallocates it when it is no longer needed.
pub(crate) struct WindowProxyHandler(*const libc::c_void);

impl MallocSizeOf for WindowProxyHandler {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        // FIXME(#6907) this is a pointer to memory allocated by `new` in NewProxyHandler in rust-mozjs.
        0
    }
}

// Safety: Send and Sync is guaranteed since the underlying pointer and all its associated methods in C++ are const.
#[allow(unsafe_code)]
unsafe impl Send for WindowProxyHandler {}
// Safety: Send and Sync is guaranteed since the underlying pointer and all its associated methods in C++ are const.
#[allow(unsafe_code)]
unsafe impl Sync for WindowProxyHandler {}

#[allow(unsafe_code)]
impl WindowProxyHandler {
    fn new(traps: &ProxyTraps) -> Self {
        // Safety: Foreign function generated by bindgen. Pointer is freed in drop to prevent memory leak.
        let ptr = unsafe { CreateWrapperProxyHandler(traps) };
        assert!(!ptr.is_null());
        Self(ptr)
    }

    /// Returns a single, shared WindowProxyHandler that contains XORIGIN_PROXY_TRAPS.
    pub(crate) fn x_origin_proxy_handler() -> &'static Self {
        use std::sync::OnceLock;
        /// We are sharing a single instance for the entire programs here due to lifetime issues.
        /// The pointer in self.0 is known to C++ and visited by the GC. Hence, we don't know when
        /// it is safe to free it.
        /// Sharing a single instance should be fine because all methods on this pointer in C++
        /// are const and don't modify its internal state.
        static SINGLETON: OnceLock<WindowProxyHandler> = OnceLock::new();
        SINGLETON.get_or_init(|| Self::new(&XORIGIN_PROXY_TRAPS))
    }

    /// Returns a single, shared WindowProxyHandler that contains normal PROXY_TRAPS.
    pub(crate) fn proxy_handler() -> &'static Self {
        use std::sync::OnceLock;
        /// We are sharing a single instance for the entire programs here due to lifetime issues.
        /// The pointer in self.0 is known to C++ and visited by the GC. Hence, we don't know when
        /// it is safe to free it.
        /// Sharing a single instance should be fine because all methods on this pointer in C++
        /// are const and don't modify its internal state.
        static SINGLETON: OnceLock<WindowProxyHandler> = OnceLock::new();
        SINGLETON.get_or_init(|| Self::new(&PROXY_TRAPS))
    }

    /// Creates a new WindowProxy object on the C++ side and returns the pointer to it.
    /// The pointer should be owned by the GC.
    pub(crate) fn new_window_proxy(
        &self,
        cx: &crate::script_runtime::JSContext,
        window_jsobject: js::gc::HandleObject,
    ) -> *mut JSObject {
        let obj = unsafe { NewWindowProxy(**cx, window_jsobject, self.0) };
        assert!(!obj.is_null());
        obj
    }
}

#[allow(unsafe_code)]
impl Drop for WindowProxyHandler {
    fn drop(&mut self) {
        // Safety: Pointer is allocated by corresponding C++ function, owned by this
        // struct and not accessible from outside.
        unsafe {
            DeleteWrapperProxyHandler(self.0);
        }
    }
}

// The proxy traps for cross-origin windows.
// These traps often throw security errors, and only pass on calls to methods
// defined in the DissimilarOriginWindow IDL.

// TODO: reuse the infrastructure in `proxyhandler.rs`. For starters, the calls
//       to this function should be replaced with those to
//       `report_cross_origin_denial`.
#[allow(unsafe_code)]
unsafe fn throw_security_error(cx: *mut JSContext, realm: InRealm) -> bool {
    if !JS_IsExceptionPending(cx) {
        let safe_context = SafeJSContext::from_ptr(cx);
        let global = GlobalScope::from_context(cx, realm);
        throw_dom_exception(safe_context, &global, Error::Security);
    }
    false
}

#[allow(unsafe_code)]
unsafe extern "C" fn has_xorigin(
    cx: *mut JSContext,
    proxy: RawHandleObject,
    id: RawHandleId,
    bp: *mut bool,
) -> bool {
    let mut slot = UndefinedValue();
    GetProxyPrivate(*proxy.ptr, &mut slot);
    rooted!(in(cx) let target = slot.to_object());
    let mut found = false;
    JS_HasOwnPropertyById(cx, target.handle().into(), id, &mut found);
    if found {
        *bp = true;
        true
    } else {
        let in_realm_proof = AlreadyInRealm::assert_for_cx(SafeJSContext::from_ptr(cx));
        throw_security_error(cx, InRealm::Already(&in_realm_proof))
    }
}

#[allow(unsafe_code)]
unsafe extern "C" fn get_xorigin(
    cx: *mut JSContext,
    proxy: RawHandleObject,
    receiver: RawHandleValue,
    id: RawHandleId,
    vp: RawMutableHandleValue,
) -> bool {
    let mut found = false;
    has_xorigin(cx, proxy, id, &mut found);
    found && get(cx, proxy, receiver, id, vp)
}

#[allow(unsafe_code)]
unsafe extern "C" fn set_xorigin(
    cx: *mut JSContext,
    _: RawHandleObject,
    _: RawHandleId,
    _: RawHandleValue,
    _: RawHandleValue,
    _: *mut ObjectOpResult,
) -> bool {
    let in_realm_proof = AlreadyInRealm::assert_for_cx(SafeJSContext::from_ptr(cx));
    throw_security_error(cx, InRealm::Already(&in_realm_proof))
}

#[allow(unsafe_code)]
unsafe extern "C" fn delete_xorigin(
    cx: *mut JSContext,
    _: RawHandleObject,
    _: RawHandleId,
    _: *mut ObjectOpResult,
) -> bool {
    let in_realm_proof = AlreadyInRealm::assert_for_cx(SafeJSContext::from_ptr(cx));
    throw_security_error(cx, InRealm::Already(&in_realm_proof))
}

#[allow(unsafe_code, non_snake_case)]
unsafe extern "C" fn getOwnPropertyDescriptor_xorigin(
    cx: *mut JSContext,
    proxy: RawHandleObject,
    id: RawHandleId,
    desc: RawMutableHandle<PropertyDescriptor>,
    is_none: *mut bool,
) -> bool {
    let mut found = false;
    has_xorigin(cx, proxy, id, &mut found);
    found && getOwnPropertyDescriptor(cx, proxy, id, desc, is_none)
}

#[allow(unsafe_code, non_snake_case)]
unsafe extern "C" fn defineProperty_xorigin(
    cx: *mut JSContext,
    _: RawHandleObject,
    _: RawHandleId,
    _: RawHandle<PropertyDescriptor>,
    _: *mut ObjectOpResult,
) -> bool {
    let in_realm_proof = AlreadyInRealm::assert_for_cx(SafeJSContext::from_ptr(cx));
    throw_security_error(cx, InRealm::Already(&in_realm_proof))
}

#[allow(unsafe_code, non_snake_case)]
unsafe extern "C" fn preventExtensions_xorigin(
    cx: *mut JSContext,
    _: RawHandleObject,
    _: *mut ObjectOpResult,
) -> bool {
    let in_realm_proof = AlreadyInRealm::assert_for_cx(SafeJSContext::from_ptr(cx));
    throw_security_error(cx, InRealm::Already(&in_realm_proof))
}

static XORIGIN_PROXY_TRAPS: ProxyTraps = ProxyTraps {
    enter: None,
    getOwnPropertyDescriptor: Some(getOwnPropertyDescriptor_xorigin),
    defineProperty: Some(defineProperty_xorigin),
    ownPropertyKeys: None,
    delete_: Some(delete_xorigin),
    enumerate: None,
    getPrototypeIfOrdinary: None,
    getPrototype: None,
    setPrototype: None,
    setImmutablePrototype: None,
    preventExtensions: Some(preventExtensions_xorigin),
    isExtensible: None,
    has: Some(has_xorigin),
    get: Some(get_xorigin),
    set: Some(set_xorigin),
    call: None,
    construct: None,
    hasOwn: Some(has_xorigin),
    getOwnEnumerablePropertyKeys: None,
    nativeCall: None,
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
unsafe extern "C" fn finalize(_fop: *mut GCContext, obj: *mut JSObject) {
    let mut slot = UndefinedValue();
    GetProxyReservedSlot(obj, 0, &mut slot);
    let this = slot.to_private() as *mut WindowProxy;
    if this.is_null() {
        // GC during obj creation or after transplanting.
        return;
    }
    let jsobject = (*this).reflector.get_jsobject().get();
    debug!(
        "WindowProxy finalize: {:p}, with reflector {:p} from {:p}.",
        this, jsobject, obj
    );
    let _ = Box::from_raw(this);
}

#[allow(unsafe_code)]
unsafe extern "C" fn trace(trc: *mut JSTracer, obj: *mut JSObject) {
    let mut slot = UndefinedValue();
    GetProxyReservedSlot(obj, 0, &mut slot);
    let this = slot.to_private() as *const WindowProxy;
    if this.is_null() {
        // GC during obj creation or after transplanting.
        return;
    }
    (*this).trace(trc);
}
