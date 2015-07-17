/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::activation::Activatable;
use dom::attr::Attr;
use dom::attr::AttrHelpers;
use dom::bindings::codegen::Bindings::HTMLButtonElementBinding;
use dom::bindings::codegen::Bindings::HTMLButtonElementBinding::HTMLButtonElementMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast, HTMLButtonElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{HTMLButtonElementDerived, HTMLFieldSetElementDerived};
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::{AttributeHandlers, Element, ElementTypeId};
use dom::element::ActivationElementHelpers;
use dom::event::Event;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::htmlformelement::{FormSubmitter, FormControl, HTMLFormElementHelpers};
use dom::htmlformelement::{SubmittedFrom};
use dom::node::{DisabledStateHelpers, Node, NodeHelpers, NodeTypeId, document_from_node, window_from_node};
use dom::validitystate::ValidityState;
use dom::virtualmethods::VirtualMethods;

use std::ascii::OwnedAsciiExt;
use std::borrow::ToOwned;
use util::str::DOMString;
use std::cell::Cell;

#[derive(JSTraceable, PartialEq, Copy, Clone)]
#[allow(dead_code)]
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
            //TODO: implement button_type in after_set_attr
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

impl<'a> HTMLButtonElementMethods for &'a HTMLButtonElement {
    fn Validity(self) -> Root<ValidityState> {
        let window = window_from_node(self);
        ValidityState::new(window.r())
    }

    // https://www.whatwg.org/html/#dom-fe-disabled
    make_bool_getter!(Disabled);

    // https://www.whatwg.org/html/#dom-fe-disabled
    make_bool_setter!(SetDisabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-button-type
    fn Type(self) -> DOMString {
        let elem = ElementCast::from_ref(self);
        let ty = elem.get_string_attribute(&atom!("type")).into_ascii_lowercase();
        // https://html.spec.whatwg.org/multipage/#attr-button-type
        match &*ty {
            "reset" | "button" | "menu" => ty,
            _ => "submit".to_owned()
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-button-type
    make_setter!(SetType, "type");

    // https://html.spec.whatwg.org/multipage/#htmlbuttonelement
    make_url_or_base_getter!(FormAction);

    make_setter!(SetFormAction, "formaction");

    make_enumerated_getter!(
        FormEnctype, "application/x-www-form-urlencoded", ("text/plain") | ("multipart/form-data"));

    make_setter!(SetFormEnctype, "formenctype");

    make_enumerated_getter!(FormMethod, "get", ("post") | ("dialog"));

    make_setter!(SetFormMethod, "formmethod");

    make_getter!(FormTarget);

    make_setter!(SetFormTarget, "formtarget");

    // https://html.spec.whatwg.org/multipage/#dom-fe-name
    make_getter!(Name);

    // https://html.spec.whatwg.org/multipage/#dom-fe-name
    make_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-button-value
    make_getter!(Value);

    // https://html.spec.whatwg.org/multipage/#dom-button-value
    make_setter!(SetValue, "value");
}

impl<'a> VirtualMethods for &'a HTMLButtonElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &&HTMLElement = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.after_set_attr(attr);
        }

        match attr.local_name() {
            &atom!("disabled") => {
                let node = NodeCast::from_ref(*self);
                node.set_disabled_state(true);
                node.set_enabled_state(false);
            },
            _ => ()
        }
    }

    fn before_remove_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.before_remove_attr(attr);
        }

        match attr.local_name() {
            &atom!("disabled") => {
                let node = NodeCast::from_ref(*self);
                node.set_disabled_state(false);
                node.set_enabled_state(true);
                node.check_ancestors_disabled_state_for_form_control();
            },
            _ => ()
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        let node = NodeCast::from_ref(*self);
        node.check_ancestors_disabled_state_for_form_control();
    }

    fn unbind_from_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.unbind_from_tree(tree_in_doc);
        }

        let node = NodeCast::from_ref(*self);
        if node.ancestors().any(|ancestor| ancestor.r().is_htmlfieldsetelement()) {
            node.check_ancestors_disabled_state_for_form_control();
        } else {
            node.check_disabled_attribute();
        }
    }
}

impl<'a> FormControl<'a> for &'a HTMLButtonElement {
    fn to_element(self) -> &'a Element {
        ElementCast::from_ref(self)
    }
}

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

