/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::LocalName;
use script_bindings::reflector::{Reflector, reflect_dom_object};

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::NamedNodeMapBinding::NamedNodeMapMethods;
use crate::dom::bindings::domname::namespace_from_domstring;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct NamedNodeMap {
    reflector_: Reflector,
    owner: Dom<Element>,
}

impl NamedNodeMap {
    fn new_inherited(elem: &Element) -> NamedNodeMap {
        NamedNodeMap {
            reflector_: Reflector::new(),
            owner: Dom::from_ref(elem),
        }
    }

    pub(crate) fn new(window: &Window, elem: &Element, can_gc: CanGc) -> DomRoot<NamedNodeMap> {
        reflect_dom_object(Box::new(NamedNodeMap::new_inherited(elem)), window, can_gc)
    }
}

impl NamedNodeMapMethods<crate::DomTypeHolder> for NamedNodeMap {
    /// <https://dom.spec.whatwg.org/#dom-namednodemap-length>
    fn Length(&self) -> u32 {
        self.owner.attrs().borrow().len() as u32
    }

    /// <https://dom.spec.whatwg.org/#dom-namednodemap-item>
    fn Item(&self, index: u32) -> Option<DomRoot<Attr>> {
        let index: usize = index as _;
        if self.owner.attrs().borrow().len() <= index {
            None
        } else {
            Some(self.owner.attrs().ensure_dom(index, &self.owner))
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-namednodemap-getnameditem>
    fn GetNamedItem(&self, name: DOMString) -> Option<DomRoot<Attr>> {
        self.owner.get_attribute_by_name(name)
    }

    /// <https://dom.spec.whatwg.org/#dom-namednodemap-getnameditemns>
    fn GetNamedItemNS(
        &self,
        namespace: Option<DOMString>,
        local_name: DOMString,
    ) -> Option<DomRoot<Attr>> {
        let ns = namespace_from_domstring(namespace);
        self.owner
            .get_attribute_with_namespace(&ns, &LocalName::from(local_name))
    }

    /// <https://dom.spec.whatwg.org/#dom-namednodemap-setnameditem>
    fn SetNamedItem(
        &self,
        cx: &mut js::context::JSContext,
        attr: &Attr,
    ) -> Fallible<Option<DomRoot<Attr>>> {
        self.owner.SetAttributeNode(cx, attr)
    }

    /// <https://dom.spec.whatwg.org/#dom-namednodemap-setnameditemns>
    fn SetNamedItemNS(
        &self,
        cx: &mut js::context::JSContext,
        attr: &Attr,
    ) -> Fallible<Option<DomRoot<Attr>>> {
        self.SetNamedItem(cx, attr)
    }

    /// <https://dom.spec.whatwg.org/#dom-namednodemap-removenameditem>
    fn RemoveNamedItem(
        &self,
        cx: &mut js::context::JSContext,
        name: DOMString,
    ) -> Fallible<DomRoot<Attr>> {
        let name = self.owner.parsed_name(name);
        self.owner
            .remove_attribute_by_name(cx, &name)
            .ok_or(Error::NotFound(None))
    }

    /// <https://dom.spec.whatwg.org/#dom-namednodemap-removenameditemns>
    fn RemoveNamedItemNS(
        &self,
        cx: &mut js::context::JSContext,
        namespace: Option<DOMString>,
        local_name: DOMString,
    ) -> Fallible<DomRoot<Attr>> {
        let ns = namespace_from_domstring(namespace);
        self.owner
            .remove_attribute(cx, &ns, &LocalName::from(local_name))
            .ok_or(Error::NotFound(None))
    }

    /// <https://dom.spec.whatwg.org/#dom-namednodemap-item>
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<Attr>> {
        self.Item(index)
    }

    // check-tidy: no specs after this line
    fn NamedGetter(&self, name: DOMString) -> Option<DomRoot<Attr>> {
        self.GetNamedItem(name)
    }

    /// <https://heycam.github.io/webidl/#dfn-supported-property-names>
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        let mut names = vec![];
        let html_element_in_html_document = self.owner.html_element_in_html_document();
        for attr in self.owner.attrs().borrow().iter() {
            let s = &**attr.name();
            if html_element_in_html_document && !s.bytes().all(|b| b.to_ascii_lowercase() == b) {
                continue;
            }

            if !names.iter().any(|name| name == s) {
                names.push(DOMString::from(s));
            }
        }
        names
    }
}
