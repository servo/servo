/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use crate::dom::bindings::codegen::Bindings::RadioNodeListBinding::RadioNodeListMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::htmlformelement::HTMLFormElement;
use crate::dom::htmlinputelement::{HTMLInputElement, InputType};
use crate::dom::node::Node;
use crate::dom::nodelist::{NodeList, NodeListType, RadioList, RadioListMode};
use crate::dom::window::Window;

#[dom_struct]
pub struct RadioNodeList {
    node_list: NodeList,
}

impl RadioNodeList {
    #[allow(crown::unrooted_must_root)]
    fn new_inherited(list_type: NodeListType) -> RadioNodeList {
        RadioNodeList {
            node_list: NodeList::new_inherited(list_type),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(window: &Window, list_type: NodeListType) -> DomRoot<RadioNodeList> {
        reflect_dom_object(Box::new(RadioNodeList::new_inherited(list_type)), window)
    }

    pub fn new_controls_except_image_inputs(
        window: &Window,
        form: &HTMLFormElement,
        name: &Atom,
    ) -> DomRoot<RadioNodeList> {
        RadioNodeList::new(
            window,
            NodeListType::Radio(RadioList::new(
                form,
                RadioListMode::ControlsExceptImageInputs,
                name.clone(),
            )),
        )
    }

    pub fn new_images(
        window: &Window,
        form: &HTMLFormElement,
        name: &Atom,
    ) -> DomRoot<RadioNodeList> {
        RadioNodeList::new(
            window,
            NodeListType::Radio(RadioList::new(form, RadioListMode::Images, name.clone())),
        )
    }

    // https://dom.spec.whatwg.org/#dom-nodelist-length
    // https://github.com/servo/servo/issues/5875
    #[allow(non_snake_case)]
    pub fn Length(&self) -> u32 {
        self.node_list.Length()
    }
}

impl RadioNodeListMethods for RadioNodeList {
    // https://html.spec.whatwg.org/multipage/#dom-radionodelist-value
    fn Value(&self) -> DOMString {
        self.upcast::<NodeList>()
            .iter()
            .filter_map(|node| {
                // Step 1
                node.downcast::<HTMLInputElement>().and_then(|input| {
                    if input.input_type() == InputType::Radio && input.Checked() {
                        // Step 3-4
                        let value = input.Value();
                        Some(if value.is_empty() {
                            DOMString::from("on")
                        } else {
                            value
                        })
                    } else {
                        None
                    }
                })
            })
            .next()
            // Step 2
            .unwrap_or(DOMString::from(""))
    }

    // https://html.spec.whatwg.org/multipage/#dom-radionodelist-value
    fn SetValue(&self, value: DOMString) {
        for node in self.upcast::<NodeList>().iter() {
            // Step 1
            if let Some(input) = node.downcast::<HTMLInputElement>() {
                match input.input_type() {
                    InputType::Radio if value == *"on" => {
                        // Step 2
                        let val = input.Value();
                        if val.is_empty() || val == value {
                            input.SetChecked(true);
                            return;
                        }
                    },
                    InputType::Radio => {
                        // Step 2
                        if input.Value() == value {
                            input.SetChecked(true);
                            return;
                        }
                    },
                    _ => {},
                }
            }
        }
    }

    // FIXME: This shouldn't need to be implemented here since NodeList (the parent of
    // RadioNodeList) implements IndexedGetter.
    // https://github.com/servo/servo/issues/5875
    //
    // https://dom.spec.whatwg.org/#dom-nodelist-item
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<Node>> {
        self.node_list.IndexedGetter(index)
    }
}
