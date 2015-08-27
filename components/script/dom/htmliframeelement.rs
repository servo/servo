/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrHelpersForLayout, AttrValue};
use dom::bindings::codegen::Bindings::HTMLIFrameElementBinding;
use dom::bindings::codegen::Bindings::HTMLIFrameElementBinding::HTMLIFrameElementMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::HTMLIFrameElementDerived;
use dom::bindings::codegen::InheritTypes::{EventTargetCast, HTMLElementCast};
use dom::bindings::codegen::InheritTypes::{NodeCast, ElementCast, EventCast};
use dom::bindings::conversions::ToJSValConvertible;
use dom::bindings::error::Error::NotSupported;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Root};
use dom::bindings::utils::Reflectable;
use dom::customevent::CustomEvent;
use dom::document::Document;
use dom::element::{ElementTypeId, self};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId, window_from_node};
use dom::urlhelper::UrlHelper;
use dom::virtualmethods::VirtualMethods;
use dom::window::Window;
use page::IterablePage;

use msg::constellation_msg::IFrameSandboxState::{IFrameSandboxed, IFrameUnsandboxed};
use msg::constellation_msg::Msg as ConstellationMsg;
use msg::constellation_msg::{PipelineId, SubpageId, ConstellationChan, MozBrowserEvent, NavigationDirection};
use string_cache::Atom;
use util::opts;
use util::str::DOMString;

use js::jsapi::{RootedValue, JSAutoRequest, JSAutoCompartment};
use js::jsval::UndefinedValue;
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::cell::Cell;
use url::{Url, UrlParser};
use util::str::{self, LengthOrPercentageOrAuto};

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
    subpage_id: Cell<Option<SubpageId>>,
    containing_page_pipeline_id: Cell<Option<PipelineId>>,
    sandbox: Cell<Option<u8>>,
}

impl HTMLIFrameElementDerived for EventTarget {
    fn is_htmliframeelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLIFrameElement)))
    }
}



impl HTMLIFrameElement {
    pub fn is_sandboxed(&self) -> bool {
        self.sandbox.get().is_some()
    }

    pub fn get_url(&self) -> Option<Url> {
        let element = ElementCast::from_ref(self);
        element.get_attribute(&ns!(""), &atom!("src")).and_then(|src| {
            let url = src.r().value();
            if url.is_empty() {
                None
            } else {
                let window = window_from_node(self);
                UrlParser::new().base_url(&window.r().get_url())
                    .parse(&url).ok()
            }
        })
    }

    pub fn generate_new_subpage_id(&self) -> (SubpageId, Option<SubpageId>) {
        let old_subpage_id = self.subpage_id.get();
        let win = window_from_node(self);
        let subpage_id = win.r().get_next_subpage_id();
        self.subpage_id.set(Some(subpage_id));
        (subpage_id, old_subpage_id)
    }

    pub fn navigate_child_browsing_context(&self, url: Url) {
        let sandboxed = if self.is_sandboxed() {
            IFrameSandboxed
        } else {
            IFrameUnsandboxed
        };

        let window = window_from_node(self);
        let window = window.r();
        let (new_subpage_id, old_subpage_id) = self.generate_new_subpage_id();

        self.containing_page_pipeline_id.set(Some(window.pipeline()));

        let ConstellationChan(ref chan) = window.constellation_chan();
        chan.send(ConstellationMsg::ScriptLoadedURLInIFrame(url,
                                                            window.pipeline(),
                                                            new_subpage_id,
                                                            old_subpage_id,
                                                            sandboxed)).unwrap();

        if opts::experimental_enabled() {
            // https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowserloadstart
            self.dispatch_mozbrowser_event(MozBrowserEvent::LoadStart);
        }
    }

    pub fn process_the_iframe_attributes(&self) {
        let url = match self.get_url() {
            Some(url) => url.clone(),
            None => Url::parse("about:blank").unwrap(),
        };

        self.navigate_child_browsing_context(url);
    }

    pub fn dispatch_mozbrowser_event(&self, event: MozBrowserEvent) {
        // TODO(gw): Support mozbrowser event types that have detail which is not a string.
        // See https://developer.mozilla.org/en-US/docs/Web/API/Using_the_Browser_API
        // for a list of mozbrowser events.
        assert!(opts::experimental_enabled());

        if self.Mozbrowser() {
            let window = window_from_node(self);
            let cx = window.r().get_cx();
            let _ar = JSAutoRequest::new(cx);
            let _ac = JSAutoCompartment::new(cx, window.reflector().get_jsobject().get());
            let mut detail = RootedValue::new(cx, UndefinedValue());
            event.detail().to_jsval(cx, detail.handle_mut());
            let custom_event = CustomEvent::new(GlobalRef::Window(window.r()),
                                                event.name().to_owned(),
                                                true,
                                                true,
                                                detail.handle());
            let target = EventTargetCast::from_ref(self);
            let event = EventCast::from_ref(custom_event.r());
            event.fire(target);
        }
    }

    pub fn update_subpage_id(&self, new_subpage_id: SubpageId) {
        self.subpage_id.set(Some(new_subpage_id));
    }
}

impl HTMLIFrameElement {
    #[allow(unsafe_code)]
    pub fn get_width(&self) -> LengthOrPercentageOrAuto {
        unsafe {
            element::get_attr_for_layout(ElementCast::from_ref(&*self),
                                         &ns!(""),
                                         &atom!("width")).map(|attribute| {
                str::parse_length(&**attribute.value_for_layout())
            }).unwrap_or(LengthOrPercentageOrAuto::Auto)
        }
    }

    #[allow(unsafe_code)]
    pub fn get_height(&self) -> LengthOrPercentageOrAuto {
        unsafe {
            element::get_attr_for_layout(ElementCast::from_ref(&*self),
                                         &ns!(""),
                                         &atom!("height")).map(|attribute| {
                str::parse_length(&**attribute.value_for_layout())
            }).unwrap_or(LengthOrPercentageOrAuto::Auto)
        }
    }
}

impl HTMLIFrameElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLIFrameElement {
        HTMLIFrameElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLIFrameElement, localName, prefix, document),
            subpage_id: Cell::new(None),
            containing_page_pipeline_id: Cell::new(None),
            sandbox: Cell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLIFrameElement> {
        let element = HTMLIFrameElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLIFrameElementBinding::Wrap)
    }

    #[inline]
    pub fn containing_page_pipeline_id(&self) -> Option<PipelineId> {
        self.containing_page_pipeline_id.get()
    }

    #[inline]
    pub fn subpage_id(&self) -> Option<SubpageId> {
        self.subpage_id.get()
    }
}

pub fn Navigate(iframe: &HTMLIFrameElement, direction: NavigationDirection) -> Fallible<()> {
    if iframe.Mozbrowser() {
        let node = NodeCast::from_ref(iframe);
        if node.is_in_doc() {
            let window = window_from_node(iframe);
            let window = window.r();

            let pipeline_info = Some((iframe.containing_page_pipeline_id().unwrap(),
                                      iframe.subpage_id().unwrap()));
            let ConstellationChan(ref chan) = window.constellation_chan();
            let msg = ConstellationMsg::Navigate(pipeline_info, direction);
            chan.send(msg).unwrap();
        }

        Ok(())
    } else {
        debug!("this frame is not mozbrowser (or experimental_enabled is false)");
        Err(NotSupported)
    }
}

impl<'a> HTMLIFrameElementMethods for &'a HTMLIFrameElement {
    // https://html.spec.whatwg.org/multipage/#dom-iframe-src
    fn Src(self) -> DOMString {
        let element = ElementCast::from_ref(self);
        element.get_string_attribute(&atom!("src"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-iframe-src
    fn SetSrc(self, src: DOMString) {
        let element = ElementCast::from_ref(self);
        element.set_url_attribute(&atom!("src"), src)
    }

    // https://html.spec.whatwg.org/multipage/#dom-iframe-sandbox
    fn Sandbox(self) -> DOMString {
        let element = ElementCast::from_ref(self);
        element.get_string_attribute(&atom!("sandbox"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-iframe-sandbox
    fn SetSandbox(self, sandbox: DOMString) {
        let element = ElementCast::from_ref(self);
        element.set_tokenlist_attribute(&atom!("sandbox"), sandbox);
    }

    // https://html.spec.whatwg.org/multipage/#dom-iframe-contentwindow
    fn GetContentWindow(self) -> Option<Root<Window>> {
        self.subpage_id.get().and_then(|subpage_id| {
            let window = window_from_node(self);
            let window = window.r();
            let children = window.page().children.borrow();
            children.iter().find(|page| {
                let window = page.window();
                window.r().subpage() == Some(subpage_id)
            }).map(|page| page.window())
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-iframe-contentdocument
    fn GetContentDocument(self) -> Option<Root<Document>> {
        self.GetContentWindow().and_then(|window| {
            let self_url = match self.get_url() {
                Some(self_url) => self_url,
                None => return None,
            };
            let win_url = window_from_node(self).r().get_url();

            if UrlHelper::SameOrigin(&self_url, &win_url) {
                Some(window.r().Document())
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
    fn Mozbrowser(self) -> bool {
        if opts::experimental_enabled() {
            let element = ElementCast::from_ref(self);
            element.has_attribute(&Atom::from_slice("mozbrowser"))
        } else {
            false
        }
    }

    // https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe#attr-mozbrowser
    fn SetMozbrowser(self, value: bool) -> ErrorResult {
        if opts::experimental_enabled() {
            let element = ElementCast::from_ref(self);
            element.set_bool_attribute(&Atom::from_slice("mozbrowser"), value);
        }
        Ok(())
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/HTMLIFrameElement/goBack
    fn GoBack(self) -> Fallible<()> {
        Navigate(self, NavigationDirection::Back)
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/HTMLIFrameElement/goForward
    fn GoForward(self) -> Fallible<()> {
        Navigate(self, NavigationDirection::Forward)
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/HTMLIFrameElement/reload
    fn Reload(self, _hardReload: bool) -> Fallible<()> {
        Err(NotSupported)
    }

    // https://developer.mozilla.org/en-US/docs/Web/API/HTMLIFrameElement/stop
    fn Stop(self) -> Fallible<()> {
        Err(NotSupported)
    }

    make_getter!(Width);

    make_setter!(SetWidth, "width");

    make_getter!(Height);

    make_setter!(SetHeight, "height");
}

impl VirtualMethods for HTMLIFrameElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &HTMLElement = HTMLElementCast::from_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.after_set_attr(attr);
        }

        match attr.local_name() {
            &atom!("sandbox") => {
                let mut modes = SandboxAllowance::AllowNothing as u8;
                if let Some(ref tokens) = attr.value().tokens() {
                    for token in *tokens {
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
                }
                self.sandbox.set(Some(modes));
            }
            &atom!("src") => {
                let node = NodeCast::from_ref(self);
                if node.is_in_doc() {
                    self.process_the_iframe_attributes()
                }
            },
            _ => ()
        }
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("sandbox") => AttrValue::from_serialized_tokenlist(value),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }

    fn before_remove_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
           s.before_remove_attr(attr);
        }

        match attr.local_name() {
            &atom!("sandbox") => self.sandbox.set(None),
            _ => ()
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

    fn unbind_from_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.unbind_from_tree(tree_in_doc);
        }

        // https://html.spec.whatwg.org/multipage/#a-browsing-context-is-discarded
        match (self.containing_page_pipeline_id(), self.subpage_id()) {
            (Some(containing_pipeline_id), Some(subpage_id)) => {
                let window = window_from_node(self);
                let window = window.r();

                let ConstellationChan(ref chan) = window.constellation_chan();
                let msg = ConstellationMsg::RemoveIFrame(containing_pipeline_id,
                                                         subpage_id);
                chan.send(msg).unwrap();

                // Resetting the subpage id to None is required here so that
                // if this iframe is subsequently re-added to the document
                // the load doesn't think that it's a navigation, but instead
                // a new iframe. Without this, the constellation gets very
                // confused.
                self.subpage_id.set(None);
            }
            _ => {}
        }
    }
}
