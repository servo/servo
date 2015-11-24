/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::activation::Activatable;
use dom::attr::AttrValue;
use dom::bindings::codegen::Bindings::HTMLLabelElementBinding;
use dom::bindings::codegen::Bindings::HTMLLabelElementBinding::HTMLLabelElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::Element;
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::htmlformelement::{FormControl, HTMLFormElement};
use dom::node::{document_from_node, Node};
use dom::virtualmethods::VirtualMethods;
use string_cache::Atom;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLLabelElement {
    htmlelement: HTMLElement,
}

impl HTMLLabelElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLLabelElement {
        HTMLLabelElement {
            htmlelement:
                HTMLElement::new_inherited(localName, prefix, document)
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

impl Activatable for HTMLLabelElement {
    fn as_element(&self) -> &Element {
        self.upcast::<Element>()
    }

    fn is_instance_activatable(&self) -> bool {
        return true;
    }

    // https://html.spec.whatwg.org/multipage/#run-pre-click-activation-steps
    // https://html.spec.whatwg.org/multipage/#the-button-element:activation-behavior
    fn pre_click_activation(&self) {
    }

    // https://html.spec.whatwg.org/multipage/#run-canceled-activation-steps
    fn canceled_activation(&self) {
    }

    // https://html.spec.whatwg.org/multipage/#run-post-click-activation-steps
    fn activation_behavior(&self, _event: &Event, _target: &EventTarget) {
        self.upcast::<Element>()
            .as_maybe_activatable()
            .map(|a| a.synthetic_click_activation(false, false, false, false));
    }

    // https://html.spec.whatwg.org/multipage/#implicit-submission
    fn implicit_submission(&self, _ctrlKey: bool, _shiftKey: bool, _altKey: bool, _metaKey: bool) {
        //FIXME: Investigate and implement implicit submission for label elements
        // Issue filed at https://github.com/servo/servo/issues/8263
    }


}

impl HTMLLabelElementMethods for HTMLLabelElement {
    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<Root<HTMLFormElement>> {
        self.form_owner()
    }

    // https://html.spec.whatwg.org/multipage/#dom-label-htmlfor
    make_getter!(HtmlFor, "for");

    // https://html.spec.whatwg.org/multipage/#dom-label-htmlfor
    make_atomic_setter!(SetHtmlFor, "for");

    // https://html.spec.whatwg.org/multipage/#dom-label-control
    fn GetControl(&self) -> Option<Root<HTMLElement>> {
        if !self.upcast::<Node>().is_in_doc() {
            return None;
        }

        let for_attr = match self.upcast::<Element>().get_attribute(&ns!(), &atom!("for")) {
            Some(for_attr) => for_attr,
            None => return self.first_labelable_descendant(),
        };

        let for_value = for_attr.value();
        document_from_node(self).get_element_by_id(for_value.as_atom())
                                .and_then(Root::downcast::<HTMLElement>)
                                .into_iter()
                                .filter(|e| e.is_labelable_element())
                                .next()
    }
}

impl VirtualMethods for HTMLLabelElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("for") => AttrValue::from_atomic(value),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}

impl HTMLLabelElement {
    pub fn first_labelable_descendant(&self) -> Option<Root<HTMLElement>> {
        self.upcast::<Node>()
            .traverse_preorder()
            .filter_map(Root::downcast::<HTMLElement>)
            .filter(|elem| elem.is_labelable_element())
            .next()
    }
}

impl FormControl for HTMLLabelElement {}
