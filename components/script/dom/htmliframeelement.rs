/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::attr::AttrValue;
use dom::attr::AttrHelpers;
use dom::bindings::codegen::Bindings::HTMLIFrameElementBinding;
use dom::bindings::codegen::Bindings::HTMLIFrameElementBinding::HTMLIFrameElementMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::{NodeCast, ElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLIFrameElementDerived};
use dom::bindings::js::{JSRef, Temporary, OptionalRootable};
use dom::document::Document;
use dom::element::Element;
use dom::element::AttributeHandlers;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeHelpers, NodeTypeId, window_from_node};
use dom::urlhelper::UrlHelper;
use dom::virtualmethods::VirtualMethods;
use dom::window::Window;
use page::{IterablePage, Page};

use msg::constellation_msg::{PipelineId, SubpageId, ConstellationChan};
use msg::constellation_msg::IFrameSandboxState::{IFrameSandboxed, IFrameUnsandboxed};
use msg::constellation_msg::Msg as ConstellationMsg;
use util::str::DOMString;
use string_cache::Atom;

use std::ascii::AsciiExt;
use std::cell::Cell;
use url::{Url, UrlParser};

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
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLIFrameElement)))
    }
}

pub trait HTMLIFrameElementHelpers {
    fn is_sandboxed(self) -> bool;
    fn get_url(self) -> Option<Url>;
    /// http://www.whatwg.org/html/#process-the-iframe-attributes
    fn process_the_iframe_attributes(self);
    fn generate_new_subpage_id(self, page: &Page) -> (SubpageId, Option<SubpageId>);
}

impl<'a> HTMLIFrameElementHelpers for JSRef<'a, HTMLIFrameElement> {
    fn is_sandboxed(self) -> bool {
        self.sandbox.get().is_some()
    }

    fn get_url(self) -> Option<Url> {
        let element: JSRef<Element> = ElementCast::from_ref(self);
        element.get_attribute(ns!(""), &atom!("src")).root().and_then(|src| {
            let url = src.r().value();
            if url.as_slice().is_empty() {
                None
            } else {
                let window = window_from_node(self).root();
                UrlParser::new().base_url(&window.r().page().get_url())
                    .parse(url.as_slice()).ok()
            }
        })
    }

    fn generate_new_subpage_id(self, page: &Page) -> (SubpageId, Option<SubpageId>) {
        let old_subpage_id = self.subpage_id.get();
        let subpage_id = page.get_next_subpage_id();
        self.subpage_id.set(Some(subpage_id));
        (subpage_id, old_subpage_id)
    }

    fn process_the_iframe_attributes(self) {
        let url = match self.get_url() {
            Some(url) => url.clone(),
            None => Url::parse("about:blank").unwrap(),
        };

        let sandboxed = if self.is_sandboxed() {
            IFrameSandboxed
        } else {
            IFrameUnsandboxed
        };

        let window = window_from_node(self).root();
        let window = window.r();
        let page = window.page();
        let (new_subpage_id, old_subpage_id) = self.generate_new_subpage_id(page);

        self.containing_page_pipeline_id.set(Some(page.id));

        let ConstellationChan(ref chan) = page.constellation_chan;
    chan.send(ConstellationMsg::ScriptLoadedURLInIFrame(url,
                                                        page.id,
                                                        new_subpage_id,
                                                        old_subpage_id,
                                                        sandboxed)).unwrap();
    }
}

impl HTMLIFrameElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLIFrameElement {
        HTMLIFrameElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLIFrameElement, localName, prefix, document),
            subpage_id: Cell::new(None),
            containing_page_pipeline_id: Cell::new(None),
            sandbox: Cell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLIFrameElement> {
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

impl<'a> HTMLIFrameElementMethods for JSRef<'a, HTMLIFrameElement> {
    fn Src(self) -> DOMString {
        let element: JSRef<Element> = ElementCast::from_ref(self);
        element.get_string_attribute(&atom!("src"))
    }

    fn SetSrc(self, src: DOMString) {
        let element: JSRef<Element> = ElementCast::from_ref(self);
        element.set_url_attribute(&atom!("src"), src)
    }

    fn Sandbox(self) -> DOMString {
        let element: JSRef<Element> = ElementCast::from_ref(self);
        element.get_string_attribute(&atom!("sandbox"))
    }

    fn SetSandbox(self, sandbox: DOMString) {
        let element: JSRef<Element> = ElementCast::from_ref(self);
        element.set_tokenlist_attribute(&atom!("sandbox"), sandbox);
    }

    fn GetContentWindow(self) -> Option<Temporary<Window>> {
        self.subpage_id.get().and_then(|subpage_id| {
            let window = window_from_node(self).root();
            let window = window.r();
            let children = window.page().children.borrow();
            let child = children.iter().find(|child| {
                child.subpage_id.unwrap() == subpage_id
            });
            child.and_then(|page| {
                page.frame.borrow().as_ref().map(|frame| {
                    Temporary::new(frame.window.clone())
                })
            })
        })
    }

    fn GetContentDocument(self) -> Option<Temporary<Document>> {
        self.GetContentWindow().root().and_then(|window| {
            let self_url = match self.get_url() {
                Some(self_url) => self_url,
                None => return None,
            };
            let win_url = window_from_node(self).root().r().page().get_url();

            if UrlHelper::SameOrigin(&self_url, &win_url) {
                Some(window.r().Document())
            } else {
                None
            }
        })
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLIFrameElement> {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(attr),
            _ => ()
        }

        match attr.local_name() {
            &atom!("sandbox") => {
                let mut modes = SandboxAllowance::AllowNothing as u8;
                if let Some(ref tokens) = attr.value().tokens() {
                    for token in tokens.iter() {
                        modes |= match token.as_slice().to_ascii_lowercase().as_slice() {
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
            },
            &atom!("src") => {
                let node: JSRef<Node> = NodeCast::from_ref(*self);
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

    fn before_remove_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(attr),
            _ => ()
        }

        match attr.local_name() {
            &atom!("sandbox") => self.sandbox.set(None),
            _ => ()
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        match self.super_type() {
            Some(ref s) => s.bind_to_tree(tree_in_doc),
            _ => (),
        }

        if tree_in_doc {
            self.process_the_iframe_attributes();
        }
    }
}

