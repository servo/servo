// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::HTMLLegendElementBinding;
use dom::bindings::codegen::Bindings::HTMLLegendElementBinding::HTMLLegendElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::Element;
use dom::htmlelement::HTMLElement;
use dom::htmlfieldsetelement::HTMLFieldSetElement;
use dom::htmlformelement::{HTMLFormElement, FormControl};
use dom::node::{ChildrenMutation, Node, NodeDamage, UnbindContext};
use dom::virtualmethods::VirtualMethods;
use string_cache::Atom;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLLegendElement {
    htmlelement: HTMLElement,
}

impl HTMLLegendElement {
    fn new_inherited(localName: Atom,
                     prefix: Option<DOMString>,
                     document: &Document)
                     -> HTMLLegendElement {
        HTMLLegendElement { htmlelement: HTMLElement::new_inherited(localName, prefix, document) }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom,
               prefix: Option<DOMString>,
               document: &Document)
               -> Root<HTMLLegendElement> {
        let element = HTMLLegendElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLLegendElementBinding::Wrap)
    }
}

impl VirtualMethods for HTMLLegendElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        self.upcast::<Element>().check_ancestors_disabled_state_for_form_control();
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);

        let node = self.upcast::<Node>();
        let el = self.upcast::<Element>();
        if node.ancestors().any(|ancestor| ancestor.is::<HTMLFieldSetElement>()) {
            el.check_ancestors_disabled_state_for_form_control();
        } else {
            el.check_disabled_attribute();
        }
    }
}


impl HTMLLegendElementMethods for HTMLLegendElement {
    // https://html.spec.whatwg.org/multipage/#dom-legend-form
    fn GetForm(&self) -> Option<Root<HTMLFormElement>> {
        let parent = match self.upcast::<Node>().GetParentElement() {
            Some(parent) => parent,
            None => return None,
        };
        if parent.is::<HTMLFieldSetElement>() {
            return self.form_owner();
        }
        None
    }
}

impl FormControl for HTMLLegendElement {}
