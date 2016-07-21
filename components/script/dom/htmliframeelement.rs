/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::{LoadType, LoadBlocker};
use dom::attr::Attr;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BrowserElementBinding::BrowserElementErrorEventDetail;
use dom::bindings::codegen::Bindings::BrowserElementBinding::BrowserElementIconChangeEventDetail;
use dom::bindings::codegen::Bindings::BrowserElementBinding::BrowserElementLocationChangeEventDetail;
use dom::bindings::codegen::Bindings::BrowserElementBinding::BrowserElementOpenTabEventDetail;
use dom::bindings::codegen::Bindings::BrowserElementBinding::BrowserElementOpenWindowEventDetail;
use dom::bindings::codegen::Bindings::BrowserElementBinding::BrowserElementSecurityChangeDetail;
use dom::bindings::codegen::Bindings::BrowserElementBinding::BrowserElementVisibilityChangeEventDetail;
use dom::bindings::codegen::Bindings::BrowserElementBinding::BrowserShowModalPromptEventDetail;
use dom::bindings::codegen::Bindings::HTMLIFrameElementBinding;
use dom::bindings::codegen::Bindings::HTMLIFrameElementBinding::HTMLIFrameElementMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::conversions::ToJSValConvertible;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableHeap, Root, LayoutJS};
use dom::bindings::reflector::Reflectable;
use dom::bindings::str::DOMString;
use dom::browsingcontext::BrowsingContext;
use dom::customevent::CustomEvent;
use dom::document::Document;
use dom::domtokenlist::DOMTokenList;
use dom::element::{AttributeMutation, Element, RawLayoutElementHelpers};
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeDamage, UnbindContext, window_from_node, document_from_node};
use dom::urlhelper::UrlHelper;
use dom::virtualmethods::VirtualMethods;
use dom::window::{ReflowReason, Window};
use ipc_channel::ipc;
use js::jsapi::{JSAutoCompartment, JSContext, MutableHandleValue};
use js::jsval::{UndefinedValue, NullValue};
use msg::constellation_msg::{FrameType, LoadData, NavigationDirection, PipelineId, SubpageId};
use net_traits::response::HttpsState;
use script_layout_interface::message::ReflowQueryType;
use script_traits::IFrameSandboxState::{IFrameSandboxed, IFrameUnsandboxed};
use script_traits::{IFrameLoadInfo, MozBrowserEvent, ScriptMsg as ConstellationMsg};
use std::cell::Cell;
use string_cache::Atom;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};
use style::context::ReflowGoal;
use url::Url;
use util::prefs::PREFS;
use util::servo_version;

bitflags! {
    #[derive(JSTraceable, HeapSizeOf)]
    flags SandboxAllowance: u8 {
        const ALLOW_NOTHING = 0x00,
        const ALLOW_SAME_ORIGIN = 0x01,
        const ALLOW_TOP_NAVIGATION = 0x02,
        const ALLOW_FORMS = 0x04,
        const ALLOW_SCRIPTS = 0x08,
        const ALLOW_POINTER_LOCK = 0x10,
        const ALLOW_POPUPS = 0x20
    }
}

#[dom_struct]
pub struct HTMLIFrameElement {
    htmlelement: HTMLElement,
    pipeline_id: Cell<Option<PipelineId>>,
    subpage_id: Cell<Option<SubpageId>>,
    sandbox: MutNullableHeap<JS<DOMTokenList>>,
    sandbox_allowance: Cell<Option<SandboxAllowance>>,
    load_blocker: DOMRefCell<Option<LoadBlocker>>,
    visibility: Cell<bool>,
}

impl HTMLIFrameElement {
    pub fn is_sandboxed(&self) -> bool {
        self.sandbox_allowance.get().is_some()
    }

    /// <https://html.spec.whatwg.org/multipage/#otherwise-steps-for-iframe-or-frame-elements>,
    /// step 1.
    fn get_url(&self) -> Url {
        let element = self.upcast::<Element>();
        element.get_attribute(&ns!(), &atom!("src")).and_then(|src| {
            let url = src.value();
            if url.is_empty() {
                None
            } else {
                document_from_node(self).base_url().join(&url).ok()
            }
        }).unwrap_or_else(|| Url::parse("about:blank").unwrap())
    }

    pub fn generate_new_subpage_id(&self) -> (SubpageId, Option<SubpageId>) {
        self.pipeline_id.set(Some(PipelineId::new()));

        let old_subpage_id = self.subpage_id.get();
        let win = window_from_node(self);
        let subpage_id = win.get_next_subpage_id();
        self.subpage_id.set(Some(subpage_id));
        (subpage_id, old_subpage_id)
    }

    pub fn navigate_or_reload_child_browsing_context(&self, load_data: Option<LoadData>) {
        let sandboxed = if self.is_sandboxed() {
            IFrameSandboxed
        } else {
            IFrameUnsandboxed
        };

        let document = document_from_node(self);

        let mut load_blocker = self.load_blocker.borrow_mut();
        // Any oustanding load is finished from the point of view of the blocked
        // document; the new navigation will continue blocking it.
        LoadBlocker::terminate(&mut load_blocker);

        //TODO(#9592): Deal with the case where an iframe is being reloaded so url is None.
        //      The iframe should always have access to the nested context's active
        //      document URL through the browsing context.
        if let Some(ref load_data) = load_data {
            *load_blocker = Some(LoadBlocker::new(&*document, LoadType::Subframe(load_data.url.clone())));
        }

        let window = window_from_node(self);
        let (new_subpage_id, old_subpage_id) = self.generate_new_subpage_id();
        let new_pipeline_id = self.pipeline_id.get().unwrap();
        let private_iframe = self.privatebrowsing();
        let frame_type = if self.Mozbrowser() { FrameType::MozBrowserIFrame } else { FrameType::IFrame };

        let load_info = IFrameLoadInfo {
            load_data: load_data,
            containing_pipeline_id: window.pipeline(),
            new_subpage_id: new_subpage_id,
            old_subpage_id: old_subpage_id,
            new_pipeline_id: new_pipeline_id,
            sandbox: sandboxed,
            is_private: private_iframe,
            frame_type: frame_type,
        };
        window.constellation_chan()
              .send(ConstellationMsg::ScriptLoadedURLInIFrame(load_info))
              .unwrap();

        if PREFS.is_mozbrowser_enabled() {
            // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowserloadstart
            self.dispatch_mozbrowser_event(MozBrowserEvent::LoadStart);
        }
    }

    pub fn process_the_iframe_attributes(&self) {
        let url = self.get_url();

        let document = document_from_node(self);
        self.navigate_or_reload_child_browsing_context(
            Some(LoadData::new(url, document.get_referrer_policy(), Some(document.url().clone()))));
    }

    #[allow(unsafe_code)]
    pub fn dispatch_mozbrowser_event(&self, event: MozBrowserEvent) {
        assert!(PREFS.is_mozbrowser_enabled());

        if self.Mozbrowser() {
            let window = window_from_node(self);
            let custom_event = build_mozbrowser_custom_event(&window, event);
            custom_event.upcast::<Event>().fire(self.upcast());
        }
    }

    pub fn update_subpage_id(&self, new_subpage_id: SubpageId, new_pipeline_id: PipelineId) {
        self.subpage_id.set(Some(new_subpage_id));
        self.pipeline_id.set(Some(new_pipeline_id));

        let mut blocker = self.load_blocker.borrow_mut();
        LoadBlocker::terminate(&mut blocker);

        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }

    fn new_inherited(localName: Atom,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLIFrameElement {
        HTMLIFrameElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document),
            pipeline_id: Cell::new(None),
            subpage_id: Cell::new(None),
            sandbox: Default::default(),
            sandbox_allowance: Cell::new(None),
            load_blocker: DOMRefCell::new(None),
            visibility: Cell::new(true),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLIFrameElement> {
        Node::reflect_node(box HTMLIFrameElement::new_inherited(localName, prefix, document),
                           document,
                           HTMLIFrameElementBinding::Wrap)
    }

    #[inline]
    pub fn pipeline_id(&self) -> Option<PipelineId> {
        self.pipeline_id.get()
    }

    #[inline]
    pub fn subpage_id(&self) -> Option<SubpageId> {
        self.subpage_id.get()
    }

    pub fn pipeline(&self) -> Option<PipelineId> {
        self.pipeline_id.get()
    }

    pub fn change_visibility_status(&self, visibility: bool) {
        if self.visibility.get() != visibility {
            self.visibility.set(visibility);

            // Visibility changes are only exposed to Mozbrowser iframes
            if self.Mozbrowser() {
                self.dispatch_mozbrowser_event(MozBrowserEvent::VisibilityChange(visibility));
            }
        }
    }

    pub fn set_visible(&self, visible: bool) {
        if let Some(pipeline_id) = self.pipeline_id.get() {
            let window = window_from_node(self);
            let window = window.r();
            let msg = ConstellationMsg::SetVisible(pipeline_id, visible);
            window.constellation_chan().send(msg).unwrap();
        }
    }

    /// https://html.spec.whatwg.org/multipage/#iframe-load-event-steps steps 1-4
    pub fn iframe_load_event_steps(&self, loaded_pipeline: PipelineId) {
        // TODO(#9592): assert that the load blocker is present at all times when we
        //              can guarantee that it's created for the case of iframe.reload().
        assert_eq!(loaded_pipeline, self.pipeline().unwrap());

        // TODO A cross-origin child document would not be easily accessible
        //      from this script thread. It's unclear how to implement
        //      steps 2, 3, and 5 efficiently in this case.
        // TODO Step 2 - check child document `mute iframe load` flag
        // TODO Step 3 - set child document  `mut iframe load` flag

        // Step 4
        self.upcast::<EventTarget>().fire_simple_event("load");

        let mut blocker = self.load_blocker.borrow_mut();
        LoadBlocker::terminate(&mut blocker);

        // TODO Step 5 - unset child document `mut iframe load` flag

        let window = window_from_node(self);
        window.reflow(ReflowGoal::ForDisplay,
                      ReflowQueryType::NoQuery,
                      ReflowReason::IFrameLoadEvent);
    }

    /// Check whether the iframe has the mozprivatebrowsing attribute set
    pub fn privatebrowsing(&self) -> bool {
        if self.Mozbrowser() {
            let element = self.upcast::<Element>();
            element.has_attribute(&Atom::from("mozprivatebrowsing"))
        } else {
            false
        }
    }

    pub fn get_content_window(&self) -> Option<Root<Window>> {
        self.subpage_id.get().and_then(|subpage_id| {
            let window = window_from_node(self);
            let window = window.r();
            let browsing_context = window.browsing_context();
            browsing_context.find_child_by_subpage(subpage_id)
        })
    }

}

pub trait HTMLIFrameElementLayoutMethods {
    fn pipeline_id(self) -> Option<PipelineId>;
    fn get_width(&self) -> LengthOrPercentageOrAuto;
    fn get_height(&self) -> LengthOrPercentageOrAuto;
}

impl HTMLIFrameElementLayoutMethods for LayoutJS<HTMLIFrameElement> {
    #[inline]
    #[allow(unsafe_code)]
    fn pipeline_id(self) -> Option<PipelineId> {
        unsafe {
            (*self.unsafe_get()).pipeline_id.get()
        }
    }

    #[allow(unsafe_code)]
    fn get_width(&self) -> LengthOrPercentageOrAuto {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &atom!("width"))
                .map(AttrValue::as_dimension)
                .cloned()
                .unwrap_or(LengthOrPercentageOrAuto::Auto)
        }
    }

    #[allow(unsafe_code)]
    fn get_height(&self) -> LengthOrPercentageOrAuto {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &atom!("height"))
                .map(AttrValue::as_dimension)
                .cloned()
                .unwrap_or(LengthOrPercentageOrAuto::Auto)
        }
    }
}

#[allow(unsafe_code)]
pub fn build_mozbrowser_custom_event(window: &Window, event: MozBrowserEvent) -> Root<CustomEvent> {
    // TODO(gw): Support mozbrowser event types that have detail which is not a string.
    // See https://developer.mozilla.org/en-US/docs/Web/API/Using_the_Browser_API
    // for a list of mozbrowser events.
    let cx = window.get_cx();
    let _ac = JSAutoCompartment::new(cx, window.reflector().get_jsobject().get());
    rooted!(in(cx) let mut detail = UndefinedValue());
    let event_name = Atom::from(event.name());
    unsafe { build_mozbrowser_event_detail(event, cx, detail.handle_mut()); }
    CustomEvent::new(GlobalRef::Window(window),
                     event_name,
                     true,
                     true,
                     detail.handle())
}

#[allow(unsafe_code)]
unsafe fn build_mozbrowser_event_detail(event: MozBrowserEvent,
                                        cx: *mut JSContext,
                                        rval: MutableHandleValue) {
    match event {
        MozBrowserEvent::AsyncScroll | MozBrowserEvent::Close | MozBrowserEvent::ContextMenu |
        MozBrowserEvent::LoadEnd | MozBrowserEvent::LoadStart |
        MozBrowserEvent::Connected | MozBrowserEvent::OpenSearch  |
        MozBrowserEvent::UsernameAndPasswordRequired => {
            rval.set(NullValue());
        }
        MozBrowserEvent::Error(error_type, description, report) => {
            BrowserElementErrorEventDetail {
                type_: Some(DOMString::from(error_type.name())),
                description: Some(DOMString::from(description)),
                report: Some(DOMString::from(report)),
                version: Some(DOMString::from_string(servo_version().into())),
            }.to_jsval(cx, rval);
        },
        MozBrowserEvent::SecurityChange(https_state) => {
            BrowserElementSecurityChangeDetail {
                // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowsersecuritychange
                state: Some(DOMString::from(match https_state {
                    HttpsState::Modern => "secure",
                    HttpsState::Deprecated => "broken",
                    HttpsState::None => "insecure",
                }.to_owned())),
                // FIXME - Not supported yet:
                trackingContent: None,
                mixedContent: None,
                trackingState: None,
                extendedValidation: None,
                mixedState: None,
            }.to_jsval(cx, rval);
        }
        MozBrowserEvent::TitleChange(ref string) => {
            string.to_jsval(cx, rval);
        }
        MozBrowserEvent::LocationChange(url, can_go_back, can_go_forward) => {
            BrowserElementLocationChangeEventDetail {
                url: Some(DOMString::from(url)),
                canGoBack: Some(can_go_back),
                canGoForward: Some(can_go_forward),
            }.to_jsval(cx, rval);
        }
        MozBrowserEvent::OpenTab(url) => {
            BrowserElementOpenTabEventDetail {
                url: Some(DOMString::from(url)),
            }.to_jsval(cx, rval);
        }
        MozBrowserEvent::OpenWindow(url, target, features) => {
            BrowserElementOpenWindowEventDetail {
                url: Some(DOMString::from(url)),
                target: target.map(DOMString::from),
                features: features.map(DOMString::from),
            }.to_jsval(cx, rval);
        }
        MozBrowserEvent::IconChange(rel, href, sizes) => {
            BrowserElementIconChangeEventDetail {
                rel: Some(DOMString::from(rel)),
                href: Some(DOMString::from(href)),
                sizes: Some(DOMString::from(sizes)),
            }.to_jsval(cx, rval);
        }
        MozBrowserEvent::ShowModalPrompt(prompt_type, title, message, return_value) => {
            BrowserShowModalPromptEventDetail {
                promptType: Some(DOMString::from(prompt_type)),
                title: Some(DOMString::from(title)),
                message: Some(DOMString::from(message)),
                returnValue: Some(DOMString::from(return_value)),
            }.to_jsval(cx, rval)
        }
        MozBrowserEvent::VisibilityChange(visibility) => {
            BrowserElementVisibilityChangeEventDetail {
                visible: Some(visibility),
            }.to_jsval(cx, rval);
        }
    }
}


pub fn Navigate(iframe: &HTMLIFrameElement, direction: NavigationDirection) -> ErrorResult {
    if iframe.Mozbrowser() {
        if iframe.upcast::<Node>().is_in_doc() {
            let window = window_from_node(iframe);

            let pipeline_info = Some((window.pipeline(),
                                      iframe.subpage_id().unwrap()));
            let msg = ConstellationMsg::Navigate(pipeline_info, direction);
            window.constellation_chan().send(msg).unwrap();
        }

        Ok(())
    } else {
        debug!("this frame is not mozbrowser: mozbrowser attribute missing, or not a top
            level window, or mozbrowser preference not set (use --pref dom.mozbrowser.enabled)");
        Err(Error::NotSupported)
    }
}

impl HTMLIFrameElementMethods for HTMLIFrameElement {
    // https://html.spec.whatwg.org/multipage/#dom-iframe-src
    fn Src(&self) -> DOMString {
        self.upcast::<Element>().get_string_attribute(&atom!("src"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-iframe-src
    fn SetSrc(&self, src: DOMString) {
        self.upcast::<Element>().set_url_attribute(&atom!("src"), src)
    }

    // https://html.spec.whatwg.org/multipage/#dom-iframe-sandbox
    fn Sandbox(&self) -> Root<DOMTokenList> {
        self.sandbox.or_init(|| DOMTokenList::new(self.upcast::<Element>(), &atom!("sandbox")))
    }

    // https://html.spec.whatwg.org/multipage/#dom-iframe-contentwindow
    fn GetContentWindow(&self) -> Option<Root<BrowsingContext>> {
        match self.get_content_window() {
            Some(ref window) => Some(window.browsing_context()),
            None => None
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-iframe-contentdocument
    fn GetContentDocument(&self) -> Option<Root<Document>> {
        self.get_content_window().and_then(|window| {
            // FIXME(#10964): this should use the Document's origin and the
            //                origin of the incumbent settings object.
            let self_url = self.get_url();
            let win_url = window_from_node(self).get_url();

            if UrlHelper::SameOrigin(&self_url, &win_url) {
                Some(window.Document())
            } else {
                None
            }
        })
    }

    // Experimental mozbrowser implementation is based on the webidl
    // present in the gecko source tree, and the documentation here:
    // https://developer.mozilla.org/en-US/docs/Web/API/Using_the_Browser_API
    // https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe#attr-mozbrowser
    fn Mozbrowser(&self) -> bool {
        if window_from_node(self).is_mozbrowser() {
            let element = self.upcast::<Element>();
            element.has_attribute(&atom!("mozbrowser"))
        } else {
            false
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe#attr-mozbrowser
    fn SetMozbrowser(&self, value: bool) {
        let element = self.upcast::<Element>();
        element.set_bool_attribute(&atom!("mozbrowser"), value);
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/HTMLIFrameElement/goBack
    fn GoBack(&self) -> ErrorResult {
        Navigate(self, NavigationDirection::Back(1))
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/HTMLIFrameElement/goForward
    fn GoForward(&self) -> ErrorResult {
        Navigate(self, NavigationDirection::Forward(1))
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/HTMLIFrameElement/reload
    fn Reload(&self, _hardReload: bool) -> ErrorResult {
        if self.Mozbrowser() {
            if self.upcast::<Node>().is_in_doc() {
                self.navigate_or_reload_child_browsing_context(None);
            }
            Ok(())
        } else {
            debug!("this frame is not mozbrowser: mozbrowser attribute missing, or not a top
                level window, or mozbrowser preference not set (use --pref dom.mozbrowser.enabled)");
            Err(Error::NotSupported)
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/HTMLIFrameElement/setVisible
    fn SetVisible(&self, visible: bool) -> ErrorResult {
        if self.Mozbrowser() {
            self.set_visible(visible);
            Ok(())
        } else {
            debug!("this frame is not mozbrowser: mozbrowser attribute missing, or not a top
                level window, or mozbrowser preference not set (use --pref dom.mozbrowser.enabled)");
            Err(Error::NotSupported)
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/HTMLIFrameElement/getVisible
    fn GetVisible(&self) -> Fallible<bool> {
        if self.Mozbrowser() {
            Ok(self.visibility.get())
        } else {
            debug!("this frame is not mozbrowser: mozbrowser attribute missing, or not a top
                level window, or mozbrowser preference not set (use --pref dom.mozbrowser.enabled)");
            Err(Error::NotSupported)
        }
    }


    // https://developer.mozilla.org/en-US/docs/Web/API/HTMLIFrameElement/stop
    fn Stop(&self) -> ErrorResult {
        Err(Error::NotSupported)
    }

    // https://html.spec.whatwg.org/multipage/#dom-dim-width
    make_getter!(Width, "width");
    // https://html.spec.whatwg.org/multipage/#dom-dim-width
    make_dimension_setter!(SetWidth, "width");

    // https://html.spec.whatwg.org/multipage/#dom-dim-height
    make_getter!(Height, "height");
    // https://html.spec.whatwg.org/multipage/#dom-dim-height
    make_dimension_setter!(SetHeight, "height");

    // check-tidy: no specs after this line
    fn SetMozprivatebrowsing(&self, value: bool) {
        let element = self.upcast::<Element>();
        element.set_bool_attribute(&Atom::from("mozprivatebrowsing"), value);
    }

    fn Mozprivatebrowsing(&self) -> bool {
        if window_from_node(self).is_mozbrowser() {
            let element = self.upcast::<Element>();
            element.has_attribute(&Atom::from("mozprivatebrowsing"))
        } else {
            false
        }
    }
}

impl VirtualMethods for HTMLIFrameElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &atom!("sandbox") => {
                self.sandbox_allowance.set(mutation.new_value(attr).map(|value| {
                    let mut modes = ALLOW_NOTHING;
                    for token in value.as_tokens() {
                        modes |= match &*token.to_ascii_lowercase() {
                            "allow-same-origin" => ALLOW_SAME_ORIGIN,
                            "allow-forms" => ALLOW_FORMS,
                            "allow-pointer-lock" => ALLOW_POINTER_LOCK,
                            "allow-popups" => ALLOW_POPUPS,
                            "allow-scripts" => ALLOW_SCRIPTS,
                            "allow-top-navigation" => ALLOW_TOP_NAVIGATION,
                            _ => ALLOW_NOTHING
                        };
                    }
                    modes
                }));
            },
            &atom!("src") => {
                if let AttributeMutation::Set(_) = mutation {
                    if self.upcast::<Node>().is_in_doc() {
                        self.process_the_iframe_attributes();
                    }
                }
            },
            _ => {},
        }
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("sandbox") => AttrValue::from_serialized_tokenlist(value.into()),
            &atom!("width") => AttrValue::from_dimension(value.into()),
            &atom!("height") => AttrValue::from_dimension(value.into()),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        if tree_in_doc {
            self.process_the_iframe_attributes();
        }
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);

        let mut blocker = self.load_blocker.borrow_mut();
        LoadBlocker::terminate(&mut blocker);

        // https://html.spec.whatwg.org/multipage/#a-browsing-context-is-discarded
        if let Some(pipeline_id) = self.pipeline_id.get() {
            let window = window_from_node(self);

            // The only reason we're waiting for the iframe to be totally
            // removed is to ensure the script thread can't add iframes faster
            // than the compositor can remove them.
            //
            // Since most of this cleanup doesn't happen on same-origin
            // iframes, and since that would cause a deadlock, don't do it.
            let same_origin = {
                // FIXME(#10968): this should probably match the origin check in
                //                HTMLIFrameElement::contentDocument.
                let self_url = self.get_url();
                let win_url = window_from_node(self).get_url();
                UrlHelper::SameOrigin(&self_url, &win_url)
            };
            let (sender, receiver) = if same_origin {
                (None, None)
            } else {
                let (sender, receiver) = ipc::channel().unwrap();
                (Some(sender), Some(receiver))
            };
            let msg = ConstellationMsg::RemoveIFrame(pipeline_id, sender);
            window.constellation_chan().send(msg).unwrap();
            if let Some(receiver) = receiver {
                receiver.recv().unwrap()
            }

            // Resetting the subpage id to None is required here so that
            // if this iframe is subsequently re-added to the document
            // the load doesn't think that it's a navigation, but instead
            // a new iframe. Without this, the constellation gets very
            // confused.
            self.subpage_id.set(None);
            self.pipeline_id.set(None);
        }
    }
}
