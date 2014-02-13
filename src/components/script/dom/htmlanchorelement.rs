/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLAnchorElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLAnchorElementDerived;
use dom::bindings::js::JS;
use dom::bindings::utils::ErrorResult;
use dom::document::Document;
use dom::element::HTMLAnchorElementTypeId;
use dom::event::Event;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use dom::virtualmethods::VirtualMethods;
use servo_util::namespace::Null;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLAnchorElement {
    htmlelement: HTMLElement
}

impl HTMLAnchorElementDerived for EventTarget {
    fn is_htmlanchorelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLAnchorElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLAnchorElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLAnchorElement {
        HTMLAnchorElement {
            htmlelement: HTMLElement::new_inherited(HTMLAnchorElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLAnchorElement> {
        let element = HTMLAnchorElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLAnchorElementBinding::Wrap)
    }
}

impl HTMLAnchorElement {
    pub fn Href(&self) -> DOMString {
        ~""
    }

    pub fn SetHref(&mut self, _href: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Target(&self) -> DOMString {
        ~""
    }

    pub fn SetTarget(&self, _target: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Download(&self) -> DOMString {
        ~""
    }

    pub fn SetDownload(&self, _download: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Ping(&self) -> DOMString {
        ~""
    }

    pub fn SetPing(&self, _ping: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Rel(&self) -> DOMString {
        ~""
    }

    pub fn SetRel(&self, _rel: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Hreflang(&self) -> DOMString {
        ~""
    }

    pub fn SetHreflang(&self, _href_lang: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        ~""
    }

    pub fn SetType(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Text(&self) -> DOMString {
        ~""
    }

    pub fn SetText(&mut self, _text: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Coords(&self) -> DOMString {
        ~""
    }

    pub fn SetCoords(&mut self, _coords: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Charset(&self) -> DOMString {
        ~""
    }

    pub fn SetCharset(&mut self, _charset: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Name(&self) -> DOMString {
        ~""
    }

    pub fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Rev(&self) -> DOMString {
        ~""
    }

    pub fn SetRev(&mut self, _rev: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Shape(&self) -> DOMString {
        ~""
    }

    pub fn SetShape(&mut self, _shape: DOMString) -> ErrorResult {
        Ok(())
    }
}

impl HTMLAnchorElement {
    fn handle_event_impl(&mut self, abstract_self: &JS<Node>, event: &JS<Event>) {
        let event = event.get();
        if "click" == event.Type() && !event.DefaultPrevented() {
            let attr = self.htmlelement.element.get_attribute(Null, "href");
            for href in attr.iter() {
                let value = href.get().Value();
                debug!("clicked on link to {:s}", value);
                let doc = abstract_self.get().owner_doc();
                doc.get().load_anchor_href(value);
            }
        }
    }
}

impl VirtualMethods for HTMLAnchorElement {
    fn super_type<'a>(&'a mut self) -> Option<&'a mut VirtualMethods> {
        Some(&mut self.htmlelement as &mut VirtualMethods)
    }

    fn handle_event(&mut self, abstract_self: &JS<Node>, event: &JS<Event>) {
        self.super_type().map(|s| s.handle_event(abstract_self, event));
        self.handle_event_impl(abstract_self, event);
    }
}
