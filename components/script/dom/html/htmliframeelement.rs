/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use base::id::{BrowsingContextId, PipelineId, WebViewId};
use constellation_traits::{
    IFrameLoadInfo, IFrameLoadInfoWithData, JsEvalResult, LoadData, LoadOrigin,
    NavigationHistoryBehavior, ScriptToConstellationMessage,
};
use content_security_policy::sandboxing_directive::{
    SandboxingFlagSet, parse_a_sandboxing_directive,
};
use dom_struct::dom_struct;
use embedder_traits::ViewportDetails;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::context::JSContext;
use js::rust::HandleObject;
use net_traits::ReferrerPolicy;
use net_traits::request::Destination;
use profile_traits::ipc as ProfiledIpc;
use script_bindings::script_runtime::temp_cx;
use script_traits::{NewPipelineInfo, UpdatePipelineIdReason};
use servo_url::ServoUrl;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};
use stylo_atoms::Atom;

use crate::document_loader::{LoadBlocker, LoadType};
use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLIFrameElementBinding::HTMLIFrameElementMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::Window_Binding::WindowMethods;
use crate::dom::bindings::codegen::UnionTypes::TrustedHTMLOrString;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{DomRoot, LayoutDom, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::document::Document;
use crate::dom::domtokenlist::DOMTokenList;
use crate::dom::element::{
    AttributeMutation, Element, LayoutElementHelpers, reflect_referrer_policy_attribute,
};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::node::{BindContext, Node, NodeDamage, NodeTraits, UnbindContext};
use crate::dom::performance::performanceresourcetiming::InitiatorType;
use crate::dom::trustedhtml::TrustedHTML;
use crate::dom::virtualmethods::VirtualMethods;
use crate::dom::windowproxy::WindowProxy;
use crate::network_listener::ResourceTimingListener;
use crate::script_runtime::CanGc;
use crate::script_thread::{ScriptThread, with_script_thread};
use crate::script_window_proxies::ScriptWindowProxies;

#[derive(PartialEq)]
enum PipelineType {
    InitialAboutBlank,
    Navigation,
}

#[derive(PartialEq)]
enum ProcessingMode {
    FirstTime,
    NotFirstTime,
}

/// <https://html.spec.whatwg.org/multipage/#lazy-load-resumption-steps>
#[derive(Clone, Copy, Default, MallocSizeOf, PartialEq)]
enum LazyLoadResumptionSteps {
    #[default]
    None,
    SrcDoc,
}

#[dom_struct]
pub(crate) struct HTMLIFrameElement {
    htmlelement: HTMLElement,
    #[no_trace]
    webview_id: Cell<Option<WebViewId>>,
    #[no_trace]
    browsing_context_id: Cell<Option<BrowsingContextId>>,
    #[no_trace]
    pipeline_id: Cell<Option<PipelineId>>,
    #[no_trace]
    pending_pipeline_id: Cell<Option<PipelineId>>,
    #[no_trace]
    about_blank_pipeline_id: Cell<Option<PipelineId>>,
    sandbox: MutNullableDom<DOMTokenList>,
    #[no_trace]
    sandboxing_flag_set: Cell<Option<SandboxingFlagSet>>,
    load_blocker: DomRefCell<Option<LoadBlocker>>,
    throttled: Cell<bool>,
    #[conditional_malloc_size_of]
    script_window_proxies: Rc<ScriptWindowProxies>,
    /// <https://html.spec.whatwg.org/multipage/#current-navigation-was-lazy-loaded>
    current_navigation_was_lazy_loaded: Cell<bool>,
    /// <https://html.spec.whatwg.org/multipage/#lazy-load-resumption-steps>
    #[no_trace]
    lazy_load_resumption_steps: Cell<LazyLoadResumptionSteps>,
    /// Keeping track of whether the iframe will be navigated
    /// outside of the processing of it's attribute(for example: form navigation).
    /// This is necessary to prevent the iframe load event steps
    /// from asynchronously running for the initial blank document
    /// while script at this point(when the flag is set)
    /// expects those to run only for the navigated documented.
    pending_navigation: Cell<bool>,
    /// Whether a load event was synchronously fired, for example when
    /// an empty iframe is attached. In that case, we shouldn't fire a
    /// subsequent asynchronous load event.
    already_fired_synchronous_load_event: Cell<bool>,
}

impl HTMLIFrameElement {
    /// <https://html.spec.whatwg.org/multipage/#shared-attribute-processing-steps-for-iframe-and-frame-elements>,
    fn shared_attribute_processing_steps_for_iframe_and_frame_elements(&self) -> Option<ServoUrl> {
        let element = self.upcast::<Element>();
        // Step 2. If element has a src attribute specified, and its value is not the empty string, then:
        let url = element
            .get_attribute(&ns!(), &local_name!("src"))
            .and_then(|src| {
                let url = src.value();
                if url.is_empty() {
                    None
                } else {
                    // Step 2.1. Let maybeURL be the result of encoding-parsing a URL given that attribute's value,
                    // relative to element's node document.
                    // Step 2.2. If maybeURL is not failure, then set url to maybeURL.
                    self.owner_document().base_url().join(&url).ok()
                }
            })
            // Step 1. Let url be the URL record about:blank.
            .unwrap_or_else(|| ServoUrl::parse("about:blank").unwrap());
        // Step 3. If the inclusive ancestor navigables of element's node navigable contains
        // a navigable whose active document's URL equals url with exclude fragments set to true, then return null.
        // TODO

        // Step 4. If url matches about:blank and initialInsertion is true, then perform the URL and history update steps
        // given element's content navigable's active document and url.
        // TODO

        // Step 5. Return url.
        Some(url)
    }

    pub(crate) fn navigate_or_reload_child_browsing_context(
        &self,
        load_data: LoadData,
        history_handling: NavigationHistoryBehavior,
        cx: &mut js::context::JSContext,
    ) {
        // In case we fired a synchronous load event, but navigate away
        // in the event listener of that event, then we should still
        // fire a second asynchronous load event when that navigation
        // finishes. Therefore, on any navigation (but not the initial
        // about blank), we should always set this to false, regardless
        // of whether we synchronously fired a load in the same microtask.
        self.already_fired_synchronous_load_event.set(false);

        self.start_new_pipeline(load_data, PipelineType::Navigation, history_handling, cx);
    }

    fn start_new_pipeline(
        &self,
        mut load_data: LoadData,
        pipeline_type: PipelineType,
        history_handling: NavigationHistoryBehavior,
        cx: &mut js::context::JSContext,
    ) {
        let browsing_context_id = match self.browsing_context_id() {
            None => return warn!("Attempted to start a new pipeline on an unattached iframe."),
            Some(id) => id,
        };

        let webview_id = match self.webview_id() {
            None => return warn!("Attempted to start a new pipeline on an unattached iframe."),
            Some(id) => id,
        };

        let document = self.owner_document();

        {
            let load_blocker = &self.load_blocker;
            // Any oustanding load is finished from the point of view of the blocked
            // document; the new navigation will continue blocking it.
            LoadBlocker::terminate(load_blocker, cx);
        }

        if load_data.url.scheme() == "javascript" {
            let window_proxy = self.GetContentWindow();
            if let Some(window_proxy) = window_proxy {
                if !ScriptThread::navigate_to_javascript_url(
                    cx,
                    &document.global(),
                    &window_proxy.global(),
                    &mut load_data,
                    Some(self.upcast()),
                ) {
                    return;
                }
                load_data.about_base_url = document.about_base_url();
            }
        }

        match load_data.js_eval_result {
            Some(JsEvalResult::NoContent) => (),
            _ => {
                let mut load_blocker = self.load_blocker.borrow_mut();
                *load_blocker = Some(LoadBlocker::new(
                    &document,
                    LoadType::Subframe(load_data.url.clone()),
                ));
            },
        };

        let window = self.owner_window();
        let old_pipeline_id = self.pipeline_id();
        let new_pipeline_id = PipelineId::new();
        self.pending_pipeline_id.set(Some(new_pipeline_id));

        let load_info = IFrameLoadInfo {
            parent_pipeline_id: window.pipeline_id(),
            browsing_context_id,
            webview_id,
            new_pipeline_id,
            is_private: false, // FIXME
            inherited_secure_context: load_data.inherited_secure_context,
            history_handling,
        };

        let viewport_details = window
            .get_iframe_viewport_details_if_known(browsing_context_id)
            .unwrap_or_else(|| ViewportDetails {
                hidpi_scale_factor: window.device_pixel_ratio(),
                ..Default::default()
            });

        match pipeline_type {
            PipelineType::InitialAboutBlank => {
                self.about_blank_pipeline_id.set(Some(new_pipeline_id));

                let load_info = IFrameLoadInfoWithData {
                    info: load_info,
                    load_data: load_data.clone(),
                    old_pipeline_id,
                    viewport_details,
                    theme: window.theme(),
                };
                window
                    .as_global_scope()
                    .script_to_constellation_chan()
                    .send(ScriptToConstellationMessage::ScriptNewIFrame(load_info))
                    .unwrap();

                let new_pipeline_info = NewPipelineInfo {
                    parent_info: Some(window.pipeline_id()),
                    new_pipeline_id,
                    browsing_context_id,
                    webview_id,
                    opener: None,
                    load_data,
                    viewport_details,
                    user_content_manager_id: None,
                    theme: window.theme(),
                };

                self.pipeline_id.set(Some(new_pipeline_id));
                with_script_thread(|script_thread| {
                    script_thread.spawn_pipeline(new_pipeline_info);
                });
            },
            PipelineType::Navigation => {
                let load_info = IFrameLoadInfoWithData {
                    info: load_info,
                    load_data,
                    old_pipeline_id,
                    viewport_details,
                    theme: window.theme(),
                };
                window
                    .as_global_scope()
                    .script_to_constellation_chan()
                    .send(ScriptToConstellationMessage::ScriptLoadedURLInIFrame(
                        load_info,
                    ))
                    .unwrap();
            },
        }
    }

    /// When an iframe is first inserted into the document,
    /// an "about:blank" document is created,
    /// and synchronously processed by the script thread.
    /// This initial synchronous load should have no noticeable effect in script.
    /// See the note in `iframe_load_event_steps`.
    pub(crate) fn is_initial_blank_document(&self) -> bool {
        self.pending_pipeline_id.get() == self.about_blank_pipeline_id.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#navigate-an-iframe-or-frame>
    fn navigate_an_iframe_or_frame(&self, cx: &mut js::context::JSContext, load_data: LoadData) {
        // Step 2. If element's content navigable's active document is not completely loaded,
        // then set historyHandling to "replace".
        let history_handling = if !self
            .GetContentDocument()
            .is_some_and(|doc| doc.completely_loaded())
        {
            NavigationHistoryBehavior::Replace
        } else {
            // Step 1. Let historyHandling be "auto".
            NavigationHistoryBehavior::Auto
        };
        // Step 3. If element is an iframe, then set element's pending resource-timing start time
        // to the current high resolution time given element's node document's relevant global object.
        // TODO

        // Step 4. Navigate element's content navigable to url using element's node document,
        // with historyHandling set to historyHandling, referrerPolicy set to referrerPolicy,
        // documentResource set to srcdocString, and initialInsertion set to initialInsertion.
        self.navigate_or_reload_child_browsing_context(load_data, history_handling, cx);
    }

    /// <https://html.spec.whatwg.org/multipage/#will-lazy-load-element-steps>
    fn will_lazy_load_element_steps(&self) -> bool {
        // Step 1. If scripting is disabled for element, then return false.
        if !self.owner_document().scripting_enabled() {
            return false;
        }
        // Step 2. If element's lazy loading attribute is in the Lazy state, then return true.
        // Step 3. Return false.
        self.Loading() == "lazy"
    }

    /// Step 1.3. of <https://html.spec.whatwg.org/multipage/#process-the-iframe-attributes>
    fn navigate_to_the_srcdoc_resource(&self, cx: &mut js::context::JSContext) {
        // Step 1.3. Navigate to the srcdoc resource: Navigate an iframe or frame given element,
        // about:srcdoc, the empty string, and the value of element's srcdoc attribute.
        let url = ServoUrl::parse("about:srcdoc").unwrap();
        let document = self.owner_document();
        let window = self.owner_window();
        let pipeline_id = Some(window.pipeline_id());
        let mut load_data = LoadData::new(
            LoadOrigin::Script(document.origin().snapshot()),
            url,
            Some(document.base_url()),
            pipeline_id,
            window.as_global_scope().get_referrer(),
            document.get_referrer_policy(),
            Some(window.as_global_scope().is_secure_context()),
            Some(document.insecure_requests_policy()),
            document.has_trustworthy_ancestor_or_current_origin(),
            self.sandboxing_flag_set(),
        );
        load_data.destination = Destination::IFrame;
        load_data.policy_container = Some(window.as_global_scope().policy_container());
        load_data.srcdoc = String::from(
            self.upcast::<Element>()
                .get_string_attribute(&local_name!("srcdoc")),
        );

        self.navigate_an_iframe_or_frame(cx, load_data);
    }

    /// <https://html.spec.whatwg.org/multipage/#the-iframe-element:potentially-delays-the-load-event>
    fn mark_navigation_as_lazy_loaded(&self, cx: &mut js::context::JSContext) {
        // > An iframe element whose current navigation was lazy loaded boolean is false potentially delays the load event.
        self.current_navigation_was_lazy_loaded.set(true);
        let blocker = &self.load_blocker;
        LoadBlocker::terminate(blocker, cx);
    }

    /// <https://html.spec.whatwg.org/multipage/#process-the-iframe-attributes>
    fn process_the_iframe_attributes(&self, mode: ProcessingMode, cx: &mut js::context::JSContext) {
        let element = self.upcast::<Element>();
        // Step 1. If `element`'s `srcdoc` attribute is specified, then:
        //
        // Note that this also includes the empty string
        if element.has_attribute(&local_name!("srcdoc")) {
            // Step 1.1. Set element's current navigation was lazy loaded boolean to false.
            self.current_navigation_was_lazy_loaded.set(false);
            // Step 1.2. If the will lazy load element steps given element return true, then:
            if self.will_lazy_load_element_steps() {
                // Step 1.2.1. Set element's lazy load resumption steps to the rest of this algorithm
                // starting with the step labeled navigate to the srcdoc resource.
                self.lazy_load_resumption_steps
                    .set(LazyLoadResumptionSteps::SrcDoc);
                // Step 1.2.2. Set element's current navigation was lazy loaded boolean to true.
                self.mark_navigation_as_lazy_loaded(cx);
                // Step 1.2.3. Start intersection-observing a lazy loading element for element.
                // TODO
                // Step 1.2.4. Return.
                return;
            }
            // Step 1.3. Navigate to the srcdoc resource: Navigate an iframe or frame given element,
            // about:srcdoc, the empty string, and the value of element's srcdoc attribute.
            self.navigate_to_the_srcdoc_resource(cx);
            return;
        }

        let window = self.owner_window();

        // https://html.spec.whatwg.org/multipage/#attr-iframe-name
        // Note: the spec says to set the name 'when the nested browsing context is created'.
        // The current implementation sets the name on the window,
        // when the iframe attributes are first processed.
        if mode == ProcessingMode::FirstTime {
            if let Some(window) = self.GetContentWindow() {
                window.set_name(
                    element
                        .get_name()
                        .map_or(DOMString::from(""), |n| DOMString::from(&*n)),
                );
            }
        }

        // Step 2.1. Let url be the result of running the shared attribute processing steps
        // for iframe and frame elements given element and initialInsertion.
        let Some(url) = self.shared_attribute_processing_steps_for_iframe_and_frame_elements()
        else {
            // Step 2.2. If url is null, then return.
            return;
        };

        // Step 2.3. If url matches about:blank and initialInsertion is true, then:
        if url.matches_about_blank() && mode == ProcessingMode::FirstTime {
            // We should **not** send a load event in `iframe_load_event_steps`.
            self.already_fired_synchronous_load_event.set(true);
            // Step 2.3.1. Run the iframe load event steps given element.
            //
            // Note: we are not actually calling that method. That's because
            // `iframe_load_event_steps` currently doesn't adhere to the spec
            // at all. In this case, WPT tests only care about the load event,
            // so we can fire that. Following https://github.com/servo/servo/issues/31973
            // we should call `iframe_load_event_steps` once it is spec-compliant.
            self.upcast::<EventTarget>()
                .fire_event(atom!("load"), CanGc::from_cx(cx));
            // Step 2.3.2. Return.
            return;
        }

        // Step 2.4: Let referrerPolicy be the current state of element's referrerpolicy content
        // attribute.
        let document = self.owner_document();
        let referrer_policy_token = self.ReferrerPolicy();

        // Note: despite not being explicitly stated in the spec steps, this falls back to
        // document's referrer policy here because it satisfies the expectations that when unset,
        // the iframe should inherit the referrer policy of its parent
        let referrer_policy = match ReferrerPolicy::from(&*referrer_policy_token.str()) {
            ReferrerPolicy::EmptyString => document.get_referrer_policy(),
            policy => policy,
        };

        // TODO(#25748):
        // By spec, we return early if there's an ancestor browsing context
        // "whose active document's url, ignoring fragments, is equal".
        // However, asking about ancestor browsing contexts is more nuanced than
        // it sounds and not implemented here.
        // Within a single origin, we can do it by walking window proxies,
        // and this check covers only that single-origin case, protecting
        // against simple typo self-includes but nothing more elaborate.
        let mut ancestor = window.GetParent();
        while let Some(a) = ancestor {
            if let Some(ancestor_url) = a.document().map(|d| d.url()) {
                if ancestor_url.scheme() == url.scheme() &&
                    ancestor_url.username() == url.username() &&
                    ancestor_url.password() == url.password() &&
                    ancestor_url.host() == url.host() &&
                    ancestor_url.port() == url.port() &&
                    ancestor_url.path() == url.path() &&
                    ancestor_url.query() == url.query()
                {
                    return;
                }
            }
            ancestor = a.parent().map(DomRoot::from_ref);
        }

        let (creator_pipeline_id, about_base_url) = if url.matches_about_blank() {
            (Some(window.pipeline_id()), Some(document.base_url()))
        } else {
            (None, document.about_base_url())
        };

        let propagate_encoding_to_child_document = url.origin().same_origin(window.origin());
        let mut load_data = LoadData::new(
            LoadOrigin::Script(document.origin().snapshot()),
            url,
            about_base_url,
            creator_pipeline_id,
            window.as_global_scope().get_referrer(),
            referrer_policy,
            Some(window.as_global_scope().is_secure_context()),
            Some(document.insecure_requests_policy()),
            document.has_trustworthy_ancestor_or_current_origin(),
            self.sandboxing_flag_set(),
        );
        load_data.destination = Destination::IFrame;
        load_data.policy_container = Some(window.as_global_scope().policy_container());
        if propagate_encoding_to_child_document {
            load_data.container_document_encoding = Some(document.encoding());
        }

        let pipeline_id = self.pipeline_id();
        // If the initial `about:blank` page is the current page, load with replacement enabled,
        // see https://html.spec.whatwg.org/multipage/#the-iframe-element:about:blank-3
        let is_about_blank =
            pipeline_id.is_some() && pipeline_id == self.about_blank_pipeline_id.get();

        let history_handling = if is_about_blank {
            NavigationHistoryBehavior::Replace
        } else {
            NavigationHistoryBehavior::Push
        };

        self.navigate_or_reload_child_browsing_context(load_data, history_handling, cx);
    }

    /// <https://html.spec.whatwg.org/multipage/#create-a-new-child-navigable>
    /// Synchronously create a new browsing context(This is not a navigation).
    /// The pipeline started here should remain unnoticeable to script, but this is not easy
    /// to refactor because it appears other features have come to rely on the current behavior.
    /// For now only the iframe load event steps are skipped in some cases for this initial document,
    /// and we still fire load and pageshow events as part of `maybe_queue_document_completion`.
    /// Also, some controversy spec-wise remains: <https://github.com/whatwg/html/issues/4965>
    fn create_nested_browsing_context(&self, cx: &mut js::context::JSContext) {
        let url = ServoUrl::parse("about:blank").unwrap();
        let document = self.owner_document();
        let window = self.owner_window();
        let pipeline_id = Some(window.pipeline_id());
        let mut load_data = LoadData::new(
            LoadOrigin::Script(document.origin().snapshot()),
            url,
            Some(document.base_url()),
            pipeline_id,
            window.as_global_scope().get_referrer(),
            document.get_referrer_policy(),
            Some(window.as_global_scope().is_secure_context()),
            Some(document.insecure_requests_policy()),
            document.has_trustworthy_ancestor_or_current_origin(),
            self.sandboxing_flag_set(),
        );
        load_data.destination = Destination::IFrame;
        load_data.policy_container = Some(window.as_global_scope().policy_container());

        let browsing_context_id = BrowsingContextId::new();
        let webview_id = window.window_proxy().webview_id();
        self.pipeline_id.set(None);
        self.pending_pipeline_id.set(None);
        self.webview_id.set(Some(webview_id));
        self.browsing_context_id.set(Some(browsing_context_id));
        self.start_new_pipeline(
            load_data,
            PipelineType::InitialAboutBlank,
            NavigationHistoryBehavior::Push,
            cx,
        );
    }

    fn destroy_nested_browsing_context(&self) {
        self.pipeline_id.set(None);
        self.pending_pipeline_id.set(None);
        self.about_blank_pipeline_id.set(None);
        self.webview_id.set(None);
        if let Some(browsing_context_id) = self.browsing_context_id.take() {
            self.script_window_proxies.remove(browsing_context_id)
        }
    }

    pub(crate) fn update_pipeline_id(
        &self,
        new_pipeline_id: PipelineId,
        reason: UpdatePipelineIdReason,
        cx: &mut js::context::JSContext,
    ) {
        // For all updates except the one for the initial blank document,
        // we need to set the flag back to false because the navigation is complete,
        // because the goal is to, when a navigation is pending, to skip the async load
        // steps of the initial blank document.
        if !self.is_initial_blank_document() {
            self.pending_navigation.set(false);
        }
        if self.pending_pipeline_id.get() != Some(new_pipeline_id) &&
            reason == UpdatePipelineIdReason::Navigation
        {
            return;
        }

        self.pipeline_id.set(Some(new_pipeline_id));

        // Only terminate the load blocker if the pipeline id was updated due to a traversal.
        // The load blocker will be terminated for a navigation in iframe_load_event_steps.
        if reason == UpdatePipelineIdReason::Traversal {
            let blocker = &self.load_blocker;
            LoadBlocker::terminate(blocker, cx);
        }

        self.upcast::<Node>().dirty(NodeDamage::Other);
    }

    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLIFrameElement {
        HTMLIFrameElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            browsing_context_id: Cell::new(None),
            webview_id: Cell::new(None),
            pipeline_id: Cell::new(None),
            pending_pipeline_id: Cell::new(None),
            about_blank_pipeline_id: Cell::new(None),
            sandbox: Default::default(),
            sandboxing_flag_set: Cell::new(None),
            load_blocker: DomRefCell::new(None),
            throttled: Cell::new(false),
            script_window_proxies: ScriptThread::window_proxies(),
            current_navigation_was_lazy_loaded: Default::default(),
            lazy_load_resumption_steps: Default::default(),
            pending_navigation: Default::default(),
            already_fired_synchronous_load_event: Default::default(),
        }
    }

    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLIFrameElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLIFrameElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        )
    }

    #[inline]
    pub(crate) fn pipeline_id(&self) -> Option<PipelineId> {
        self.pipeline_id.get()
    }

    #[inline]
    pub(crate) fn browsing_context_id(&self) -> Option<BrowsingContextId> {
        self.browsing_context_id.get()
    }

    #[inline]
    pub(crate) fn webview_id(&self) -> Option<WebViewId> {
        self.webview_id.get()
    }

    #[inline]
    pub(crate) fn sandboxing_flag_set(&self) -> SandboxingFlagSet {
        self.sandboxing_flag_set
            .get()
            .unwrap_or_else(SandboxingFlagSet::empty)
    }

    pub(crate) fn set_throttled(&self, throttled: bool) {
        if self.throttled.get() != throttled {
            self.throttled.set(throttled);
        }
    }

    /// Note a pending navigation.
    /// This is used to ignore the async load event steps for
    /// the initial blank document if those haven't run yet.
    pub(crate) fn note_pending_navigation(&self) {
        self.pending_navigation.set(true);
    }

    /// <https://html.spec.whatwg.org/multipage/#iframe-load-event-steps>
    pub(crate) fn iframe_load_event_steps(
        &self,
        loaded_pipeline: PipelineId,
        cx: &mut js::context::JSContext,
    ) {
        // TODO(#9592): assert that the load blocker is present at all times when we
        //              can guarantee that it's created for the case of iframe.reload().
        if Some(loaded_pipeline) != self.pending_pipeline_id.get() {
            return;
        }

        // TODO 1. Assert: element's content navigable is not null.

        // TODO 2-4 Mark resource timing.

        // TODO 5 Set childDocument's iframe load in progress flag.

        // Note: in the spec, these steps are either run synchronously as part of
        // "If url matches about:blank and initialInsertion is true, then:"
        // in `process the iframe attributes`,
        // or asynchronously when navigation completes.
        //
        // In our current implementation,
        // we arrive here always asynchronously in the following two cases:
        // 1. as part of loading the initial blank document
        //    created in `create_nested_browsing_context`
        // 2. optionally, as part of loading a second document created as
        //    as part of the first processing of the iframe attributes.
        //
        // To preserve the logic of the spec--firing the load event once--in the context of
        // our current implementation, we must not fire the load event
        // for the initial blank document if we know that a navigation is ongoing,
        // which can be deducted from `pending_navigation` or the presence of an src.
        //
        // Additionally, to prevent a race condition with navigations,
        // in all cases, skip the load event if there is a pending navigation.
        // See #40348
        //
        // TODO: run these step synchronously as part of processing the iframe attributes.
        let should_fire_event = if self.is_initial_blank_document() {
            // If this is the initial blank doc:
            // do not fire if there is a pending navigation,
            // or if the iframe has an src.
            !self.pending_navigation.get() &&
                !self.upcast::<Element>().has_attribute(&local_name!("src"))
        } else {
            // If this is not the initial blank doc:
            // do not fire if there is a pending navigation.
            !self.pending_navigation.get()
        };
        // If we already fired a synchronous load event, we shouldn't fire another
        // one in this method.
        let should_fire_event =
            !self.already_fired_synchronous_load_event.replace(false) && should_fire_event;
        if should_fire_event {
            // Step 6. Fire an event named load at element.
            self.upcast::<EventTarget>()
                .fire_event(atom!("load"), CanGc::from_cx(cx));
        }

        let blocker = &self.load_blocker;
        LoadBlocker::terminate(blocker, cx);

        // TODO Step 7 - unset child document `mute iframe load` flag
    }

    /// Parse the `sandbox` attribute value given the [`Attr`]. This sets the `sandboxing_flag_set`
    /// property or clears it is the value isn't specified. Notably, an unspecified sandboxing
    /// attribute (no sandboxing) is different from an empty one (full sandboxing).
    fn parse_sandbox_attribute(&self) {
        let attribute = self
            .upcast::<Element>()
            .get_attribute(&ns!(), &local_name!("sandbox"));
        self.sandboxing_flag_set
            .set(attribute.map(|attribute_value| {
                let tokens: Vec<_> = attribute_value
                    .value()
                    .as_tokens()
                    .iter()
                    .map(|atom| atom.to_string().to_ascii_lowercase())
                    .collect();
                parse_a_sandboxing_directive(&tokens)
            }));
    }

    /// Step 4.2. of <https://html.spec.whatwg.org/multipage/#destroy-a-document-and-its-descendants>
    pub(crate) fn destroy_document_and_its_descendants(&self, cx: &mut js::context::JSContext) {
        let Some(pipeline_id) = self.pipeline_id.get() else {
            return;
        };
        // Step 4.2. Destroy a document and its descendants given childNavigable's active document and incrementDestroyed.
        if let Some(exited_document) = ScriptThread::find_document(pipeline_id) {
            exited_document.destroy_document_and_its_descendants(cx);
        }
        self.destroy_nested_browsing_context();
    }

    /// <https://html.spec.whatwg.org/multipage/#destroy-a-child-navigable>
    fn destroy_child_navigable(&self, cx: &mut js::context::JSContext) {
        let blocker = &self.load_blocker;
        LoadBlocker::terminate(blocker, cx);

        // Step 1. Let navigable be container's content navigable.
        let Some(browsing_context_id) = self.browsing_context_id() else {
            // Step 2. If navigable is null, then return.
            return;
        };
        // Store now so that we can destroy the context and delete the
        // document later
        let pipeline_id = self.pipeline_id.get();

        // Step 3. Set container's content navigable to null.
        //
        // Resetting the pipeline_id to None is required here so that
        // if this iframe is subsequently re-added to the document
        // the load doesn't think that it's a navigation, but instead
        // a new iframe. Without this, the constellation gets very
        // confused.
        self.destroy_nested_browsing_context();

        // Step 4. Inform the navigation API about child navigable destruction given navigable.
        // TODO

        // Step 5. Destroy a document and its descendants given navigable's active document.
        let (sender, receiver) =
            ProfiledIpc::channel(self.global().time_profiler_chan().clone()).unwrap();
        let msg = ScriptToConstellationMessage::RemoveIFrame(browsing_context_id, sender);
        self.owner_window()
            .as_global_scope()
            .script_to_constellation_chan()
            .send(msg)
            .unwrap();
        let _exited_pipeline_ids = receiver.recv().unwrap();
        let Some(pipeline_id) = pipeline_id else {
            return;
        };
        if let Some(exited_document) = ScriptThread::find_document(pipeline_id) {
            exited_document.destroy_document_and_its_descendants(cx);
        }

        // Step 6. Let parentDocState be container's node navigable's active session history entry's document state.
        // TODO

        // Step 7. Remove the nested history from parentDocState's nested histories whose id equals navigable's id.
        // TODO

        // Step 8. Let traversable be container's node navigable's traversable navigable.
        // TODO

        // Step 9. Append the following session history traversal steps to traversable:
        // TODO

        // Step 10. Invoke WebDriver BiDi navigable destroyed with navigable.
        // TODO
    }
}

pub(crate) trait HTMLIFrameElementLayoutMethods {
    fn pipeline_id(self) -> Option<PipelineId>;
    fn browsing_context_id(self) -> Option<BrowsingContextId>;
    fn get_width(self) -> LengthOrPercentageOrAuto;
    fn get_height(self) -> LengthOrPercentageOrAuto;
}

impl HTMLIFrameElementLayoutMethods for LayoutDom<'_, HTMLIFrameElement> {
    #[inline]
    fn pipeline_id(self) -> Option<PipelineId> {
        (self.unsafe_get()).pipeline_id.get()
    }

    #[inline]
    fn browsing_context_id(self) -> Option<BrowsingContextId> {
        (self.unsafe_get()).browsing_context_id.get()
    }

    fn get_width(self) -> LengthOrPercentageOrAuto {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("width"))
            .map(AttrValue::as_dimension)
            .cloned()
            .unwrap_or(LengthOrPercentageOrAuto::Auto)
    }

    fn get_height(self) -> LengthOrPercentageOrAuto {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("height"))
            .map(AttrValue::as_dimension)
            .cloned()
            .unwrap_or(LengthOrPercentageOrAuto::Auto)
    }
}

impl HTMLIFrameElementMethods<crate::DomTypeHolder> for HTMLIFrameElement {
    // https://html.spec.whatwg.org/multipage/#dom-iframe-src
    make_url_getter!(Src, "src");

    // https://html.spec.whatwg.org/multipage/#dom-iframe-src
    make_url_setter!(SetSrc, "src");

    /// <https://html.spec.whatwg.org/multipage/#dom-iframe-srcdoc>
    fn Srcdoc(&self) -> TrustedHTMLOrString {
        let element = self.upcast::<Element>();
        element.get_trusted_html_attribute(&local_name!("srcdoc"))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-iframe-srcdoc>
    fn SetSrcdoc(&self, value: TrustedHTMLOrString, can_gc: CanGc) -> Fallible<()> {
        // Step 1: Let compliantString be the result of invoking the
        // Get Trusted Type compliant string algorithm with TrustedHTML,
        // this's relevant global object, the given value, "HTMLIFrameElement srcdoc", and "script".
        let element = self.upcast::<Element>();
        let value = TrustedHTML::get_trusted_script_compliant_string(
            &element.owner_global(),
            value,
            "HTMLIFrameElement srcdoc",
            can_gc,
        )?;
        // Step 2: Set an attribute value given this, srcdoc's local name, and compliantString.
        element.set_attribute(
            &local_name!("srcdoc"),
            AttrValue::String(value.str().to_owned()),
            can_gc,
        );
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-iframe-sandbox>
    ///
    /// The supported tokens for sandbox's DOMTokenList are the allowed values defined in the
    /// sandbox attribute and supported by the user agent. These range of possible values is
    /// defined here: <https://html.spec.whatwg.org/multipage/#attr-iframe-sandbox>
    fn Sandbox(&self, can_gc: CanGc) -> DomRoot<DOMTokenList> {
        self.sandbox.or_init(|| {
            DOMTokenList::new(
                self.upcast::<Element>(),
                &local_name!("sandbox"),
                Some(vec![
                    Atom::from("allow-downloads"),
                    Atom::from("allow-forms"),
                    Atom::from("allow-modals"),
                    Atom::from("allow-orientation-lock"),
                    Atom::from("allow-pointer-lock"),
                    Atom::from("allow-popups"),
                    Atom::from("allow-popups-to-escape-sandbox"),
                    Atom::from("allow-presentation"),
                    Atom::from("allow-same-origin"),
                    Atom::from("allow-scripts"),
                    Atom::from("allow-top-navigation"),
                    Atom::from("allow-top-navigation-by-user-activation"),
                    Atom::from("allow-top-navigation-to-custom-protocols"),
                ]),
                can_gc,
            )
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-iframe-contentwindow>
    fn GetContentWindow(&self) -> Option<DomRoot<WindowProxy>> {
        self.browsing_context_id
            .get()
            .and_then(|id| self.script_window_proxies.find_window_proxy(id))
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-bcc-content-document>
    fn GetContentDocument(&self) -> Option<DomRoot<Document>> {
        // Step 1. If container's content navigable is null, then return null.
        let pipeline_id = self.pipeline_id.get()?;

        // Step 2. Let document be container's content navigable's active document.
        // Note that this lookup will fail if the document is dissimilar-origin,
        // so we should return None in that case.
        let document = ScriptThread::find_document(pipeline_id)?;
        // Step 3. If document's origin and container's node document's origin are not same origin-domain, then return null.
        if !self
            .owner_document()
            .origin()
            .same_origin_domain(document.origin())
        {
            return None;
        }
        // Step 4. Return document.
        Some(document)
    }

    /// <https://html.spec.whatwg.org/multipage/#attr-iframe-referrerpolicy>
    fn ReferrerPolicy(&self) -> DOMString {
        reflect_referrer_policy_attribute(self.upcast::<Element>())
    }

    // https://html.spec.whatwg.org/multipage/#attr-iframe-referrerpolicy
    make_setter!(SetReferrerPolicy, "referrerpolicy");

    // https://html.spec.whatwg.org/multipage/#attr-iframe-allowfullscreen
    make_bool_getter!(AllowFullscreen, "allowfullscreen");
    // https://html.spec.whatwg.org/multipage/#attr-iframe-allowfullscreen
    make_bool_setter!(SetAllowFullscreen, "allowfullscreen");

    // <https://html.spec.whatwg.org/multipage/#dom-dim-width>
    make_getter!(Width, "width");
    // <https://html.spec.whatwg.org/multipage/#dom-dim-width>
    make_dimension_setter!(SetWidth, "width");

    // <https://html.spec.whatwg.org/multipage/#dom-dim-height>
    make_getter!(Height, "height");
    // <https://html.spec.whatwg.org/multipage/#dom-dim-height>
    make_dimension_setter!(SetHeight, "height");

    // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:attr-iframe-frameborder
    make_getter!(FrameBorder, "frameborder");
    // https://html.spec.whatwg.org/multipage/#other-elements,-attributes-and-apis:attr-iframe-frameborder
    make_setter!(SetFrameBorder, "frameborder");

    // https://html.spec.whatwg.org/multipage/#dom-iframe-name
    // A child browsing context checks the name of its iframe only at the time
    // it is created; subsequent name sets have no special effect.
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-iframe-name
    // This is specified as reflecting the name content attribute of the
    // element, not the name of the child browsing context.
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#attr-iframe-loading
    // > The loading attribute is a lazy loading attribute. Its purpose is to indicate the policy for loading iframe elements that are outside the viewport.
    make_enumerated_getter!(
        Loading,
        "loading",
        "lazy" | "eager",
        // https://html.spec.whatwg.org/multipage/#lazy-loading-attribute
        // > The attribute's missing value default and invalid value default are both the Eager state.
        missing => "eager",
        invalid => "eager"
    );

    // https://html.spec.whatwg.org/multipage/#attr-iframe-loading
    make_setter!(SetLoading, "loading");
}

impl VirtualMethods for HTMLIFrameElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    #[expect(unsafe_code)]
    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, _can_gc: CanGc) {
        // TODO: https://github.com/servo/servo/issues/42812
        let mut cx = unsafe { temp_cx() };
        let cx = &mut cx;
        self.super_type()
            .unwrap()
            .attribute_mutated(attr, mutation, CanGc::from_cx(cx));
        match *attr.local_name() {
            // From <https://html.spec.whatwg.org/multipage/#attr-iframe-sandbox>:
            //
            // > When an iframe element's sandbox attribute is set or changed while
            // > it has a non-null content navigable, the user agent must parse the
            // > sandboxing directive given the attribute's value and the iframe
            // > element's iframe sandboxing flag set.
            //
            // > When an iframe element's sandbox attribute is removed while it has
            // > a non-null content navigable, the user agent must empty the iframe
            // > element's iframe sandboxing flag set.
            local_name!("sandbox") if self.browsing_context_id.get().is_some() => {
                self.parse_sandbox_attribute();
            },
            local_name!("srcdoc") => {
                // https://html.spec.whatwg.org/multipage/#the-iframe-element:the-iframe-element-9
                // "Whenever an iframe element with a non-null nested browsing context has its
                // srcdoc attribute set, changed, or removed, the user agent must process the
                // iframe attributes."
                // but we can't check that directly, since the child browsing context
                // may be in a different script thread. Instead, we check to see if the parent
                // is in a document tree and has a browsing context, which is what causes
                // the child browsing context to be created.

                // trigger the processing of iframe attributes whenever "srcdoc" attribute is set, changed or removed
                if self.upcast::<Node>().is_connected_with_browsing_context() {
                    debug!("iframe srcdoc modified while in browsing context.");
                    self.process_the_iframe_attributes(ProcessingMode::NotFirstTime, cx);
                }
            },
            local_name!("src") => {
                // https://html.spec.whatwg.org/multipage/#the-iframe-element
                // "Similarly, whenever an iframe element with a non-null nested browsing context
                // but with no srcdoc attribute specified has its src attribute set, changed, or removed,
                // the user agent must process the iframe attributes,"
                // but we can't check that directly, since the child browsing context
                // may be in a different script thread. Instead, we check to see if the parent
                // is in a document tree and has a browsing context, which is what causes
                // the child browsing context to be created.
                if self.upcast::<Node>().is_connected_with_browsing_context() {
                    debug!("iframe src set while in browsing context.");
                    self.process_the_iframe_attributes(ProcessingMode::NotFirstTime, cx);
                }
            },
            local_name!("loading") => {
                // https://html.spec.whatwg.org/multipage/#attr-iframe-loading
                // > When the loading attribute's state is changed to the Eager state, the user agent must run these steps:
                if !mutation.is_removal() && &**attr.value() == "lazy" {
                    return;
                }

                // Step 1. Let resumptionSteps be the iframe element's lazy load resumption steps.
                // Step 3. Set the iframe's lazy load resumption steps to null.
                let previous_resumption_steps = self
                    .lazy_load_resumption_steps
                    .replace(LazyLoadResumptionSteps::None);
                match previous_resumption_steps {
                    // Step 2. If resumptionSteps is null, then return.
                    LazyLoadResumptionSteps::None => (),
                    LazyLoadResumptionSteps::SrcDoc => {
                        // Step 4. Invoke resumptionSteps.
                        self.navigate_to_the_srcdoc_resource(cx);
                    },
                }
            },
            _ => {},
        }
    }

    fn attribute_affects_presentational_hints(&self, attr: &Attr) -> bool {
        match attr.local_name() {
            &local_name!("width") | &local_name!("height") => true,
            _ => self
                .super_type()
                .unwrap()
                .attribute_affects_presentational_hints(attr),
        }
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match *name {
            local_name!("sandbox") => AttrValue::from_serialized_tokenlist(value.into()),
            local_name!("width") => AttrValue::from_dimension(value.into()),
            local_name!("height") => AttrValue::from_dimension(value.into()),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#the-iframe-element:html-element-post-connection-steps>
    fn post_connection_steps(&self, cx: &mut JSContext) {
        if let Some(s) = self.super_type() {
            s.post_connection_steps(cx);
        }

        // This isn't mentioned any longer in the specification, but still seems important. This is
        // likely due to the fact that we have deviated a great deal with it comes to navigables
        // and browsing contexts.
        if !self.upcast::<Node>().is_connected_with_browsing_context() {
            return;
        }

        debug!("<iframe> running post connection steps");

        // Step 1. Create a new child navigable for insertedNode.
        self.create_nested_browsing_context(cx);

        // Step 2: If insertedNode has a sandbox attribute, then parse the sandboxing directive
        // given the attribute's value and insertedNode's iframe sandboxing flag set.
        self.parse_sandbox_attribute();

        // Step 3. Process the iframe attributes for insertedNode, with initialInsertion set to true.
        self.process_the_iframe_attributes(ProcessingMode::FirstTime, cx);
    }

    fn bind_to_tree(&self, context: &BindContext, can_gc: CanGc) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context, can_gc);
        }
        self.owner_document().invalidate_iframes_collection();
    }

    /// <https://html.spec.whatwg.org/multipage/#the-iframe-element:html-element-removing-steps>
    #[expect(unsafe_code)]
    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        self.super_type().unwrap().unbind_from_tree(context, can_gc);

        // TODO: https://github.com/servo/servo/issues/42837
        let mut cx = unsafe { temp_cx() };

        // The iframe HTML element removing steps, given removedNode, are to destroy a child navigable given removedNode
        self.destroy_child_navigable(&mut cx);

        self.owner_document().invalidate_iframes_collection();
    }
}

/// IframeContext is a wrapper around [`HTMLIFrameElement`] that implements the [`ResourceTimingListener`] trait.
/// Note: this implementation of `resource_timing_global` returns the parent document's global scope, not the iframe's global scope.
pub(crate) struct IframeContext<'a> {
    // The iframe element that this context is associated with.
    element: &'a HTMLIFrameElement,
    // The URL of the iframe document.
    url: ServoUrl,
}

impl<'a> IframeContext<'a> {
    /// Creates a new IframeContext from a reference to an HTMLIFrameElement.
    pub fn new(element: &'a HTMLIFrameElement) -> Self {
        Self {
            element,
            url: element
                .shared_attribute_processing_steps_for_iframe_and_frame_elements()
                .expect("Must always have a URL when navigating"),
        }
    }
}

impl<'a> ResourceTimingListener for IframeContext<'a> {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        (
            InitiatorType::LocalName("iframe".to_string()),
            self.url.clone(),
        )
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.element.upcast::<Node>().owner_doc().global()
    }
}
