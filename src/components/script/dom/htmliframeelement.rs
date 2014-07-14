/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLIFrameElementBinding;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLIFrameElementDerived, HTMLElementCast};
use dom::bindings::js::{JSRef, Temporary, OptionalRootable};
use dom::bindings::trace::Traceable;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::{HTMLIFrameElementTypeId, Element};
use dom::element::AttributeHandlers;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom::window::Window;
use page::IterablePage;
use servo_msg::constellation_msg::{PipelineId, SubpageId};
use servo_msg::constellation_msg::{IFrameSandboxed, IFrameUnsandboxed};
use servo_msg::constellation_msg::{ConstellationChan, LoadIframeUrlMsg};
use servo_util::namespace::Null;
use servo_util::str::DOMString;
use servo_util::url::try_parse_url;

use std::ascii::StrAsciiExt;
use std::cell::Cell;
use url::Url;

enum SandboxAllowance {
    AllowNothing = 0x00,
    AllowSameOrigin = 0x01,
    AllowTopNavigation = 0x02,
    AllowForms = 0x04,
    AllowScripts = 0x08,
    AllowPointerLock = 0x10,
    AllowPopups = 0x20
}

#[deriving(Encodable)]
pub struct HTMLIFrameElement {
    pub htmlelement: HTMLElement,
    pub size: Traceable<Cell<Option<IFrameSize>>>,
    pub sandbox: Traceable<Cell<Option<u8>>>,
}

impl HTMLIFrameElementDerived for EventTarget {
    fn is_htmliframeelement(&self) -> bool {
       self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLIFrameElementTypeId))
    }
}

#[deriving(Encodable)]
pub struct IFrameSize {
    pub pipeline_id: PipelineId,
    pub subpage_id: SubpageId,
}

pub trait HTMLIFrameElementHelpers {
    fn is_sandboxed(&self) -> bool;
    fn get_url(&self) -> Option<Url>;
}

impl<'a> HTMLIFrameElementHelpers for JSRef<'a, HTMLIFrameElement> {
    fn is_sandboxed(&self) -> bool {
        self.sandbox.deref().get().is_some()
    }

    fn get_url(&self) -> Option<Url> {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_attribute(Null, "src").root().and_then(|src| {
            let window = window_from_node(self).root();
            try_parse_url(src.deref().value().as_slice(),
                          Some(window.deref().page().get_url())).ok()
        })
    }
}

impl HTMLIFrameElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLIFrameElement {
        HTMLIFrameElement {
            htmlelement: HTMLElement::new_inherited(HTMLIFrameElementTypeId, localName, document),
            size: Traceable::new(Cell::new(None)),
            sandbox: Traceable::new(Cell::new(None)),
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLIFrameElement> {
        let element = HTMLIFrameElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLIFrameElementBinding::Wrap)
    }
}

pub trait HTMLIFrameElementMethods {
    fn Src(&self) -> DOMString;
    fn SetSrc(&self, src: DOMString);
    fn Sandbox(&self) -> DOMString;
    fn SetSandbox(&self, sandbox: DOMString);
    fn GetContentWindow(&self) -> Option<Temporary<Window>>;
}

impl<'a> HTMLIFrameElementMethods for JSRef<'a, HTMLIFrameElement> {
    fn Src(&self) -> DOMString {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_string_attribute("src")
    }

    fn SetSrc(&self, src: DOMString) {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.set_url_attribute("src", src)
    }

    fn Sandbox(&self) -> DOMString {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_string_attribute("sandbox")
    }

    fn SetSandbox(&self, sandbox: DOMString) {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.set_string_attribute("sandbox", sandbox);
    }

    fn GetContentWindow(&self) -> Option<Temporary<Window>> {
        self.size.deref().get().and_then(|size| {
            let window = window_from_node(self).root();
            let children = &*window.deref().page.children.deref().borrow();
            let child = children.iter().find(|child| {
                child.subpage_id.unwrap() == size.subpage_id
            });
            child.and_then(|page| {
                page.frame.deref().borrow().as_ref().map(|frame| {
                    Temporary::new(frame.window.clone())
                })
            })
        })
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLIFrameElement> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods+> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_ref(self);
        Some(htmlelement as &VirtualMethods+)
    }

    fn after_set_attr(&self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(name.clone(), value.clone()),
            _ => (),
        }

        if "sandbox" == name.as_slice() {
            let mut modes = AllowNothing as u8;
            for word in value.as_slice().split(' ') {
                modes |= match word.to_ascii_lower().as_slice() {
                    "allow-same-origin" => AllowSameOrigin,
                    "allow-forms" => AllowForms,
                    "allow-pointer-lock" => AllowPointerLock,
                    "allow-popups" => AllowPopups,
                    "allow-scripts" => AllowScripts,
                    "allow-top-navigation" => AllowTopNavigation,
                    _ => AllowNothing
                } as u8;
            }
            self.deref().sandbox.deref().set(Some(modes));
        }
    }

    fn before_remove_attr(&self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(name.clone(), value),
            _ => (),
        }

        if "sandbox" == name.as_slice() {
            self.deref().sandbox.deref().set(None);
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        match self.super_type() {
            Some(ref s) => s.bind_to_tree(tree_in_doc),
            _ => (),
        }

        if !tree_in_doc { return; }

        match self.get_url() {
            Some(url) => {
                let sandboxed = if self.is_sandboxed() {
                    IFrameSandboxed
                } else {
                    IFrameUnsandboxed
                };

                // Subpage Id
                let window = window_from_node(self).root();
                let page = window.deref().page();
                let subpage_id = page.get_next_subpage_id();

                self.deref().size.deref().set(Some(IFrameSize {
                    pipeline_id: page.id,
                    subpage_id: subpage_id,
                }));

                let ConstellationChan(ref chan) = *page.constellation_chan.deref();
                chan.send(LoadIframeUrlMsg(url, page.id, subpage_id, sandboxed));
            }
            _ => ()
        }
    }
}

impl Reflectable for HTMLIFrameElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
