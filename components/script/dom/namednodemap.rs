/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::NamedNodeMapBinding;
use dom::bindings::codegen::Bindings::NamedNodeMapBinding::NamedNodeMapMethods;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::utils::{namespace_from_domstring, Reflector, reflect_dom_object};
use dom::element::{AttributeHandlers, Element, ElementHelpers};
use dom::window::Window;
use util::str::DOMString;

use string_cache::Atom;

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
                           GlobalRef::Window(window), NamedNodeMapBinding::Wrap)
    }
}

impl<'a> NamedNodeMapMethods for &'a NamedNodeMap {
    // https://dom.spec.whatwg.org/#dom-namednodemap-length
    fn Length(self) -> u32 {
        let owner = self.owner.root();
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let owner = owner.r();
        let attrs = owner.attrs();
        attrs.len() as u32
    }

    // https://dom.spec.whatwg.org/#dom-namednodemap-item
    fn Item(self, index: u32) -> Option<Root<Attr>> {
        let owner = self.owner.root();
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let owner = owner.r();
        let attrs = owner.attrs();
        attrs.get(index as usize).map(|t| t.root())
    }

    // https://dom.spec.whatwg.org/#dom-namednodemap-getnameditem
    fn GetNamedItem(self, name: DOMString) -> Option<Root<Attr>> {
        let owner = self.owner.root();
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let owner = owner.r();
        owner.get_attribute_by_name(name)
    }

    // https://dom.spec.whatwg.org/#dom-namednodemap-getnameditemns
    fn GetNamedItemNS(self, namespace: Option<DOMString>, local_name: DOMString)
                     -> Option<Root<Attr>> {
        let owner = self.owner.root();
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let owner = owner.r();
        let ns = namespace_from_domstring(namespace);
        owner.get_attribute(&ns, &Atom::from_slice(&local_name))
    }

    // https://dom.spec.whatwg.org/#dom-namednodemap-removenameditem
    fn RemoveNamedItem(self, name: DOMString) -> Fallible<Root<Attr>> {
        let owner = self.owner.root();
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let owner = owner.r();
        let name = owner.parsed_name(name);
        owner.remove_attribute_by_name(&name).ok_or(Error::NotFound)
    }

    // https://dom.spec.whatwg.org/#dom-namednodemap-removenameditemns
    fn RemoveNamedItemNS(self, namespace: Option<DOMString>, local_name: DOMString)
                      -> Fallible<Root<Attr>> {
        let owner = self.owner.root();
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let owner = owner.r();
        let ns = namespace_from_domstring(namespace);
        owner.remove_attribute(&ns, &Atom::from_slice(&local_name)).ok_or(Error::NotFound)
    }

    fn IndexedGetter(self, index: u32, found: &mut bool) -> Option<Root<Attr>> {
        let item = self.Item(index);
        *found = item.is_some();
        item
    }

    fn NamedGetter(self, name: DOMString, found: &mut bool) -> Option<Root<Attr>> {
        let item = self.GetNamedItem(name);
        *found = item.is_some();
        item
    }
}

