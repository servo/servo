/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use base::id::{BrowsingContextId, PipelineId, TopLevelBrowsingContextId};
use bitflags::bitflags;
use dom_struct::dom_struct;
use html5ever::{local_name, namespace_url, ns, LocalName, Prefix};
use js::rust::HandleObject;
use net_traits::ReferrerPolicy;
use profile_traits::ipc as ProfiledIpc;
use script_traits::IFrameSandboxState::{IFrameSandboxed, IFrameUnsandboxed};
use script_traits::{
    IFrameLoadInfo, IFrameLoadInfoWithData, JsEvalResult, LoadData, LoadOrigin,
    NavigationHistoryBehavior, NewLayoutInfo, ScriptMsg, UpdatePipelineIdReason, WindowSizeData,
};
use servo_atoms::Atom;
use servo_url::ServoUrl;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};

use crate::document_loader::{LoadBlocker, LoadType};
use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLIFrameElementBinding::HTMLIFrameElementMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::Window_Binding::WindowMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{DomRoot, LayoutDom, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::document::{determine_policy_for_token, Document};
use crate::dom::domtokenlist::DOMTokenList;
use crate::dom::element::{
    reflect_referrer_policy_attribute, AttributeMutation, Element, LayoutElementHelpers,
};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{Node, NodeDamage, NodeTraits, UnbindContext};
use crate::dom::virtualmethods::VirtualMethods;
use crate::dom::windowproxy::WindowProxy;
use crate::script_runtime::CanGc;
use crate::script_thread::ScriptThread;

#[derive(Clone, Copy, JSTraceable, MallocSizeOf)]
struct SandboxAllowance(u8);

bitflags! {
    impl SandboxAllowance: u8 {
        const ALLOW_NOTHING = 0x00;
        const ALLOW_SAME_ORIGIN = 0x01;
        const ALLOW_TOP_NAVIGATION = 0x02;
        const ALLOW_FORMS = 0x04;
        const ALLOW_SCRIPTS = 0x08;
        const ALLOW_POINTER_LOCK = 0x10;
        const ALLOW_POPUPS = 0x20;
    }
}

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

#[dom_struct]
pub(crate) struct HTMLIFrameElement {
    htmlelement: HTMLElement,
    #[no_trace]
    top_level_browsing_context_id: Cell<Option<TopLevelBrowsingContextId>>,
    #[no_trace]
    browsing_context_id: Cell<Option<BrowsingContextId>>,
    #[no_trace]
    pipeline_id: Cell<Option<PipelineId>>,
    #[no_trace]
    pending_pipeline_id: Cell<Option<PipelineId>>,
    #[no_trace]
    about_blank_pipeline_id: Cell<Option<PipelineId>>,
    sandbox: MutNullableDom<DOMTokenList>,
    sandbox_allowance: Cell<Option<SandboxAllowance>>,
    load_blocker: DomRefCell<Option<LoadBlocker>>,
    throttled: Cell<bool>,
}

impl HTMLIFrameElement {
    pub(crate) fn is_sandboxed(&self) -> bool {
        self.sandbox_allowance.get().is_some()
    }

    /// <https://html.spec.whatwg.org/multipage/#otherwise-steps-for-iframe-or-frame-elements>,
    /// step 1.
    fn get_url(&self) -> ServoUrl {
        let element = self.upcast::<Element>();
        element
            .get_attribute(&ns!(), &local_name!("src"))
            .and_then(|src| {
                let url = src.value();
                if url.is_empty() {
                    None
                } else {
                    self.owner_document().base_url().join(&url).ok()
                }
            })
            .unwrap_or_else(|| ServoUrl::parse("about:blank").unwrap())
    }

    pub(crate) fn navigate_or_reload_child_browsing_context(
        &self,
        load_data: LoadData,
        history_handling: NavigationHistoryBehavior,
        can_gc: CanGc,
    ) {
        self.start_new_pipeline(
            load_data,
            PipelineType::Navigation,
            history_handling,
            can_gc,
        );
    }

    fn start_new_pipeline(
        &self,
        mut load_data: LoadData,
        pipeline_type: PipelineType,
        history_handling: NavigationHistoryBehavior,
        can_gc: CanGc,
    ) {
        let sandboxed = if self.is_sandboxed() {
            IFrameSandboxed
        } else {
            IFrameUnsandboxed
        };

        let browsing_context_id = match self.browsing_context_id() {
            None => return warn!("Attempted to start a new pipeline on an unattached iframe."),
            Some(id) => id,
        };

        let top_level_browsing_context_id = match self.top_level_browsing_context_id() {
            None => return warn!("Attempted to start a new pipeline on an unattached iframe."),
            Some(id) => id,
        };

        let document = self.owner_document();

        {
            let load_blocker = &self.load_blocker;
            // Any oustanding load is finished from the point of view of the blocked
            // document; the new navigation will continue blocking it.
            LoadBlocker::terminate(load_blocker, can_gc);
        }

        if load_data.url.scheme() == "javascript" {
            let window_proxy = self.GetContentWindow();
            if let Some(window_proxy) = window_proxy {
                // Important re security. See https://github.com/servo/servo/issues/23373
                // TODO: check according to https://w3c.github.io/webappsec-csp/#should-block-navigation-request
                if ScriptThread::check_load_origin(&load_data.load_origin, &document.url().origin())
                {
                    ScriptThread::eval_js_url(&window_proxy.global(), &mut load_data, can_gc);
                }
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
            top_level_browsing_context_id,
            new_pipeline_id,
            is_private: false, // FIXME
            inherited_secure_context: load_data.inherited_secure_context,
            history_handling,
        };

        let window_size = WindowSizeData {
            initial_viewport: window
                .get_iframe_size_if_known(browsing_context_id, can_gc)
                .unwrap_or_default(),
            device_pixel_ratio: window.device_pixel_ratio(),
        };

        match pipeline_type {
            PipelineType::InitialAboutBlank => {
                self.about_blank_pipeline_id.set(Some(new_pipeline_id));

                let load_info = IFrameLoadInfoWithData {
                    info: load_info,
                    load_data: load_data.clone(),
                    old_pipeline_id,
                    sandbox: sandboxed,
                    window_size,
                };
                window
                    .as_global_scope()
                    .script_to_constellation_chan()
                    .send(ScriptMsg::ScriptNewIFrame(load_info))
                    .unwrap();

                let new_layout_info = NewLayoutInfo {
                    parent_info: Some(window.pipeline_id()),
                    new_pipeline_id,
                    browsing_context_id,
                    top_level_browsing_context_id,
                    opener: None,
                    load_data,
                    window_size,
                };

                self.pipeline_id.set(Some(new_pipeline_id));
                ScriptThread::process_attach_layout(new_layout_info, document.origin().clone());
            },
            PipelineType::Navigation => {
                let load_info = IFrameLoadInfoWithData {
                    info: load_info,
                    load_data,
                    old_pipeline_id,
                    sandbox: sandboxed,
                    window_size,
                };
                window
                    .as_global_scope()
                    .script_to_constellation_chan()
                    .send(ScriptMsg::ScriptLoadedURLInIFrame(load_info))
                    .unwrap();
            },
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#process-the-iframe-attributes>
    fn process_the_iframe_attributes(&self, mode: ProcessingMode, can_gc: CanGc) {
        // > 1. If `element`'s `srcdoc` attribute is specified, then:
        if self
            .upcast::<Element>()
            .has_attribute(&local_name!("srcdoc"))
        {
            let url = ServoUrl::parse("about:srcdoc").unwrap();
            let document = self.owner_document();
            let window = self.owner_window();
            let pipeline_id = Some(window.pipeline_id());
            let mut load_data = LoadData::new(
                LoadOrigin::Script(document.origin().immutable().clone()),
                url,
                pipeline_id,
                window.as_global_scope().get_referrer(),
                document.get_referrer_policy(),
                Some(window.as_global_scope().is_secure_context()),
                Some(document.insecure_requests_policy()),
            );
            let element = self.upcast::<Element>();
            load_data.srcdoc = String::from(element.get_string_attribute(&local_name!("srcdoc")));
            self.navigate_or_reload_child_browsing_context(
                load_data,
                NavigationHistoryBehavior::Push,
                can_gc,
            );
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
                    self.upcast::<Element>()
                        .get_name()
                        .map_or(DOMString::from(""), |n| DOMString::from(&*n)),
                );
            }
        }

        if mode == ProcessingMode::FirstTime &&
            !self.upcast::<Element>().has_attribute(&local_name!("src"))
        {
            return;
        }

        // > 2. Otherwise, if `element` has a `src` attribute specified, or
        // >    `initialInsertion` is false, then run the shared attribute
        // >    processing steps for `iframe` and `frame` elements given
        // >    `element`.
        let url = self.get_url();

        // Step 2.4: Let referrerPolicy be the current state of element's referrerpolicy content
        // attribute.
        let document = self.owner_document();
        let referrer_policy_token = self.ReferrerPolicy();

        // Note: despite not being explicitly stated in the spec steps, this falls back to
        // document's referrer policy here because it satisfies the expectations that when unset,
        // the iframe should inherit the referrer policy of its parent
        let referrer_policy = match determine_policy_for_token(referrer_policy_token.str()) {
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

        let creator_pipeline_id = if url.as_str() == "about:blank" {
            Some(window.pipeline_id())
        } else {
            None
        };

        let load_data = LoadData::new(
            LoadOrigin::Script(document.origin().immutable().clone()),
            url,
            creator_pipeline_id,
            window.as_global_scope().get_referrer(),
            referrer_policy,
            Some(window.as_global_scope().is_secure_context()),
            Some(document.insecure_requests_policy()),
        );

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

        self.navigate_or_reload_child_browsing_context(load_data, history_handling, can_gc);
    }

    fn create_nested_browsing_context(&self, can_gc: CanGc) {
        // Synchronously create a new browsing context, which will present
        // `about:blank`. (This is not a navigation.)
        //
        // The pipeline started here will synchronously "completely finish
        // loading", which will then asynchronously call
        // `iframe_load_event_steps`.
        //
        // The precise event timing differs between implementations and
        // remains controversial:
        //
        //  - [Unclear "iframe load event steps" for initial load of about:blank
        //    in an iframe #490](https://github.com/whatwg/html/issues/490)
        //  - [load event handling for iframes with no src may not be web
        //    compatible #4965](https://github.com/whatwg/html/issues/4965)
        //
        let url = ServoUrl::parse("about:blank").unwrap();
        let document = self.owner_document();
        let window = self.owner_window();
        let pipeline_id = Some(window.pipeline_id());
        let load_data = LoadData::new(
            LoadOrigin::Script(document.origin().immutable().clone()),
            url,
            pipeline_id,
            window.as_global_scope().get_referrer(),
            document.get_referrer_policy(),
            Some(window.as_global_scope().is_secure_context()),
            Some(document.insecure_requests_policy()),
        );
        let browsing_context_id = BrowsingContextId::new();
        let top_level_browsing_context_id = window.window_proxy().top_level_browsing_context_id();
        self.pipeline_id.set(None);
        self.pending_pipeline_id.set(None);
        self.top_level_browsing_context_id
            .set(Some(top_level_browsing_context_id));
        self.browsing_context_id.set(Some(browsing_context_id));
        self.start_new_pipeline(
            load_data,
            PipelineType::InitialAboutBlank,
            NavigationHistoryBehavior::Push,
            can_gc,
        );
    }

    fn destroy_nested_browsing_context(&self) {
        self.pipeline_id.set(None);
        self.pending_pipeline_id.set(None);
        self.about_blank_pipeline_id.set(None);
        self.top_level_browsing_context_id.set(None);
        self.browsing_context_id.set(None);
    }

    pub(crate) fn update_pipeline_id(
        &self,
        new_pipeline_id: PipelineId,
        reason: UpdatePipelineIdReason,
        can_gc: CanGc,
    ) {
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
            LoadBlocker::terminate(blocker, can_gc);
        }

        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }

    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLIFrameElement {
        HTMLIFrameElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            browsing_context_id: Cell::new(None),
            top_level_browsing_context_id: Cell::new(None),
            pipeline_id: Cell::new(None),
            pending_pipeline_id: Cell::new(None),
            about_blank_pipeline_id: Cell::new(None),
            sandbox: Default::default(),
            sandbox_allowance: Cell::new(None),
            load_blocker: DomRefCell::new(None),
            throttled: Cell::new(false),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
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
    pub(crate) fn top_level_browsing_context_id(&self) -> Option<TopLevelBrowsingContextId> {
        self.top_level_browsing_context_id.get()
    }

    pub(crate) fn set_throttled(&self, throttled: bool) {
        if self.throttled.get() != throttled {
            self.throttled.set(throttled);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#iframe-load-event-steps> steps 1-4
    pub(crate) fn iframe_load_event_steps(&self, loaded_pipeline: PipelineId, can_gc: CanGc) {
        // TODO(#9592): assert that the load blocker is present at all times when we
        //              can guarantee that it's created for the case of iframe.reload().
        if Some(loaded_pipeline) != self.pending_pipeline_id.get() {
            return;
        }

        // TODO A cross-origin child document would not be easily accessible
        //      from this script thread. It's unclear how to implement
        //      steps 2, 3, and 5 efficiently in this case.
        // TODO Step 2 - check child document `mute iframe load` flag
        // TODO Step 3 - set child document  `mut iframe load` flag

        // Step 4
        self.upcast::<EventTarget>()
            .fire_event(atom!("load"), can_gc);

        let blocker = &self.load_blocker;
        LoadBlocker::terminate(blocker, can_gc);

        // TODO Step 5 - unset child document `mut iframe load` flag
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

    // https://html.spec.whatwg.org/multipage/#dom-iframe-srcdoc
    make_getter!(Srcdoc, "srcdoc");

    // https://html.spec.whatwg.org/multipage/#dom-iframe-srcdoc
    make_setter!(SetSrcdoc, "srcdoc");

    // https://html.spec.whatwg.org/multipage/#dom-iframe-sandbox
    fn Sandbox(&self) -> DomRoot<DOMTokenList> {
        self.sandbox.or_init(|| {
            DOMTokenList::new(
                self.upcast::<Element>(),
                &local_name!("sandbox"),
                Some(vec![
                    Atom::from("allow-same-origin"),
                    Atom::from("allow-forms"),
                    Atom::from("allow-pointer-lock"),
                    Atom::from("allow-popups"),
                    Atom::from("allow-scripts"),
                    Atom::from("allow-top-navigation"),
                ]),
            )
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-iframe-contentwindow
    fn GetContentWindow(&self) -> Option<DomRoot<WindowProxy>> {
        self.browsing_context_id
            .get()
            .and_then(ScriptThread::find_window_proxy)
    }

    // https://html.spec.whatwg.org/multipage/#dom-iframe-contentdocument
    // https://html.spec.whatwg.org/multipage/#concept-bcc-content-document
    fn GetContentDocument(&self) -> Option<DomRoot<Document>> {
        // Step 1.
        let pipeline_id = self.pipeline_id.get()?;

        // Step 2-3.
        // Note that this lookup will fail if the document is dissimilar-origin,
        // so we should return None in that case.
        let document = ScriptThread::find_document(pipeline_id)?;

        // Step 4.
        let current = GlobalScope::current()
            .expect("No current global object")
            .as_window()
            .Document();
        if !current.origin().same_origin_domain(document.origin()) {
            return None;
        }
        // Step 5.
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

    // https://html.spec.whatwg.org/multipage/#dom-dim-width
    make_getter!(Width, "width");
    // https://html.spec.whatwg.org/multipage/#dom-dim-width
    make_dimension_setter!(SetWidth, "width");

    // https://html.spec.whatwg.org/multipage/#dom-dim-height
    make_getter!(Height, "height");
    // https://html.spec.whatwg.org/multipage/#dom-dim-height
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
}

impl VirtualMethods for HTMLIFrameElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match *attr.local_name() {
            local_name!("sandbox") => {
                self.sandbox_allowance
                    .set(mutation.new_value(attr).map(|value| {
                        let mut modes = SandboxAllowance::ALLOW_NOTHING;
                        for token in value.as_tokens() {
                            modes |= match &*token.to_ascii_lowercase() {
                                "allow-same-origin" => SandboxAllowance::ALLOW_SAME_ORIGIN,
                                "allow-forms" => SandboxAllowance::ALLOW_FORMS,
                                "allow-pointer-lock" => SandboxAllowance::ALLOW_POINTER_LOCK,
                                "allow-popups" => SandboxAllowance::ALLOW_POPUPS,
                                "allow-scripts" => SandboxAllowance::ALLOW_SCRIPTS,
                                "allow-top-navigation" => SandboxAllowance::ALLOW_TOP_NAVIGATION,
                                _ => SandboxAllowance::ALLOW_NOTHING,
                            };
                        }
                        modes
                    }));
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
                    self.process_the_iframe_attributes(ProcessingMode::NotFirstTime, CanGc::note());
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
                    self.process_the_iframe_attributes(ProcessingMode::NotFirstTime, CanGc::note());
                }
            },
            _ => {},
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

    fn post_connection_steps(&self) {
        if let Some(s) = self.super_type() {
            s.post_connection_steps();
        }

        // https://html.spec.whatwg.org/multipage/#the-iframe-element
        // "When an iframe element is inserted into a document that has
        // a browsing context, the user agent must create a new
        // browsing context, set the element's nested browsing context
        // to the newly-created browsing context, and then process the
        // iframe attributes for the "first time"."
        if self.upcast::<Node>().is_connected_with_browsing_context() {
            debug!("iframe bound to browsing context.");
            self.create_nested_browsing_context(CanGc::note());
            self.process_the_iframe_attributes(ProcessingMode::FirstTime, CanGc::note());
        }
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);

        let blocker = &self.load_blocker;
        LoadBlocker::terminate(blocker, CanGc::note());

        // https://html.spec.whatwg.org/multipage/#a-browsing-context-is-discarded
        let window = self.owner_window();
        let (sender, receiver) =
            ProfiledIpc::channel(self.global().time_profiler_chan().clone()).unwrap();

        // Ask the constellation to remove the iframe, and tell us the
        // pipeline ids of the closed pipelines.
        let browsing_context_id = match self.browsing_context_id() {
            None => return warn!("Unbinding already unbound iframe."),
            Some(id) => id,
        };
        debug!("Unbinding frame {}.", browsing_context_id);

        let msg = ScriptMsg::RemoveIFrame(browsing_context_id, sender);
        window
            .as_global_scope()
            .script_to_constellation_chan()
            .send(msg)
            .unwrap();
        let exited_pipeline_ids = receiver.recv().unwrap();

        // The spec for discarding is synchronous,
        // so we need to discard the browsing contexts now, rather than
        // when the `PipelineExit` message arrives.
        for exited_pipeline_id in exited_pipeline_ids {
            // https://html.spec.whatwg.org/multipage/#a-browsing-context-is-discarded
            if let Some(exited_document) = ScriptThread::find_document(exited_pipeline_id) {
                debug!(
                    "Discarding browsing context for pipeline {}",
                    exited_pipeline_id
                );
                let exited_window = exited_document.window();
                exited_window.discard_browsing_context();
                for exited_iframe in exited_document.iframes().iter() {
                    debug!("Discarding nested browsing context");
                    exited_iframe.destroy_nested_browsing_context();
                }
            }
        }

        // Resetting the pipeline_id to None is required here so that
        // if this iframe is subsequently re-added to the document
        // the load doesn't think that it's a navigation, but instead
        // a new iframe. Without this, the constellation gets very
        // confused.
        self.destroy_nested_browsing_context();
    }
}
