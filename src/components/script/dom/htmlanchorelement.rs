/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLAnchorElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLAnchorElementDerived;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast, NodeCast};
use dom::bindings::js::{JSRef, Temporary, OptionalRootable};
use dom::bindings::error::ErrorResult;
use dom::document::{Document, DocumentHelpers};
use dom::attr::AttrMethods;
use dom::element::{Element, AttributeHandlers, HTMLAnchorElementTypeId};
use dom::event::{Event, EventMethods};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeHelpers, ElementNodeTypeId};
use dom::virtualmethods::VirtualMethods;
use servo_util::namespace::Null;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLAnchorElement {
    pub htmlelement: HTMLElement
}

impl HTMLAnchorElementDerived for EventTarget {
    fn is_htmlanchorelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLAnchorElementTypeId))
    }
}

impl HTMLAnchorElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLAnchorElement {
        HTMLAnchorElement {
            htmlelement: HTMLElement::new_inherited(HTMLAnchorElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLAnchorElement> {
        let element = HTMLAnchorElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLAnchorElementBinding::Wrap)
    }
}

pub trait HTMLAnchorElementMethods {
    fn Href(&self) -> DOMString;
    fn SetHref(&self, _href: DOMString) -> ErrorResult;
    fn Target(&self) -> DOMString;
    fn SetTarget(&self, _target: DOMString) -> ErrorResult;
    fn Download(&self) -> DOMString;
    fn SetDownload(&self, _download: DOMString) -> ErrorResult;
    fn Ping(&self) -> DOMString;
    fn SetPing(&self, _ping: DOMString) -> ErrorResult;
    fn Rel(&self) -> DOMString;
    fn SetRel(&self, _rel: DOMString) -> ErrorResult;
    fn Hreflang(&self) -> DOMString;
    fn SetHreflang(&self, _href_lang: DOMString) -> ErrorResult;
    fn Type(&self) -> DOMString;
    fn SetType(&self, _type: DOMString) -> ErrorResult;
    fn Text(&self) -> DOMString;
    fn SetText(&self, _text: DOMString) -> ErrorResult;
    fn Coords(&self) -> DOMString;
    fn SetCoords(&self, _coords: DOMString) -> ErrorResult;
    fn Charset(&self) -> DOMString;
    fn SetCharset(&self, _charset: DOMString) -> ErrorResult;
    fn Name(&self) -> DOMString;
    fn SetName(&self, _name: DOMString) -> ErrorResult;
    fn Rev(&self) -> DOMString;
    fn SetRev(&self, _rev: DOMString) -> ErrorResult;
    fn Shape(&self) -> DOMString;
    fn SetShape(&self, _shape: DOMString) -> ErrorResult;
}

impl<'a> HTMLAnchorElementMethods for JSRef<'a, HTMLAnchorElement> {
    fn Href(&self) -> DOMString {
        "".to_owned()
    }

    fn SetHref(&self, _href: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Target(&self) -> DOMString {
        "".to_owned()
    }

    fn SetTarget(&self, _target: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Download(&self) -> DOMString {
        "".to_owned()
    }

    fn SetDownload(&self, _download: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Ping(&self) -> DOMString {
        "".to_owned()
    }

    fn SetPing(&self, _ping: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Rel(&self) -> DOMString {
        "".to_owned()
    }

    fn SetRel(&self, _rel: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Hreflang(&self) -> DOMString {
        "".to_owned()
    }

    fn SetHreflang(&self, _href_lang: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Type(&self) -> DOMString {
        "".to_owned()
    }

    fn SetType(&self, _type: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Text(&self) -> DOMString {
        "".to_owned()
    }

    fn SetText(&self, _text: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Coords(&self) -> DOMString {
        "".to_owned()
    }

    fn SetCoords(&self, _coords: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Charset(&self) -> DOMString {
        "".to_owned()
    }

    fn SetCharset(&self, _charset: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Name(&self) -> DOMString {
        "".to_owned()
    }

    fn SetName(&self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Rev(&self) -> DOMString {
        "".to_owned()
    }

    fn SetRev(&self, _rev: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Shape(&self) -> DOMString {
        "".to_owned()
    }

    fn SetShape(&self, _shape: DOMString) -> ErrorResult {
        Ok(())
    }
}

trait PrivateHTMLAnchorElementHelpers {
    fn handle_event_impl(&self, event: &JSRef<Event>);
}

impl<'a> PrivateHTMLAnchorElementHelpers for JSRef<'a, HTMLAnchorElement> {
    fn handle_event_impl(&self, event: &JSRef<Event>) {
        if "click" == event.Type() && !event.DefaultPrevented() {
            let element: &JSRef<Element> = ElementCast::from_ref(self);
            let attr = element.get_attribute(Null, "href").root();
            match attr {
                Some(ref href) => {
                    let value = href.Value();
                    debug!("clicked on link to {:s}", value);
                    let node: &JSRef<Node> = NodeCast::from_ref(self);
                    let mut doc = node.owner_doc().root();
                    doc.load_anchor_href(value);
                }
                None => ()
            }
        }
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLAnchorElement> {
    fn super_type<'a>(&'a mut self) -> Option<&'a mut VirtualMethods:> {
        let htmlelement: &mut JSRef<HTMLElement> = HTMLElementCast::from_mut_ref(self);
        Some(htmlelement as &mut VirtualMethods:)
    }

    fn handle_event(&mut self, event: &JSRef<Event>) {
        match self.super_type() {
            Some(s) => {
                s.handle_event(event);
            }
            None => {}
        }
        self.handle_event_impl(event);
    }
}
