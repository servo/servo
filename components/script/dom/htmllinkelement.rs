/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::AttrHelpers;
use dom::bindings::codegen::Bindings::HTMLLinkElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLLinkElementDerived;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast};
use dom::bindings::js::{JSRef, Temporary, OptionalRootable};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::{AttributeHandlers, Element, HTMLLinkElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeHelpers, ElementNodeTypeId, window_from_node};
use dom::virtualmethods::VirtualMethods;
use layout_interface::{LayoutChan, LoadStylesheetMsg};
use servo_util::str::{DOMString, HTML_SPACE_CHARACTERS};

use std::ascii::StrAsciiExt;
use url::UrlParser;
use string_cache::Atom;

#[jstraceable]
#[must_root]
pub struct HTMLLinkElement {
    pub htmlelement: HTMLElement,
}

impl HTMLLinkElementDerived for EventTarget {
    fn is_htmllinkelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLLinkElementTypeId))
    }
}

impl HTMLLinkElement {
    fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLLinkElement {
        HTMLLinkElement {
            htmlelement: HTMLElement::new_inherited(HTMLLinkElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLLinkElement> {
        let element = HTMLLinkElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLLinkElementBinding::Wrap)
    }
}

fn get_attr(element: JSRef<Element>, name: &str) -> Option<String> {
    let elem = element.get_attribute(ns!(""), name).root();
    elem.map(|e| e.deref().value().as_slice().to_string())
}

fn is_stylesheet(value: &Option<String>) -> bool {
    match *value {
        Some(ref value) => {
            value.as_slice().split(HTML_SPACE_CHARACTERS.as_slice())
                .any(|s| s.as_slice().eq_ignore_ascii_case("stylesheet"))
        },
        None => false,
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLLinkElement> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, name: &Atom, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(name, value.clone()),
            _ => (),
        }

        let element: JSRef<Element> = ElementCast::from_ref(*self);
        let rel = get_attr(element, "rel");

        match (rel, name.as_slice()) {
            (ref rel, "href") => {
                if is_stylesheet(rel) {
                    self.handle_stylesheet_url(value.as_slice());
                }
            }
            (_, _) => ()
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        match self.super_type() {
            Some(ref s) => s.bind_to_tree(tree_in_doc),
            _ => ()
        }

        if tree_in_doc {
            let element: JSRef<Element> = ElementCast::from_ref(*self);

            let rel = get_attr(element, "rel");
            let href = get_attr(element, "href");

            match (rel, href) {
                (ref rel, Some(ref href)) if is_stylesheet(rel) => {
                    self.handle_stylesheet_url(href.as_slice());
                }
                _ => {}
            }
        }
    }
}

trait PrivateHTMLLinkElementHelpers {
    fn handle_stylesheet_url(self, href: &str);
}

impl<'a> PrivateHTMLLinkElementHelpers for JSRef<'a, HTMLLinkElement> {
    fn handle_stylesheet_url(self, href: &str) {
        let window = window_from_node(self).root();
        match UrlParser::new().base_url(&window.deref().page().get_url()).parse(href) {
            Ok(url) => {
                let LayoutChan(ref layout_chan) = *window.deref().page().layout_chan;
                layout_chan.send(LoadStylesheetMsg(url));
            }
            Err(e) => debug!("Parsing url {:s} failed: {:?}", href, e)
        }
    }
}

impl Reflectable for HTMLLinkElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}

