/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::HTMLObjectElementBinding;
use dom::bindings::codegen::Bindings::HTMLObjectElementBinding::HTMLObjectElementMethods;
use dom::bindings::codegen::InheritTypes::HTMLObjectElementDerived;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::{Element, HTMLObjectElementTypeId};
use dom::element::AttributeHandlers;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId, NodeHelpers, window_from_node};
use dom::validitystate::ValidityState;
use dom::virtualmethods::VirtualMethods;

use servo_net::image_cache_task;
use servo_net::image_cache_task::ImageCacheTask;
use servo_util::str::DOMString;
use string_cache::Atom;

use url::Url;

#[jstraceable]
#[must_root]
pub struct HTMLObjectElement {
    pub htmlelement: HTMLElement,
}

impl HTMLObjectElementDerived for EventTarget {
    fn is_htmlobjectelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLObjectElementTypeId))
    }
}

impl HTMLObjectElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLObjectElement {
        HTMLObjectElement {
            htmlelement: HTMLElement::new_inherited(HTMLObjectElementTypeId, localName, prefix, document),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLObjectElement> {
        let element = HTMLObjectElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLObjectElementBinding::Wrap)
    }
}

trait ProcessDataURL {
    fn process_data_url(&self, image_cache: ImageCacheTask);
}

impl<'a> ProcessDataURL for JSRef<'a, HTMLObjectElement> {
    // Makes the local `data` member match the status of the `data` attribute and starts
    /// prefetching the image. This method must be called after `data` is changed.
    fn process_data_url(&self, image_cache: ImageCacheTask) {
        let elem: JSRef<Element> = ElementCast::from_ref(*self);

        // TODO: support other values
        match (elem.get_attribute(ns!(""), "type").map(|x| x.root().Value()),
               elem.get_attribute(ns!(""), "data").map(|x| x.root().Value())) {
            (None, Some(uri)) => {
                if is_image_data(uri.as_slice()) {
                    let data_url = Url::parse(uri.as_slice()).unwrap();
                    // Issue #84
                    image_cache.send(image_cache_task::Prefetch(data_url));
                }
            }
            _ => { }
        }
    }
}

pub fn is_image_data(uri: &str) -> bool {
    static types: &'static [&'static str] = &["data:image/png", "data:image/gif", "data:image/jpeg"];
    types.iter().any(|&type_| uri.starts_with(type_))
}

impl<'a> HTMLObjectElementMethods for JSRef<'a, HTMLObjectElement> {
    fn Validity(self) -> Temporary<ValidityState> {
        let window = window_from_node(self).root();
        ValidityState::new(*window)
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLObjectElement> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, name: &Atom, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(name, value),
            _ => (),
        }

        if "data" == name.as_slice() {
            let window = window_from_node(*self).root();
            self.process_data_url(window.image_cache_task.clone());
        }
    }
}

impl Reflectable for HTMLObjectElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
