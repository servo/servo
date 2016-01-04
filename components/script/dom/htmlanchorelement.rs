/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use dom::activation::Activatable;
use dom::attr::AttrValue;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::HTMLAnchorElementBinding;
use dom::bindings::codegen::Bindings::HTMLAnchorElementBinding::HTMLAnchorElementMethods;
use dom::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::str::USVString;
use dom::document::Document;
use dom::domtokenlist::DOMTokenList;
use dom::element::Element;
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::mouseevent::MouseEvent;
use dom::node::{Node, document_from_node, window_from_node};
use dom::urlhelper::UrlHelper;
use dom::virtualmethods::VirtualMethods;
use num::ToPrimitive;
use std::default::Default;
use string_cache::Atom;
use url::Url;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLAnchorElement {
    htmlelement: HTMLElement,
    rel_list: MutNullableHeap<JS<DOMTokenList>>,
    url: DOMRefCell<Option<Url>>,
}

impl HTMLAnchorElement {
    fn new_inherited(localName: Atom,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLAnchorElement {
        HTMLAnchorElement {
            htmlelement:
                HTMLElement::new_inherited(localName, prefix, document),
            rel_list: Default::default(),
            url: DOMRefCell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLAnchorElement> {
        let element = HTMLAnchorElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLAnchorElementBinding::Wrap)
    }

    // https://html.spec.whatwg.org/multipage/#concept-hyperlink-url-set
    fn set_url(&self) {
        let attribute = self.upcast::<Element>().get_attribute(&ns!(), &atom!("href"));
        *self.url.borrow_mut() = attribute.and_then(|attribute| {
            Url::parse(&attribute.value()).ok()
        });
    }

    // https://html.spec.whatwg.org/multipage/#reinitialise-url
    fn reinitialize_url(&self) {
        // Step 1.
        match *self.url.borrow() {
            None => return,
            Some(ref url) if url.non_relative_scheme_data().is_some() => return,
            _ => (),
        }

        // Step 2.
        self.set_url();
    }

    // https://html.spec.whatwg.org/multipage/#update-href
    fn update_href(&self) {
        self.upcast::<Element>().set_string_attribute(&atom!("href"),
            self.url.borrow().as_ref().unwrap().serialize().into());
    }
}

impl VirtualMethods for HTMLAnchorElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("rel") => AttrValue::from_serialized_tokenlist(value),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}

impl HTMLAnchorElementMethods for HTMLAnchorElement {
    // https://html.spec.whatwg.org/multipage/#dom-a-text
    fn Text(&self) -> DOMString {
        self.upcast::<Node>().GetTextContent().unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-a-text
    fn SetText(&self, value: DOMString) {
        self.upcast::<Node>().SetTextContent(Some(value))
    }

    // https://html.spec.whatwg.org/multipage/#dom-a-rellist
    fn RelList(&self) -> Root<DOMTokenList> {
        self.rel_list.or_init(|| {
            DOMTokenList::new(self.upcast(), &atom!("rel"))
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-a-coords
    make_getter!(Coords, "coords");

    // https://html.spec.whatwg.org/multipage/#dom-a-coords
    make_setter!(SetCoords, "coords");

    // https://html.spec.whatwg.org/multipage/#dom-a-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-a-name
    make_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-a-rev
    make_getter!(Rev, "rev");

    // https://html.spec.whatwg.org/multipage/#dom-a-rev
    make_setter!(SetRev, "rev");

    // https://html.spec.whatwg.org/multipage/#dom-a-shape
    make_getter!(Shape, "shape");

    // https://html.spec.whatwg.org/multipage/#dom-a-shape
    make_setter!(SetShape, "shape");

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-hash
    fn Hash(&self) -> USVString {
        UrlHelper::Hash(&self.url.borrow().as_ref().unwrap())
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-hash
    fn SetHash(&self, value: USVString) {
        UrlHelper::SetHash(self.url.borrow_mut().as_mut().unwrap(), value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-host
    fn Host(&self) -> USVString {
        UrlHelper::Host(&self.url.borrow().as_ref().unwrap())
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-host
    fn SetHost(&self, value: USVString) {
        UrlHelper::SetHost(self.url.borrow_mut().as_mut().unwrap(), value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-hostname
    fn Hostname(&self) -> USVString {
        UrlHelper::Hostname(&self.url.borrow().as_ref().unwrap())
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-hostname
    fn SetHostname(&self, value: USVString) {
        UrlHelper::SetHostname(self.url.borrow_mut().as_mut().unwrap(), value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-href
    fn Href(&self) -> USVString {
        // Step 1.
        self.reinitialize_url();

        USVString(match *self.url.borrow() {
            None => {
                match self.upcast::<Element>().get_attribute(&ns!(), &atom!("href")) {
                    // Step 3.
                    None => String::new(),
                    // Step 4.
                    Some(attribute) => (**attribute.value()).to_owned(),
                }
            },
            // Step 5.
            Some(ref url) => url.serialize(),
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-href
    fn SetHref(&self, value: USVString) {
        unimplemented!()
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-password
    fn Password(&self) -> USVString {
        UrlHelper::Password(&self.url.borrow().as_ref().unwrap())
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-password
    fn SetPassword(&self, value: USVString) {
        UrlHelper::SetPassword(self.url.borrow_mut().as_mut().unwrap(), value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-pathname
    fn Pathname(&self) -> USVString {
        UrlHelper::Pathname(&self.url.borrow().as_ref().unwrap())
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-pathname
    fn SetPathname(&self, value: USVString) {
        UrlHelper::SetPathname(self.url.borrow_mut().as_mut().unwrap(), value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-port
    fn Port(&self) -> USVString {
        UrlHelper::Port(&self.url.borrow().as_ref().unwrap())
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-port
    fn SetPort(&self, value: USVString) {
        UrlHelper::SetPort(self.url.borrow_mut().as_mut().unwrap(), value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-protocol
    fn Protocol(&self) -> USVString {
        UrlHelper::Protocol(&self.url.borrow().as_ref().unwrap())
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-protocol
    fn SetProtocol(&self, value: USVString) {
        UrlHelper::SetProtocol(self.url.borrow_mut().as_mut().unwrap(), value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-search
    fn Search(&self) -> USVString {
        UrlHelper::Search(&self.url.borrow().as_ref().unwrap())
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-search
    fn SetSearch(&self, value: USVString) {
        UrlHelper::SetSearch(self.url.borrow_mut().as_mut().unwrap(), value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-username
    fn Username(&self) -> USVString {
        UrlHelper::Username(&self.url.borrow().as_ref().unwrap())
    }

    // https://html.spec.whatwg.org/multipage/#dom-hyperlink-username
    fn SetUsername(&self, value: USVString) {
        UrlHelper::SetUsername(self.url.borrow_mut().as_mut().unwrap(), value);
    }
}

impl Activatable for HTMLAnchorElement {
    fn as_element(&self) -> &Element {
        self.upcast::<Element>()
    }

    fn is_instance_activatable(&self) -> bool {
        // https://html.spec.whatwg.org/multipage/#hyperlink
        // "a [...] element[s] with an href attribute [...] must [..] create a
        // hyperlink"
        // https://html.spec.whatwg.org/multipage/#the-a-element
        // "The activation behaviour of a elements *that create hyperlinks*"
        self.upcast::<Element>().has_attribute(&atom!("href"))
    }


    //TODO:https://html.spec.whatwg.org/multipage/#the-a-element
    fn pre_click_activation(&self) {
    }

    //TODO:https://html.spec.whatwg.org/multipage/#the-a-element
    // https://html.spec.whatwg.org/multipage/#run-canceled-activation-steps
    fn canceled_activation(&self) {
    }

    //https://html.spec.whatwg.org/multipage/#the-a-element:activation-behaviour
    fn activation_behavior(&self, event: &Event, target: &EventTarget) {
        //Step 1. If the node document is not fully active, abort.
        let doc = document_from_node(self);
        if !doc.is_fully_active() {
            return;
        }
        //TODO: Step 2. Check if browsing context is specified and act accordingly.
        //Step 3. Handle <img ismap/>.
        let element = self.upcast::<Element>();
        let mouse_event = event.downcast::<MouseEvent>().unwrap();
        let mut ismap_suffix = None;
        if let Some(element) = target.downcast::<Element>() {
            if target.is::<HTMLImageElement>() && element.has_attribute(&atom!("ismap")) {

                let target_node = element.upcast::<Node>();
                let rect = window_from_node(target_node).content_box_query(
                    target_node.to_trusted_node_address());
                ismap_suffix = Some(
                    format!("?{},{}", mouse_event.ClientX().to_f32().unwrap() - rect.origin.x.to_f32_px(),
                                      mouse_event.ClientY().to_f32().unwrap() - rect.origin.y.to_f32_px())
                )
            }
        }

        // Step 4.
        //TODO: Download the link is `download` attribute is set.
        follow_hyperlink(element, ismap_suffix);
    }

    //TODO:https://html.spec.whatwg.org/multipage/#the-a-element
    fn implicit_submission(&self, _ctrlKey: bool, _shiftKey: bool, _altKey: bool, _metaKey: bool) {
    }
}

/// https://html.spec.whatwg.org/multipage/#following-hyperlinks-2
fn follow_hyperlink(subject: &Element, hyperlink_suffix: Option<String>) {
    // Step 1: replace.
    // Step 2: source browsing context.
    // Step 3: target browsing context.

    // Step 4.
    let attribute = subject.get_attribute(&ns!(), &atom!("href")).unwrap();
    let mut href = attribute.Value();

    // Step 6.
    // https://www.w3.org/Bugs/Public/show_bug.cgi?id=28925
    if let Some(suffix) = hyperlink_suffix {
        href.push_str(&suffix);
    }

    // Step 4-5.
    let document = document_from_node(subject);
    let url = match document.url().join(&href) {
        Ok(url) => url,
        Err(_) => return,
    };

    // Step 7.
    debug!("following hyperlink to {}", url.serialize());
    let window = document.window();
    window.load_url(url);
}
