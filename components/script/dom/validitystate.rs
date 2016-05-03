/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use dom::bindings::codegen::Bindings::ValidityStateBinding;
use dom::bindings::codegen::Bindings::ValidityStateBinding::ValidityStateMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::element::Element;
use dom::htmlbuttonelement::HTMLButtonElement;
use dom::htmlformelement::FormControl;
use dom::htmlinputelement::HTMLInputElement;
use dom::htmlobjectelement::HTMLObjectElement;
use dom::htmlselectelement::HTMLSelectElement;
use dom::htmltextareaelement::HTMLTextAreaElement;
use dom::node::Node;
use dom::window::Window;

/// https://html.spec.whatwg.org/multipage/#validity-states
#[derive(JSTraceable, HeapSizeOf)]
pub enum ValidityStates {
    ValueMissing,
    TypeMismatch,
    PatternMismatch,
    TooLong,
    TooShort,
    RangeUnderflow,
    RangeOverflow,
    StepMismatch,
    BadInput,
    CustomError,
    Valid
}

/// https://html.spec.whatwg.org/multipage/#validitystate
#[dom_struct]
pub struct ValidityState {
    reflector_: Reflector,
    element: JS<Element>,
    state: ValidityStates
}


impl ValidityState {
    fn new_inherited(element: &Element) -> ValidityState {
        ValidityState {
            reflector_: Reflector::new(),
            element: JS::from_ref(element),
            state: ValidityStates::Valid
        }
    }

    pub fn new(window: &Window, element: &Element) -> Root<ValidityState> {
        reflect_dom_object(box ValidityState::new_inherited(element),
                           GlobalRef::Window(window),
                           ValidityStateBinding::Wrap)
    }
}

impl ValidityStateMethods for ValidityState {

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-valuemissing
    fn ValueMissing(&self) -> bool { return false; }


    // https://html.spec.whatwg.org/multipage/#dom-validitystate-typemismatch
    fn TypeMismatch(&self) -> bool { return false; }


    // https://html.spec.whatwg.org/multipage/#dom-validitystate-patternmismatch
    fn PatternMismatch(&self) -> bool { return false; }


    // https://html.spec.whatwg.org/multipage/#dom-validitystate-toolong
    fn TooLong(&self) -> bool { return false; }


    // https://html.spec.whatwg.org/multipage/#dom-validitystate-tooshort
    fn TooShort(&self) -> bool { return false; }


    // https://html.spec.whatwg.org/multipage/#dom-validitystate-rangeunderflow
    fn RangeUnderflow(&self) -> bool { return false; }


    // https://html.spec.whatwg.org/multipage/#dom-validitystate-rangeoverflow
    fn RangeOverflow(&self) -> bool { return false; }


    // https://html.spec.whatwg.org/multipage/#dom-validitystate-stepmismatch
    fn StepMismatch(&self) -> bool { return false; }


    // https://html.spec.whatwg.org/multipage/#dom-validitystate-badinput
    fn BadInput(&self) -> bool { return false; }


    // https://html.spec.whatwg.org/multipage/#dom-validitystate-customerror
    fn CustomError(&self) -> bool { return false; }


    // https://html.spec.whatwg.org/multipage/#dom-validitystate-valid
    fn Valid(&self) -> bool {
        match self.element.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                let html_input_element = self.element.downcast::<HTMLInputElement>().unwrap();
                return !(
                    html_input_element.ValueMissing()|
                    html_input_element.TypeMismatch()|
                    html_input_element.PatternMismatch()|
                    html_input_element.TooLong()|
                    html_input_element.TooShort()|
                    html_input_element.RangeUnderflow()|
                    html_input_element.RangeOverflow()|
                    html_input_element.StepMismatch()|
                    html_input_element.BadInput()|
                    html_input_element.CustomError());

            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) => {
            let html_button_element = self.element.downcast::<HTMLButtonElement>().unwrap();
                return !(
                    html_button_element.ValueMissing()|
                    html_button_element.TypeMismatch()|
                    html_button_element.PatternMismatch()|
                    html_button_element.TooLong()|
                    html_button_element.TooShort()|
                    html_button_element.RangeUnderflow()|
                    html_button_element.RangeOverflow()|
                    html_button_element.StepMismatch()|
                    html_button_element.BadInput()|
                    html_button_element.CustomError());
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement)) => {
            let html_object_element = self.element.downcast::<HTMLObjectElement>().unwrap();
                return !(
                    html_object_element.ValueMissing()|
                    html_object_element.TypeMismatch()|
                    html_object_element.PatternMismatch()|
                    html_object_element.TooLong()|
                    html_object_element.TooShort()|
                    html_object_element.RangeUnderflow()|
                    html_object_element.RangeOverflow()|
                    html_object_element.StepMismatch()|
                    html_object_element.BadInput()|
                    html_object_element.CustomError());
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
            let html_select_element = self.element.downcast::<HTMLSelectElement>().unwrap();
                return !(
                    html_select_element.ValueMissing()|
                    html_select_element.TypeMismatch()|
                    html_select_element.PatternMismatch()|
                    html_select_element.TooLong()|
                    html_select_element.TooShort()|
                    html_select_element.RangeUnderflow()|
                    html_select_element.RangeOverflow()|
                    html_select_element.StepMismatch()|
                    html_select_element.BadInput()|
                    html_select_element.CustomError());
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
            let html_textarea_element = self.element.downcast::<HTMLTextAreaElement>().unwrap();
                return !(
                    html_textarea_element.ValueMissing()|
                    html_textarea_element.TypeMismatch()|
                    html_textarea_element.PatternMismatch()|
                    html_textarea_element.TooLong()|
                    html_textarea_element.TooShort()|
                    html_textarea_element.RangeUnderflow()|
                    html_textarea_element.RangeOverflow()|
                    html_textarea_element.StepMismatch()|
                    html_textarea_element.BadInput()|
                    html_textarea_element.CustomError());
            },
            NodeTypeId::Element(_)  => {
                return false;
            }
            NodeTypeId::CharacterData(_)  => {
                return false;
            }
            NodeTypeId::Document(_)  => {
                return false;
            }
            NodeTypeId::DocumentFragment  => {
                return false;
            }
            NodeTypeId::DocumentType  => {
                return false;
            }
        };
    }
}
