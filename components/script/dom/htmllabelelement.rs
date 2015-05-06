/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::attr::AttrHelpers;
use dom::bindings::codegen::Bindings::HTMLLabelElementBinding;
use dom::bindings::codegen::Bindings::HTMLLabelElementBinding::HTMLLabelElementMethods;
use dom::bindings::codegen::InheritTypes::ElementCast;
use dom::bindings::codegen::InheritTypes::HTMLElementCast;
use dom::bindings::codegen::InheritTypes::HTMLLabelElementDerived;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::{Element, ElementTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::htmlformelement::{HTMLFormElement, FormControl};
use dom::node::{Node, NodeTypeId};
use dom::virtualmethods::VirtualMethods;
use string_cache::Atom;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLLabelElement {
    htmlelement: HTMLElement,
    form_owner: MutNullableHeap<JS<HTMLFormElement>>,
}

impl HTMLLabelElementDerived for EventTarget {
    fn is_htmllabelelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLabelElement)))
    }
}

impl HTMLLabelElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLLabelElement {
        HTMLLabelElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLLabelElement, localName, prefix, document),
            form_owner: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLLabelElement> {
        let element = HTMLLabelElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLLabelElementBinding::Wrap)
    }
}

impl<'a> HTMLLabelElementMethods for &'a HTMLLabelElement {
    // https://html.spec.whatwg.org/multipage/forms.html#dom-fae-form
    fn GetForm(self) -> Option<Root<HTMLFormElement>> {
        self.form_owner()
    }
}

impl<'a> VirtualMethods for &'a HTMLLabelElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &&HTMLElement = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.after_set_attr(attr);
        }

        match attr.local_name() {
            &atom!("form") => {
                self.after_set_form_attr();
            },
            _ => ()
        }
    }

    fn before_remove_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.before_remove_attr(attr);
        }

        match attr.local_name() {
            &atom!("form") => {
                self.before_remove_form_attr();
            },
            _ => ()
        }
    }

    fn after_remove_attr(&self, attr: &Atom) {
        if let Some(ref s) = self.super_type() {
            s.after_remove_attr(attr);
        }

        match attr {
            &atom!("form") => {
                self.after_remove_form_attr();
            }
            _ => ()
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        self.bind_form_control_to_tree();
    }

    fn unbind_from_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.unbind_from_tree(tree_in_doc);
        }

        self.unbind_form_control_from_tree();
    }
}

impl<'a> FormControl for &'a HTMLLabelElement {
    fn form_owner(&self) -> Option<Root<HTMLFormElement>> {
        self.form_owner.get().map(Root::from_rooted)
    }

    fn set_form_owner(&self, form: Option<&HTMLFormElement>) {
        self.form_owner.set(form.map(JS::from_ref));
    }

    fn to_element<'b>(&'b self) -> &'b Element {
        ElementCast::from_ref(*self)
    }
}
