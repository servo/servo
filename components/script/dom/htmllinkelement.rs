/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser as CssParser;
use document_loader::LoadType;
use dom::attr::AttrHelpers;
use dom::attr::{Attr, AttrValue};
use dom::bindings::codegen::Bindings::HTMLLinkElementBinding;
use dom::bindings::codegen::Bindings::HTMLLinkElementBinding::HTMLLinkElementMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{EventTargetCast, HTMLLinkElementDerived};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::js::{RootedReference};
use dom::bindings::refcounted::Trusted;
use dom::document::{Document, DocumentHelpers};
use dom::domtokenlist::DOMTokenList;
use dom::element::ElementTypeId;
use dom::element::{AttributeHandlers, Element};
use dom::event::{EventBubbles, EventCancelable, Event, EventHelpers};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeHelpers, NodeTypeId, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom::window::WindowHelpers;
use layout_interface::{LayoutChan, Msg};
use msg::constellation_msg::ConstellationChan;
use msg::constellation_msg::Msg as ConstellationMsg;
use script_traits::StylesheetLoadResponder;
use style::media_queries::parse_media_query_list;
use util::str::{DOMString, HTML_SPACE_CHARACTERS};

use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::default::Default;
use string_cache::Atom;
use url::UrlParser;

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct HTMLLinkElement {
    htmlelement: HTMLElement,
    rel_list: MutNullableHeap<JS<DOMTokenList>>,
}

impl HTMLLinkElementDerived for EventTarget {
    fn is_htmllinkelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLinkElement)))
    }
}

impl HTMLLinkElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document) -> HTMLLinkElement {
        HTMLLinkElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLLinkElement, localName, prefix, document),
            rel_list: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLLinkElement> {
        let element = HTMLLinkElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLLinkElementBinding::Wrap)
    }
}

fn get_attr(element: &Element, local_name: &Atom) -> Option<String> {
    let elem = element.get_attribute(&ns!(""), local_name);
    elem.r().map(|e| {
        let value = e.value();
        (**value).to_owned()
    })
}

fn is_stylesheet(value: &Option<String>) -> bool {
    match *value {
        Some(ref value) => {
            value.split(HTML_SPACE_CHARACTERS)
                .any(|s| s.eq_ignore_ascii_case("stylesheet"))
        },
        None => false,
    }
}

/// Favicon spec usage in accordance with CEF implementation:
/// only url of icon is required/used
/// https://html.spec.whatwg.org/multipage/#rel-icon
fn is_favicon(value: &Option<String>) -> bool {
    match *value {
        Some(ref value) => {
            value.split(HTML_SPACE_CHARACTERS)
                .any(|s| s.eq_ignore_ascii_case("icon"))
        },
        None => false,
    }
}

impl<'a> VirtualMethods for &'a HTMLLinkElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &&HTMLElement = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.after_set_attr(attr);
        }

        let node = NodeCast::from_ref(*self);
        if !node.is_in_doc() {
            return;
        }

        let element = ElementCast::from_ref(*self);
        let rel = get_attr(element, &atom!("rel"));

        match (rel, attr.local_name()) {
            (ref rel, &atom!("href")) => {
                if is_stylesheet(rel) {
                    self.handle_stylesheet_url(&attr.value());
                } else if is_favicon(rel) {
                    self.handle_favicon_url(&attr.value());
                }
            }
            (ref rel, &atom!("media")) => {
                if is_stylesheet(rel) {
                    self.handle_stylesheet_url(&attr.value());
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
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        if tree_in_doc {
            let element = ElementCast::from_ref(*self);

            let rel = get_attr(element, &atom!("rel"));
            let href = get_attr(element, &atom!("href"));

            match (rel, href) {
                (ref rel, Some(ref href)) if is_stylesheet(rel) => {
                    self.handle_stylesheet_url(href);
                }
                (ref rel, Some(ref href)) if is_favicon(rel) => {
                    self.handle_favicon_url(href);
                }
                _ => {}
            }
        }
    }
}

trait PrivateHTMLLinkElementHelpers {
    fn handle_stylesheet_url(self, href: &str);
    fn handle_favicon_url(self, href: &str);
}

impl<'a> PrivateHTMLLinkElementHelpers for &'a HTMLLinkElement {
    fn handle_stylesheet_url(self, href: &str) {
        let window = window_from_node(self);
        let window = window.r();
        match UrlParser::new().base_url(&window.get_url()).parse(href) {
            Ok(url) => {
                let element = ElementCast::from_ref(self);

                let mq_attribute = element.get_attribute(&ns!(""), &atom!("media"));
                let value = mq_attribute.r().map(|a| a.value());
                let mq_str = match value {
                    Some(ref value) => &***value,
                    None => "",
                };
                let mut css_parser = CssParser::new(&mq_str);
                let media = parse_media_query_list(&mut css_parser);

                let doc = window.Document();
                let link_element = Trusted::new(window.get_cx(), self, window.script_chan().clone());
                let load_dispatcher = StylesheetLoadDispatcher::new(link_element);

                let pending = doc.r().prepare_async_load(LoadType::Stylesheet(url.clone()));
                let LayoutChan(ref layout_chan) = window.layout_chan();
                layout_chan.send(Msg::LoadStylesheet(url, media, pending, box load_dispatcher)).unwrap();
            }
            Err(e) => debug!("Parsing url {} failed: {}", href, e)
        }
    }

    fn handle_favicon_url(self, href: &str) {
        let window = window_from_node(self);
        let window = window.r();
        match UrlParser::new().base_url(&window.get_url()).parse(href) {
            Ok(url) => {
                let ConstellationChan(ref chan) = window.constellation_chan();
                let event = ConstellationMsg::NewFavicon(url.clone());
                chan.send(event).unwrap();
            }
            Err(e) => debug!("Parsing url {} failed: {}", href, e)
        }
    }
}

impl<'a> HTMLLinkElementMethods for &'a HTMLLinkElement {
    make_url_getter!(Href);
    make_setter!(SetHref, "href");

    make_getter!(Rel);
    make_setter!(SetRel, "rel");

    make_getter!(Media);
    make_setter!(SetMedia, "media");

    make_getter!(Hreflang);
    make_setter!(SetHreflang, "hreflang");

    make_getter!(Type);
    make_setter!(SetType, "type");

    // https://html.spec.whatwg.org/multipage/#dom-link-rellist
    fn RelList(self) -> Root<DOMTokenList> {
        self.rel_list.or_init(|| {
            DOMTokenList::new(ElementCast::from_ref(self), &atom!("rel"))
        })
    }
}

pub struct StylesheetLoadDispatcher {
    elem: Trusted<HTMLLinkElement>,
}

impl StylesheetLoadDispatcher {
    pub fn new(elem: Trusted<HTMLLinkElement>) -> StylesheetLoadDispatcher {
        StylesheetLoadDispatcher {
            elem: elem,
        }
    }
}

impl StylesheetLoadResponder for StylesheetLoadDispatcher {
    fn respond(self: Box<StylesheetLoadDispatcher>) {
        let elem = self.elem.root();
        let window = window_from_node(elem.r());
        let event = Event::new(GlobalRef::Window(window.r()), "load".to_owned(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable);
        let target = EventTargetCast::from_ref(elem.r());
        event.r().fire(target);
    }
}
