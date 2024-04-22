/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{local_name, namespace_url, ns, LocalName, Prefix};
use js::rust::HandleObject;
use style::attr::AttrValue;

use crate::dom::activation::Activatable;
use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use crate::dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLLabelElementBinding::HTMLLabelElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::{GetRootNodeOptions, NodeMethods};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlformelement::{FormControl, FormControlElementHelpers, HTMLFormElement};
use crate::dom::node::{Node, ShadowIncluding};
use crate::dom::virtualmethods::VirtualMethods;

#[dom_struct]
pub struct HTMLLabelElement {
    htmlelement: HTMLElement,
}

impl HTMLLabelElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLLabelElement {
        HTMLLabelElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLLabelElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLLabelElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }
}

impl Activatable for HTMLLabelElement {
    fn as_element(&self) -> &Element {
        self.upcast::<Element>()
    }

    fn is_instance_activatable(&self) -> bool {
        true
    }

    // https://html.spec.whatwg.org/multipage/#the-label-element:activation_behaviour
    // Basically this is telling us that if activation bubbles up to the label
    // at all, we are free to do an implementation-dependent thing;
    // firing a click event is an example, and the precise details of that
    // click event (e.g. isTrusted) are not specified.
    fn activation_behavior(&self, _event: &Event, _target: &EventTarget) {
        if let Some(e) = self.GetControl() {
            e.Click();
        }
    }
}

impl HTMLLabelElementMethods for HTMLLabelElement {
    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner()
    }

    // https://html.spec.whatwg.org/multipage/#dom-label-htmlfor
    make_getter!(HtmlFor, "for");

    // https://html.spec.whatwg.org/multipage/#dom-label-htmlfor
    make_atomic_setter!(SetHtmlFor, "for");

    // https://html.spec.whatwg.org/multipage/#dom-label-control
    fn GetControl(&self) -> Option<DomRoot<HTMLElement>> {
        let for_attr = match self
            .upcast::<Element>()
            .get_attribute(&ns!(), &local_name!("for"))
        {
            Some(for_attr) => for_attr,
            None => return self.first_labelable_descendant(),
        };

        let for_value = for_attr.Value();

        // "If the attribute is specified and there is an element in the tree
        // whose ID is equal to the value of the for attribute, and the first
        // such element in tree order is a labelable element, then that
        // element is the label element's labeled control."
        // Two subtle points here: we need to search the _tree_, which is
        // not necessarily the document if we're detached from the document,
        // and we only consider one element even if a later element with
        // the same ID is labelable.

        let maybe_found = self
            .upcast::<Node>()
            .GetRootNode(&GetRootNodeOptions::empty())
            .traverse_preorder(ShadowIncluding::No)
            .find_map(|e| {
                if let Some(htmle) = e.downcast::<HTMLElement>() {
                    if htmle.upcast::<Element>().Id() == for_value {
                        Some(DomRoot::from_ref(htmle))
                    } else {
                        None
                    }
                } else {
                    None
                }
            });
        // We now have the element that we would return, but only return it
        // if it's labelable.
        if let Some(ref maybe_labelable) = maybe_found {
            if maybe_labelable.is_labelable_element() {
                return maybe_found;
            }
        }
        None
    }
}

impl VirtualMethods for HTMLLabelElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("for") => AttrValue::from_atomic(value.into()),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        if *attr.local_name() == local_name!("form") {
            self.form_attribute_mutated(mutation);
        }
    }
}

impl HTMLLabelElement {
    pub fn first_labelable_descendant(&self) -> Option<DomRoot<HTMLElement>> {
        self.upcast::<Node>()
            .traverse_preorder(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<HTMLElement>)
            .find(|elem| elem.is_labelable_element())
    }
}

impl FormControl for HTMLLabelElement {
    fn form_owner(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.GetControl()
            .map(DomRoot::upcast::<Element>)
            .and_then(|elem| {
                elem.as_maybe_form_control()
                    .and_then(|control| control.form_owner())
            })
    }

    fn set_form_owner(&self, _: Option<&HTMLFormElement>) {
        // Label is a special case for form owner, it reflects its control's
        // form owner. Therefore it doesn't hold form owner itself.
    }

    fn to_element(&self) -> &Element {
        self.upcast::<Element>()
    }
}
