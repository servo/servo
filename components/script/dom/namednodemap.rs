/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::NamedNodeMapBinding;
use dom::bindings::codegen::Bindings::NamedNodeMapBinding::NamedNodeMapMethods;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bindings::xmlname::namespace_from_domstring;
use dom::element::Element;
use dom::window::Window;
use dom_struct::dom_struct;
use html5ever::LocalName;
use std::ascii::AsciiExt;

#[dom_struct]
pub struct NamedNodeMap {
    reflector_: Reflector,
    owner: JS<Element>,
}

impl NamedNodeMap {
    fn new_inherited(elem: &Element) -> NamedNodeMap {
        NamedNodeMap {
            reflector_: Reflector::new(),
            owner: JS::from_ref(elem),
        }
    }

    pub fn new(window: &Window, elem: &Element) -> Root<NamedNodeMap> {
        reflect_dom_object(box NamedNodeMap::new_inherited(elem),
                           window, NamedNodeMapBinding::Wrap)
    }
}

impl NamedNodeMapMethods for NamedNodeMap {
    // https://dom.spec.whatwg.org/#dom-namednodemap-length
    fn Length(&self) -> u32 {
        self.owner.attrs().len() as u32
    }

    // https://dom.spec.whatwg.org/#dom-namednodemap-item
    fn Item(&self, index: u32) -> Option<Root<Attr>> {
        self.owner.attrs().get(index as usize).map(|js| Root::from_ref(&**js))
    }

    // https://dom.spec.whatwg.org/#dom-namednodemap-getnameditem
    fn GetNamedItem(&self, name: DOMString) -> Option<Root<Attr>> {
        self.owner.get_attribute_by_name(name)
    }

    // https://dom.spec.whatwg.org/#dom-namednodemap-getnameditemns
    fn GetNamedItemNS(&self, namespace: Option<DOMString>, local_name: DOMString)
                     -> Option<Root<Attr>> {
        let ns = namespace_from_domstring(namespace);
        self.owner.get_attribute(&ns, &LocalName::from(local_name))
    }

    // https://dom.spec.whatwg.org/#dom-namednodemap-setnameditem
    fn SetNamedItem(&self, attr: &Attr) -> Fallible<Option<Root<Attr>>> {
        self.owner.SetAttributeNode(attr)
    }

    // https://dom.spec.whatwg.org/#dom-namednodemap-setnameditemns
    fn SetNamedItemNS(&self, attr: &Attr) -> Fallible<Option<Root<Attr>>> {
        self.SetNamedItem(attr)
    }

    // https://dom.spec.whatwg.org/#dom-namednodemap-removenameditem
    fn RemoveNamedItem(&self, name: DOMString) -> Fallible<Root<Attr>> {
        let name = self.owner.parsed_name(name);
        self.owner.remove_attribute_by_name(&name).ok_or(Error::NotFound)
    }

    // https://dom.spec.whatwg.org/#dom-namednodemap-removenameditemns
    fn RemoveNamedItemNS(&self, namespace: Option<DOMString>, local_name: DOMString)
                      -> Fallible<Root<Attr>> {
        let ns = namespace_from_domstring(namespace);
        self.owner.remove_attribute(&ns, &LocalName::from(local_name))
            .ok_or(Error::NotFound)
    }

    // https://dom.spec.whatwg.org/#dom-namednodemap-item
    fn IndexedGetter(&self, index: u32) -> Option<Root<Attr>> {
        self.Item(index)
    }

    // check-tidy: no specs after this line
    fn NamedGetter(&self, name: DOMString) -> Option<Root<Attr>> {
        self.GetNamedItem(name)
    }

    // https://heycam.github.io/webidl/#dfn-supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        let mut names = vec!();
        let html_element_in_html_document = self.owner.html_element_in_html_document();
        for attr in self.owner.attrs().iter() {
            let s = &**attr.name();
            if html_element_in_html_document && !s.bytes().all(|b| b.to_ascii_lowercase() == b) {
                continue
            }

            if !names.iter().any(|name| &*name == s) {
                names.push(DOMString::from(s));
            }
        }
        names
    }
}
