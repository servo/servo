/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::{LoadType, LoadBlocker};
use dom::attr::{Attr, AttrValue};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BrowserElementBinding::BrowserElementIconChangeEventDetail;
use dom::bindings::codegen::Bindings::BrowserElementBinding::BrowserElementLocationChangeEventDetail;
use dom::bindings::codegen::Bindings::BrowserElementBinding::BrowserElementSecurityChangeDetail;
use dom::bindings::codegen::Bindings::BrowserElementBinding::BrowserShowModalPromptEventDetail;
use dom::bindings::codegen::Bindings::HTMLIFrameElementBinding;
use dom::bindings::codegen::Bindings::HTMLIFrameElementBinding::HTMLIFrameElementMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::conversions::{ToJSValConvertible};
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{Root, LayoutJS};
use dom::bindings::reflector::Reflectable;
use dom::customevent::CustomEvent;
use dom::document::Document;
use dom::element::{AttributeMutation, Element, RawLayoutElementHelpers};
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::node::{Node, UnbindContext, window_from_node, document_from_node};
use dom::urlhelper::UrlHelper;
use dom::virtualmethods::VirtualMethods;
use dom::window::{ReflowReason, Window};
use ipc_channel::ipc;
use js::jsapi::{JSAutoCompartment, JSAutoRequest, RootedValue, JSContext, MutableHandleValue};
use js::jsval::{UndefinedValue, NullValue};
use layout_interface::ReflowQueryType;
use msg::constellation_msg::{ConstellationChan};
use msg::constellation_msg::{NavigationDirection, PipelineId, SubpageId};
use net_traits::response::HttpsState;
use page::IterablePage;
use script_traits::IFrameSandboxState::{IFrameSandboxed, IFrameUnsandboxed};
use script_traits::{IFrameLoadInfo, MozBrowserEvent, ScriptMsg as ConstellationMsg};
use std::ascii::AsciiExt;
use std::cell::Cell;
use string_cache::Atom;
use style::context::ReflowGoal;
use url::Url;
use util::prefs;
use util::str::{DOMString, LengthOrPercentageOrAuto};

pub fn mozbrowser_enabled() -> bool {
    prefs::get_pref("dom.mozbrowser.enabled").as_boolean().unwrap_or(false)
}

#[derive(HeapSizeOf)]
enum SandboxAllowance {
    AllowNothing = 0x00,
    AllowSameOrigin = 0x01,
    AllowTopNavigation = 0x02,
    AllowForms = 0x04,
    AllowScripts = 0x08,
    AllowPointerLock = 0x10,
    AllowPopups = 0x20
}

#[dom_struct]
pub struct HTMLIFrameElement {
    htmlelement: HTMLElement,
    pipeline_id: Cell<Option<PipelineId>>,
    subpage_id: Cell<Option<SubpageId>>,
    sandbox: Cell<Option<u8>>,
    load_blocker: DOMRefCell<Option<LoadBlocker>>,
}

impl HTMLIFrameElement {
    pub fn is_sandboxed(&self) -> bool {
        self.sandbox.get().is_some()
    }

    pub fn get_url(&self) -> Option<Url> {
        let element = self.upcast::<Element>();
        element.get_attribute(&ns!(), &atom!("src")).and_then(|src| {
            let url = src.value();
            if url.is_empty() {
                None
            } else {
                document_from_node(self).base_url().join(&url).ok()
            }
        })
    }

    pub fn generate_new_subpage_id(&self) -> (SubpageId, Option<SubpageId>) {
        self.pipeline_id.set(Some(PipelineId::new()));

        let old_subpage_id = self.subpage_id.get();
        let win = window_from_node(self);
        let subpage_id = win.get_next_subpage_id();
        self.subpage_id.set(Some(subpage_id));
        (subpage_id, old_subpage_id)
    }

    pub fn navigate_or_reload_child_browsing_context(&self, url: Option<Url>) {
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
        if let Some(ref url) = url {
            *load_blocker = Some(LoadBlocker::new(&*document, LoadType::Subframe(url.clone())));
        }

        let window = window_from_node(self);
        let window = window.r();
        let (new_subpage_id, old_subpage_id) = self.generate_new_subpage_id();
        let new_pipeline_id = self.pipeline_id.get().unwrap();
        let private_iframe = self.privatebrowsing();

        let ConstellationChan(ref chan) = window.constellation_chan();
        let load_info = IFrameLoadInfo {
            url: url,
            containing_pipeline_id: window.pipeline(),
            new_subpage_id: new_subpage_id,
            old_subpage_id: old_subpage_id,
            new_pipeline_id: new_pipeline_id,
            sandbox: sandboxed,
            is_private: private_iframe,
        };
        chan.send(ConstellationMsg::ScriptLoadedURLInIFrame(load_info)).unwrap();

        if mozbrowser_enabled() {
            // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowserloadstart
            self.dispatch_mozbrowser_event(MozBrowserEvent::LoadStart);
        }
    }

    pub fn process_the_iframe_attributes(&self) {
        let url = match self.get_url() {
            Some(url) => url.clone(),
            None => Url::parse("about:blank").unwrap(),
        };

        self.navigate_or_reload_child_browsing_context(Some(url));
    }

    #[allow(unsafe_code)]
    pub fn dispatch_mozbrowser_event(&self, event: MozBrowserEvent) {
        // TODO(gw): Support mozbrowser event types that have detail which is not a string.
        // See https://developer.mozilla.org/en-US/docs/Web/API/Using_the_Browser_API
        // for a list of mozbrowser events.
        assert!(mozbrowser_enabled());

        if self.Mozbrowser() {
            let window = window_from_node(self);
            let custom_event = unsafe {
                let cx = window.get_cx();
                let _ar = JSAutoRequest::new(cx);
                let _ac = JSAutoCompartment::new(cx, window.reflector().get_jsobject().get());
                let mut detail = RootedValue::new(cx, UndefinedValue());
                let event_name = Atom::from(event.name());
                self.build_mozbrowser_event_detail(event, cx, detail.handle_mut());
                CustomEvent::new(GlobalRef::Window(window.r()),
                                 event_name,
                                 true,
                                 true,
                                 detail.handle())
            };
            custom_event.upcast::<Event>().fire(self.upcast());
        }
    }

    pub fn update_subpage_id(&self, new_subpage_id: SubpageId) {
        self.subpage_id.set(Some(new_subpage_id));
    }

    fn new_inherited(localName: Atom,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLIFrameElement {
        HTMLIFrameElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document),
            pipeline_id: Cell::new(None),
            subpage_id: Cell::new(None),
            sandbox: Cell::new(None),
            load_blocker: DOMRefCell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLIFrameElement> {
        let element = HTMLIFrameElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLIFrameElementBinding::Wrap)
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

pub trait MozBrowserEventDetailBuilder {
    #[allow(unsafe_code)]
    unsafe fn build_mozbrowser_event_detail(&self,
                                            event: MozBrowserEvent,
                                            cx: *mut JSContext,
                                            rval: MutableHandleValue);
}

impl MozBrowserEventDetailBuilder for HTMLIFrameElement {
    #[allow(unsafe_code)]
    unsafe fn build_mozbrowser_event_detail(&self,
                                            event: MozBrowserEvent,
                                            cx: *mut JSContext,
                                            rval: MutableHandleValue) {
        match event {
            MozBrowserEvent::AsyncScroll | MozBrowserEvent::Close | MozBrowserEvent::ContextMenu |
            MozBrowserEvent::Error | MozBrowserEvent::LoadEnd | MozBrowserEvent::LoadStart |
            MozBrowserEvent::Connected | MozBrowserEvent::OpenWindow | MozBrowserEvent::OpenSearch  |
            MozBrowserEvent::UsernameAndPasswordRequired => {
                rval.set(NullValue());
            }
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
            MozBrowserEvent::LocationChange(uri, can_go_back, can_go_forward) => {
                BrowserElementLocationChangeEventDetail {
                    uri: Some(DOMString::from(uri)),
                    canGoBack: Some(can_go_back),
                    canGoForward: Some(can_go_forward),
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
        }
    }
}

pub fn Navigate(iframe: &HTMLIFrameElement, direction: NavigationDirection) -> ErrorResult {
    if iframe.Mozbrowser() {
        if iframe.upcast::<Node>().is_in_doc() {
            let window = window_from_node(iframe);
            let window = window.r();

            let pipeline_info = Some((window.pipeline(),
                                      iframe.subpage_id().unwrap()));
            let ConstellationChan(ref chan) = window.constellation_chan();
            let msg = ConstellationMsg::Navigate(pipeline_info, direction);
            chan.send(msg).unwrap();
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
    fn Sandbox(&self) -> DOMString {
        self.upcast::<Element>().get_string_attribute(&atom!("sandbox"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-iframe-sandbox
    fn SetSandbox(&self, sandbox: DOMString) {
        self.upcast::<Element>().set_tokenlist_attribute(&atom!("sandbox"), sandbox);
    }

    // https://html.spec.whatwg.org/multipage/#dom-iframe-contentwindow
    fn GetContentWindow(&self) -> Option<Root<Window>> {
        self.subpage_id.get().and_then(|subpage_id| {
            let window = window_from_node(self);
            let window = window.r();
            let children = window.page().children.borrow();
            children.iter().find(|page| {
                let window = page.window();
                window.subpage() == Some(subpage_id)
            }).map(|page| page.window())
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-iframe-contentdocument
    fn GetContentDocument(&self) -> Option<Root<Document>> {
        self.GetContentWindow().and_then(|window| {
            let self_url = match self.get_url() {
                Some(self_url) => self_url,
                None => return None,
            };
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

    // TODO(gw): Use experimental codegen when it is available to avoid
    // exposing these APIs. See https://github.com/servo/servo/issues/5264.

    // https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe#attr-mozbrowser
    fn Mozbrowser(&self) -> bool {
        // We don't want to allow mozbrowser iframes within iframes
        let is_root_pipeline = window_from_node(self).parent_info().is_none();
        if mozbrowser_enabled() && is_root_pipeline {
            let element = self.upcast::<Element>();
            element.has_attribute(&atom!("mozbrowser"))
        } else {
            false
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe#attr-mozbrowser
    fn SetMozbrowser(&self, value: bool) -> ErrorResult {
        if mozbrowser_enabled() {
            let element = self.upcast::<Element>();
            element.set_bool_attribute(&atom!("mozbrowser"), value);
        }
        Ok(())
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/HTMLIFrameElement/goBack
    fn GoBack(&self) -> ErrorResult {
        Navigate(self, NavigationDirection::Back)
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/HTMLIFrameElement/goForward
    fn GoForward(&self) -> ErrorResult {
        Navigate(self, NavigationDirection::Forward)
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
}

impl VirtualMethods for HTMLIFrameElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &atom!("sandbox") => {
                self.sandbox.set(mutation.new_value(attr).map(|value| {
                    let mut modes = SandboxAllowance::AllowNothing as u8;
                    for token in value.as_tokens() {
                        modes |= match &*token.to_ascii_lowercase() {
                            "allow-same-origin" => SandboxAllowance::AllowSameOrigin,
                            "allow-forms" => SandboxAllowance::AllowForms,
                            "allow-pointer-lock" => SandboxAllowance::AllowPointerLock,
                            "allow-popups" => SandboxAllowance::AllowPopups,
                            "allow-scripts" => SandboxAllowance::AllowScripts,
                            "allow-top-navigation" => SandboxAllowance::AllowTopNavigation,
                            _ => SandboxAllowance::AllowNothing
                        } as u8;
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
            &atom!("sandbox") => AttrValue::from_serialized_tokenlist(value),
            &atom!("width") => AttrValue::from_dimension(value),
            &atom!("height") => AttrValue::from_dimension(value),
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
            let window = window.r();

            // The only reason we're waiting for the iframe to be totally
            // removed is to ensure the script thread can't add iframes faster
            // than the compositor can remove them.
            //
            // Since most of this cleanup doesn't happen on same-origin
            // iframes, and since that would cause a deadlock, don't do it.
            let ConstellationChan(ref chan) = window.constellation_chan();
            let same_origin = if let Some(self_url) = self.get_url() {
                let win_url = window_from_node(self).get_url();
                UrlHelper::SameOrigin(&self_url, &win_url)
            } else {
                false
            };
            let (sender, receiver) = if same_origin {
                (None, None)
            } else {
                let (sender, receiver) = ipc::channel().unwrap();
                (Some(sender), Some(receiver))
            };
            let msg = ConstellationMsg::RemoveIFrame(pipeline_id, sender);
            chan.send(msg).unwrap();
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
