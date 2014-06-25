/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::AttrMethods;
use dom::bindings::codegen::Bindings::HTMLObjectElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLObjectElementDerived;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast};
use dom::bindings::js::{JSRef, Temporary};
use dom::document::Document;
use dom::element::{Element, HTMLObjectElementTypeId};
use dom::element::AttributeHandlers;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId, NodeHelpers, window_from_node};
use dom::validitystate::ValidityState;
use dom::virtualmethods::VirtualMethods;
use servo_util::str::DOMString;

use servo_net::image_cache_task;
use servo_net::image_cache_task::ImageCacheTask;
use servo_util::url::parse_url;
use servo_util::namespace::Null;
use servo_util::url::is_image_data;
use url::Url;

#[deriving(Encodable)]
pub struct HTMLObjectElement {
    pub htmlelement: HTMLElement,
}

impl HTMLObjectElementDerived for EventTarget {
    fn is_htmlobjectelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLObjectElementTypeId))
    }
}

impl HTMLObjectElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLObjectElement {
        HTMLObjectElement {
            htmlelement: HTMLElement::new_inherited(HTMLObjectElementTypeId, localName, document),
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLObjectElement> {
        let element = HTMLObjectElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLObjectElementBinding::Wrap)
    }
}

trait ProcessDataURL {
    fn process_data_url(&self, image_cache: ImageCacheTask, url: Option<Url>);
}

impl<'a> ProcessDataURL for JSRef<'a, HTMLObjectElement> {
    // Makes the local `data` member match the status of the `data` attribute and starts
    /// prefetching the image. This method must be called after `data` is changed.
    fn process_data_url(&self, image_cache: ImageCacheTask, url: Option<Url>) {
        let elem: &JSRef<Element> = ElementCast::from_ref(self);

        // TODO: support other values
        match (elem.get_attribute(Null, "type").map(|x| x.root().Value()),
               elem.get_attribute(Null, "data").map(|x| x.root().Value())) {
            (None, Some(uri)) => {
                if is_image_data(uri.as_slice()) {
                    let data_url = parse_url(uri.as_slice(), url);
                    // Issue #84
                    image_cache.send(image_cache_task::Prefetch(data_url));
                }
            }
            _ => { }
        }
    }
}

pub trait HTMLObjectElementMethods {
    fn Validity(&self) -> Temporary<ValidityState>;
}

impl<'a> HTMLObjectElementMethods for JSRef<'a, HTMLObjectElement> {
    fn Validity(&self) -> Temporary<ValidityState> {
        let window = window_from_node(self).root();
        ValidityState::new(&*window)
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLObjectElement> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods+> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_ref(self);
        Some(htmlelement as &VirtualMethods+)
    }

    fn after_set_attr(&self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(name.clone(), value),
            _ => (),
        }

        if "data" == name.as_slice() {
            let window = window_from_node(self).root();
            let url = Some(window.deref().get_url());
            self.process_data_url(window.deref().image_cache_task.clone(), url);
        }
    }
}
