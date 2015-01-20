/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrValue};
use dom::attr::AttrHelpers;
use dom::bindings::codegen::Bindings::HTMLLinkElementBinding;
use dom::bindings::codegen::Bindings::HTMLLinkElementBinding::HTMLLinkElementMethods;
use dom::bindings::codegen::InheritTypes::HTMLLinkElementDerived;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast};
use dom::bindings::js::{MutNullableJS, JSRef, Temporary, OptionalRootable};
use dom::document::Document;
use dom::domtokenlist::DOMTokenList;
use dom::element::{AttributeHandlers, Element};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeHelpers, NodeTypeId, window_from_node};
use dom::virtualmethods::VirtualMethods;
use layout_interface::{LayoutChan, Msg};
use servo_util::str::{DOMString, HTML_SPACE_CHARACTERS};

use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::default::Default;
use url::UrlParser;
use string_cache::Atom;

#[dom_struct]
pub struct HTMLLinkElement {
    htmlelement: HTMLElement,
    rel_list: MutNullableJS<DOMTokenList>,
}

impl HTMLLinkElementDerived for EventTarget {
    fn is_htmllinkelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLinkElement)))
    }
}

impl HTMLLinkElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLLinkElement {
        HTMLLinkElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLLinkElement, localName, prefix, document),
            rel_list: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLLinkElement> {
        let element = HTMLLinkElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLLinkElementBinding::Wrap)
    }
}

fn get_attr(element: JSRef<Element>, name: &Atom) -> Option<String> {
    let elem = element.get_attribute(ns!(""), name).root();
    elem.map(|e| e.r().value().as_slice().to_owned())
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

    fn after_set_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(attr),
            _ => ()
        }

        let element: JSRef<Element> = ElementCast::from_ref(*self);
        let rel = get_attr(element, &atom!("rel"));

        match (rel, attr.local_name()) {
            (ref rel, &atom!("href")) => {
                if is_stylesheet(rel) {
                    self.handle_stylesheet_url(attr.value().as_slice());
                }
            }
            (_, _) => ()
        }
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("rel") => AttrValue::from_serialized_tokenlist(value),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        match self.super_type() {
            Some(ref s) => s.bind_to_tree(tree_in_doc),
            _ => ()
        }

        if tree_in_doc {
            let element: JSRef<Element> = ElementCast::from_ref(*self);

            let rel = get_attr(element, &atom!("rel"));
            let href = get_attr(element, &atom!("href"));

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
        let window = window.r();
        match UrlParser::new().base_url(&window.page().get_url()).parse(href) {
            Ok(url) => {
                let LayoutChan(ref layout_chan) = window.page().layout_chan;
                layout_chan.send(Msg::LoadStylesheet(url));
            }
            Err(e) => debug!("Parsing url {} failed: {}", href, e)
        }
    }
}

impl<'a> HTMLLinkElementMethods for JSRef<'a, HTMLLinkElement> {
    make_url_getter!(Href)
    make_setter!(SetHref, "href")

    make_getter!(Rel)
    make_setter!(SetRel, "rel")

    make_getter!(Media)
    make_setter!(SetMedia, "media")

    make_getter!(Hreflang)
    make_setter!(SetHreflang, "hreflang")

    make_getter!(Type)
    make_setter!(SetType, "type")

    fn RelList(self) -> Temporary<DOMTokenList> {
        self.rel_list.or_init(|| {
            DOMTokenList::new(ElementCast::from_ref(self), &atom!("rel"))
        })
    }
}
