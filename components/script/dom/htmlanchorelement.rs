/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


use dom::activation::Activatable;
use dom::attr::AttrValue;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::HTMLAnchorElementBinding;
use dom::bindings::codegen::Bindings::HTMLAnchorElementBinding::HTMLAnchorElementMethods;
use dom::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLAnchorElementDerived, HTMLImageElementDerived};
use dom::bindings::codegen::InheritTypes::{MouseEventCast, NodeCast};
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::document::Document;
use dom::domtokenlist::DOMTokenList;
use dom::element::{Element, ElementTypeId};
use dom::event::Event;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId, document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;

use util::str::DOMString;

use num::ToPrimitive;
use string_cache::Atom;
use url::UrlParser;

use std::default::Default;

#[dom_struct]
pub struct HTMLAnchorElement {
    htmlelement: HTMLElement,
    rel_list: MutNullableHeap<JS<DOMTokenList>>,
}

impl HTMLAnchorElementDerived for EventTarget {
    fn is_htmlanchorelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAnchorElement)))
    }
}

impl HTMLAnchorElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLAnchorElement {
        HTMLAnchorElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLAnchorElement, localName, prefix, document),
            rel_list: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLAnchorElement> {
        let element = HTMLAnchorElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLAnchorElementBinding::Wrap)
    }
}

impl VirtualMethods for HTMLAnchorElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &HTMLElement = HTMLElementCast::from_ref(self);
        Some(htmlelement as &VirtualMethods)
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
        let node = NodeCast::from_ref(self);
        node.GetTextContent().unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-a-text
    fn SetText(&self, value: DOMString) {
        let node = NodeCast::from_ref(self);
        node.SetTextContent(Some(value))
    }

    // https://html.spec.whatwg.org/multipage/#dom-a-rellist
    fn RelList(&self) -> Root<DOMTokenList> {
        self.rel_list.or_init(|| {
            DOMTokenList::new(ElementCast::from_ref(self), &atom!("rel"))
        })
    }
}

impl Activatable for HTMLAnchorElement {
    fn as_element<'b>(&'b self) -> &'b Element {
        ElementCast::from_ref(self)
    }

    fn is_instance_activatable(&self) -> bool {
        // https://html.spec.whatwg.org/multipage/#hyperlink
        // "a [...] element[s] with an href attribute [...] must [..] create a
        // hyperlink"
        // https://html.spec.whatwg.org/multipage/#the-a-element
        // "The activation behaviour of a elements *that create hyperlinks*"
        ElementCast::from_ref(self).has_attribute(&atom!("href"))
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
        if !doc.r().is_fully_active() {
            return;
        }
        //TODO: Step 2. Check if browsing context is specified and act accordingly.
        //Step 3. Handle <img ismap/>.
        let element = ElementCast::from_ref(self);
        let mouse_event = MouseEventCast::to_ref(event).unwrap();
        let mut ismap_suffix = None;
        if let Some(element) = ElementCast::to_ref(target) {
            if target.is_htmlimageelement() && element.has_attribute(&atom!("ismap")) {

                let target_node = NodeCast::from_ref(element);
                let rect = window_from_node(target_node).r().content_box_query(
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
fn follow_hyperlink(subject: &Element, hyperlink_suffix: Option<DOMString>) {
    // Step 1: replace.
    // Step 2: source browsing context.
    // Step 3: target browsing context.

    // Step 4.
    let attribute = subject.get_attribute(&ns!(""), &atom!("href")).unwrap();
    let mut href = attribute.Value();

    // Step 6.
    // https://www.w3.org/Bugs/Public/show_bug.cgi?id=28925
    if let Some(suffix) = hyperlink_suffix {
        href.push_str(&suffix);
    }

    // Step 4-5.
    let document = document_from_node(subject);
    let url = match UrlParser::new().base_url(&document.url()).parse(&href) {
        Ok(url) => url,
        Err(_) => return,
    };

    // Step 7.
    debug!("following hyperlink to {}", url.serialize());
    let window = document.window();
    window.load_url(url);
}
