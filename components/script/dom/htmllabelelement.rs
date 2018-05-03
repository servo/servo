/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::activation::{Activatable, ActivationSource, synthetic_click_activation};
use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLLabelElementBinding;
use dom::bindings::codegen::Bindings::HTMLLabelElementBinding::HTMLLabelElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::htmlformelement::{FormControl, FormControlElementHelpers, HTMLFormElement};
use dom::node::{document_from_node, Node};
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use style::attr::AttrValue;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLLabelElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>
}

impl<TH: TypeHolderTrait> HTMLLabelElement<TH> {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document<TH>) -> HTMLLabelElement<TH> {
        HTMLLabelElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLLabelElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLLabelElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLLabelElementBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> Activatable<TH> for HTMLLabelElement<TH> {
    fn as_element(&self) -> &Element<TH> {
        self.upcast::<Element<TH>>()
    }

    fn is_instance_activatable(&self) -> bool {
        true
    }

    // https://html.spec.whatwg.org/multipage/#run-pre-click-activation-steps
    // https://html.spec.whatwg.org/multipage/#the-button-element:activation-behavior
    fn pre_click_activation(&self) {
    }

    // https://html.spec.whatwg.org/multipage/#run-canceled-activation-steps
    fn canceled_activation(&self) {
    }

    // https://html.spec.whatwg.org/multipage/#run-post-click-activation-steps
    fn activation_behavior(&self, _event: &Event<TH>, _target: &EventTarget<TH>) {
        if let Some(e) = self.GetControl() {
            let elem = e.upcast::<Element<TH>>();
            synthetic_click_activation(elem,
                                       false,
                                       false,
                                       false,
                                       false,
                                       ActivationSource::NotFromClick);
        }
    }

    // https://html.spec.whatwg.org/multipage/#implicit-submission
    fn implicit_submission(&self, _ctrl_key: bool, _shift_key: bool, _alt_key: bool, _meta_key: bool) {
        //FIXME: Investigate and implement implicit submission for label elements
        // Issue filed at https://github.com/servo/servo/issues/8263
    }


}

impl<TH: TypeHolderTrait> HTMLLabelElementMethods<TH> for HTMLLabelElement<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement<TH>>> {
        self.form_owner()
    }

    // https://html.spec.whatwg.org/multipage/#dom-label-htmlfor
    make_getter!(HtmlFor, "for");

    // https://html.spec.whatwg.org/multipage/#dom-label-htmlfor
    make_atomic_setter!(SetHtmlFor, "for");

    // https://html.spec.whatwg.org/multipage/#dom-label-control
    fn GetControl(&self) -> Option<DomRoot<HTMLElement<TH>>> {
        if !self.upcast::<Node<TH>>().is_in_doc() {
            return None;
        }

        let for_attr = match self.upcast::<Element<TH>>().get_attribute(&ns!(), &local_name!("for")) {
            Some(for_attr) => for_attr,
            None => return self.first_labelable_descendant(),
        };

        let for_value = for_attr.value();
        document_from_node(self).get_element_by_id(for_value.as_atom())
                                .and_then(DomRoot::downcast::<HTMLElement<TH>>)
                                .into_iter()
                                .filter(|e| e.is_labelable_element())
                                .next()
    }
}

impl<TH: TypeHolderTrait> VirtualMethods<TH> for HTMLLabelElement<TH> {
    fn super_type(&self) -> Option<&VirtualMethods<TH>> {
        Some(self.upcast::<HTMLElement<TH>>() as &VirtualMethods<TH>)
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("for") => AttrValue::from_atomic(value.into()),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }

    fn attribute_mutated(&self, attr: &Attr<TH>, mutation: AttributeMutation<TH>) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &local_name!("form") => {
                self.form_attribute_mutated(mutation);
            },
            _ => {},
        }
    }
}

impl<TH: TypeHolderTrait> HTMLLabelElement<TH> {
    pub fn first_labelable_descendant(&self) -> Option<DomRoot<HTMLElement<TH>>> {
        self.upcast::<Node<TH>>()
            .traverse_preorder()
            .filter_map(DomRoot::downcast::<HTMLElement<TH>>)
            .filter(|elem| elem.is_labelable_element())
            .next()
    }
}

impl<TH: TypeHolderTrait> FormControl<TH> for HTMLLabelElement<TH> {
    fn form_owner(&self) -> Option<DomRoot<HTMLFormElement<TH>>> {
        self.GetControl().map(DomRoot::upcast::<Element<TH>>).and_then(|elem| {
            elem.as_maybe_form_control().and_then(|control| control.form_owner())
        })
    }

    fn set_form_owner(&self, _: Option<&HTMLFormElement<TH>>) {
        // Label is a special case for form owner, it reflects its control's
        // form owner. Therefore it doesn't hold form owner itself.
    }

    fn to_element<'a>(&'a self) -> &'a Element<TH> {
        self.upcast::<Element<TH>>()
    }
}
