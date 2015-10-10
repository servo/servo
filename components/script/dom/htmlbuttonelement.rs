/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::activation::Activatable;
use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLButtonElementBinding;
use dom::bindings::codegen::Bindings::HTMLButtonElementBinding::HTMLButtonElementMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLButtonElementCast, HTMLElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{HTMLButtonElementDerived, HTMLFieldSetElementDerived};
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::{AttributeMutation, Element, ElementTypeId};
use dom::event::Event;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::htmlformelement::{FormControl, FormSubmitter};
use dom::htmlformelement::{SubmittedFrom, HTMLFormElement};
use dom::node::{Node, NodeTypeId, document_from_node, window_from_node};
use dom::validitystate::ValidityState;
use dom::virtualmethods::VirtualMethods;
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::cell::Cell;
use util::str::DOMString;

#[derive(JSTraceable, PartialEq, Copy, Clone)]
#[allow(dead_code)]
#[derive(HeapSizeOf)]
enum ButtonType {
    ButtonSubmit,
    ButtonReset,
    ButtonButton,
    ButtonMenu
}

#[dom_struct]
pub struct HTMLButtonElement {
    htmlelement: HTMLElement,
    button_type: Cell<ButtonType>
}

impl HTMLButtonElementDerived for EventTarget {
    fn is_htmlbuttonelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)))
    }
}

impl HTMLButtonElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLButtonElement {
        HTMLButtonElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLButtonElement, localName, prefix, document),
            //TODO: implement button_type in attribute_mutated
            button_type: Cell::new(ButtonType::ButtonSubmit)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLButtonElement> {
        let element = HTMLButtonElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLButtonElementBinding::Wrap)
    }
}

impl HTMLButtonElementMethods for HTMLButtonElement {
    // https://html.spec.whatwg.org/multipage/#dom-cva-validity
    fn Validity(&self) -> Root<ValidityState> {
        let window = window_from_node(self);
        ValidityState::new(window.r())
    }

    // https://www.whatwg.org/html/#dom-fe-disabled
    make_bool_getter_setter!(Disabled, SetDisabled);

    // https://html.spec.whatwg.org/multipage#dom-fae-form
    fn GetForm(&self) -> Option<Root<HTMLFormElement>> {
        self.form_owner()
    }

    // https://html.spec.whatwg.org/multipage/#dom-button-type
    make_enumerated_getter_setter!(Type, SetType, "submit", ("reset") | ("button") | ("menu"));

    // https://html.spec.whatwg.org/multipage/#dom-fs-formaction
    make_url_or_base_getter_setter!(FormAction, SetFormAction);

    // https://html.spec.whatwg.org/multipage/#dom-fs-formenctype
    make_enumerated_getter_setter!(FormEnctype, SetFormEnctype,
                                   "application/x-www-form-urlencoded",
                                   ("text/plain") | ("multipart/form-data"));

    // https://html.spec.whatwg.org/multipage/#dom-fs-formmethod
    make_enumerated_getter_setter!(FormMethod, SetFormMethod, "get", ("post") | ("dialog"));

    // https://html.spec.whatwg.org/multipage/#dom-fs-formtarget
    make_getter_setter!(FormTarget, SetFormTarget);

    // https://html.spec.whatwg.org/multipage/#dom-fe-name
    make_getter_setter!(Name, SetName);

    // https://html.spec.whatwg.org/multipage/#dom-button-value
    make_getter_setter!(Value, SetValue);
}

impl VirtualMethods for HTMLButtonElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &HTMLElement = HTMLElementCast::from_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &atom!(disabled) => {
                let node = NodeCast::from_ref(self);
                match mutation {
                    AttributeMutation::Set(Some(_)) => {}
                    AttributeMutation::Set(None) => {
                        node.set_disabled_state(true);
                        node.set_enabled_state(false);
                    },
                    AttributeMutation::Removed => {
                        node.set_disabled_state(false);
                        node.set_enabled_state(true);
                        node.check_ancestors_disabled_state_for_form_control();
                    }
                }
            },
            _ => {},
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        let node = NodeCast::from_ref(self);
        node.check_ancestors_disabled_state_for_form_control();
    }

    fn unbind_from_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.unbind_from_tree(tree_in_doc);
        }

        let node = NodeCast::from_ref(self);
        if node.ancestors().any(|ancestor| ancestor.r().is_htmlfieldsetelement()) {
            node.check_ancestors_disabled_state_for_form_control();
        } else {
            node.check_disabled_attribute();
        }
    }
}

impl FormControl for HTMLButtonElement {}

impl<'a> Activatable for &'a HTMLButtonElement {
    fn as_element<'b>(&'b self) -> &'b Element {
        ElementCast::from_ref(*self)
    }

    fn is_instance_activatable(&self) -> bool {
        //https://html.spec.whatwg.org/multipage/#the-button-element
        let node = NodeCast::from_ref(*self);
        !(node.get_disabled_state())
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
        let ty = self.button_type.get();
        match ty {
            //https://html.spec.whatwg.org/multipage/#attr-button-type-submit-state
            ButtonType::ButtonSubmit => {
                self.form_owner().map(|o| {
                    o.r().submit(SubmittedFrom::NotFromFormSubmitMethod,
                                 FormSubmitter::ButtonElement(self.clone()))
                });
            },
            _ => ()
        }
    }

    // https://html.spec.whatwg.org/multipage/#implicit-submission
    #[allow(unsafe_code)]
    fn implicit_submission(&self, ctrlKey: bool, shiftKey: bool, altKey: bool, metaKey: bool) {
        let doc = document_from_node(*self);
        let node = NodeCast::from_ref(doc.r());
        let owner = self.form_owner();
        let elem = ElementCast::from_ref(*self);
        if owner.is_none() || elem.click_in_progress() {
            return;
        }
        // This is safe because we are stopping after finding the first element
        // and only then performing actions which may modify the DOM tree
        unsafe {
            node.query_selector_iter("button[type=submit]".to_owned()).unwrap()
                .filter_map(HTMLButtonElementCast::to_root)
                .find(|r| r.r().form_owner() == owner)
                .map(|s| s.r().synthetic_click_activation(ctrlKey, shiftKey, altKey, metaKey));
        }
    }
}
