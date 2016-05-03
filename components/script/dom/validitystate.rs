/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::HTMLOptionElementBinding::HTMLOptionElementMethods;
use dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding::HTMLTextAreaElementMethods;
use dom::bindings::codegen::Bindings::ValidityStateBinding;
use dom::bindings::codegen::Bindings::ValidityStateBinding::ValidityStateMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::element::Element;
use dom::htmlinputelement::HTMLInputElement;
use dom::htmlselectelement::HTMLSelectElement;
use dom::htmltextareaelement::HTMLTextAreaElement;
use dom::node::Node;
use dom::validation::Validatable;
use dom::window::Window;
use regex::Regex;
use util::str::DOMString;

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
    fn ValueMissing(&self) -> bool {
       match self.element.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                let attr_value_check = self.element.get_attribute_by_name(DOMString::from("required"))
                    .map(|s| s.Value());
                if attr_value_check.is_some() {
                        let html_input_element = self.element.downcast::<HTMLInputElement>().unwrap();
                        let input_value_check = html_input_element.get_value_for_validation();
                        if input_value_check.is_some() {
                            return false;
                        }
                        else {
                                println!("Error - Value missing in html input element");
                                return true;
                        }
                        
                }
                else {
                    return false;
                }
                //let data = element1.form_datum(Some(FormSubmitter::InputElement(element1)));
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) => {
               return false;
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement)) => {
                return false;
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
               let attr_value_check = self.element.get_attribute_by_name(DOMString::from("required"))
                .map(|s| s.Value());
                if attr_value_check.is_some() {
                    let html_select_element = self.element.downcast::<HTMLSelectElement>().unwrap();
                    let input_value_check = html_select_element.get_value_for_validation();
                    if input_value_check.is_some() {
                        return false;
                    }
                    else {
                            println!("Error - Value missing in html select area element");
                            return true;
                    }
                    
                }
                else {
                    return false;
                }
                
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
                let attr_value_check = self.element.get_attribute_by_name(DOMString::from("required"))
                    .map(|s| s.Value());

                if attr_value_check.is_some() {
                    
                        let html_textarea_element = self.element.downcast::<HTMLTextAreaElement>().unwrap();
                        let input_value_check = html_textarea_element.get_value_for_validation();
                        if input_value_check.is_some() {
                            return false;
                        }
                        else {
                                println!("Error - Value missing in html text area element");
                                return true;
                        }
                        
                   }
                   else {
                        return false;
                   }
                   
                
            },
            NodeTypeId::Element(_)  => {
            }
            NodeTypeId::CharacterData(_)  => {
            }
            NodeTypeId::Document(_)  => {
            }
            NodeTypeId::DocumentFragment  => {
            }
            NodeTypeId::DocumentType  => {
            }
        };
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-typemismatch
    fn TypeMismatch(&self) -> bool {
        match self.element.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                //let regex_email: Regex = Regex::new(r"/^[a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-] \
                //    {0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$/").unwrap();
                let regex_email: Regex = Regex::new(r"").unwrap();
                let regex_url: Regex = Regex::new(r"").unwrap();
                let regex_number: Regex = Regex::new(r"").unwrap();
                let attr_value_check = self.element.get_attribute_by_name(DOMString::from("type"))
                    .map(|s| s.Value());
                match attr_value_check {
                    Some(attr_value) => {
                        let html_input_element = self.element.downcast::<HTMLInputElement>().unwrap();
                        let input_value_check = html_input_element.get_value_for_validation();
                        match input_value_check {
                            Some(input_value) => {
                                if attr_value == "email"{
                                    if !regex_email.is_match(&*input_value) {
                                        println!("Type error in html text input [email]");
                                        return true;
                                    }
                                }
                                else if attr_value == "url" {
                                    if !regex_url.is_match(&*input_value) {
                                        println!("Type error in html text input [url]");
                                        return true;
                                    }
                                }
                                else if attr_value == "number" {
                                    if !regex_number.is_match(&*input_value) {
                                        println!("Type error in html text input [number]");
                                        return true;
                                    }
                                }
                                else {
                                    return false;
                                }
                            },
                            None => {
                                return false;
                            }
                        }
                    },
                    None => {
                        return false;
                    }
                }
                //let data = element1.form_datum(Some(FormSubmitter::InputElement(element1)));
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
            },
            NodeTypeId::Element(_)  => {
            }
            NodeTypeId::CharacterData(_)  => {
            }
            NodeTypeId::Document(_)  => {
            }
            NodeTypeId::DocumentFragment  => {
            }
            NodeTypeId::DocumentType  => {
            }
        };
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-patternmismatch
    fn PatternMismatch(&self) -> bool {
        match self.element.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                let attr_value_check = self.element.get_attribute_by_name(DOMString::from("pattern"))
                    .map(|s| s.Value());
                match attr_value_check {
                    Some(attr_value) => {
                        let regex = Regex::new(&*attr_value).unwrap();
                        let html_input_element = self.element.downcast::<HTMLInputElement>().unwrap();
                        let input_value_check = html_input_element.get_value_for_validation();
                        match input_value_check {
                            Some(input_value) => {
                                if !regex.is_match(&*input_value) {
                                    println!("PatternMismatch error in html text input");
                                    return true;
                                }
                            },
                            None => {
                                return false;
                            }
                        }
                    },
                    None => {
                        return false;
                    }
                }
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
            },
            NodeTypeId::Element(_)  => {
            }
            NodeTypeId::CharacterData(_)  => {
            }
            NodeTypeId::Document(_)  => {
            }
            NodeTypeId::DocumentFragment  => {
            }
            NodeTypeId::DocumentType  => {
            }
        };
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-toolong
    fn TooLong(&self) -> bool {
        match self.element.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                let attr_value_check = self.element.get_attribute_by_name(DOMString::from("maxlength"))
                    .map(|s| s.Value());
                match attr_value_check {
                    Some(attr_value) => {
                        let maxlength = attr_value.parse().unwrap();
                        let html_input_element = self.element.downcast::<HTMLInputElement>().unwrap();
                        let input_value_check = html_input_element.get_value_for_validation();
                        match input_value_check {
                            Some(input_value) => {
                                if input_value.len() > maxlength {
                                    println!("Error - TooLong html input element");
                                    return true;
                                }
                                else {
                                    return false;
                                }
                            },
                            None => {
                                return false;
                            }
                        }
                    },
                    None => {
                        return false;
                    }
                }
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
                let attr_value_check = self.element.get_attribute_by_name(DOMString::from("maxlength"))
                    .map(|s| s.Value());
                match attr_value_check {
                    Some(attr_value) => {
                        let maxlength = attr_value.parse().unwrap();
                        let html_textarea_element = self.element.downcast::<HTMLTextAreaElement>().unwrap();
                        let input_value_check = html_textarea_element.get_value_for_validation();
                        match input_value_check {
                            Some(input_value) => {
                             if input_value.len() > maxlength {
                                        println!("Error - TooLong in text area");
                                        return true;
                                }
                                else {
                                    return false;
                                }
                            },
                            None => {
                                return false;
                            }
                        }
                    },
                    None => {
                        return false;
                    }
                }
            },
            NodeTypeId::Element(_)  => {
            }
            NodeTypeId::CharacterData(_)  => {
            }
            NodeTypeId::Document(_)  => {
            }
            NodeTypeId::DocumentFragment  => {
            }
            NodeTypeId::DocumentType  => {
            }
        };
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-tooshort
    fn TooShort(&self) -> bool {
        match self.element.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                let attr_value_check = self.element.get_attribute_by_name(DOMString::from("minlength"))
                    .map(|s| s.Value());
                match attr_value_check {
                    Some(attr_value) => {
                        let minlength = attr_value.parse().unwrap();
                        let html_input_element = self.element.downcast::<HTMLInputElement>().unwrap();
                        let input_value_check = html_input_element.get_value_for_validation();
                        match input_value_check {
                            Some(input_value) => {
                                if input_value.len() < minlength {
                                    println!("Error - TooShort html input element");
                                    return true;
                                }
                                else {
                                    return false;
                                }
                            },
                            None => {
                                return false;
                            }
                        }
                    },
                    None => {
                        return false;
                    }
                }
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
                let attr_value_check = self.element.get_attribute_by_name(DOMString::from("minlength"))
                    .map(|s| s.Value());
                match attr_value_check {
                    Some(attr_value) => {
                        let minlength = attr_value.parse().unwrap();
                        let html_textarea_element = self.element.downcast::<HTMLTextAreaElement>().unwrap();
                        let input_value_check = html_textarea_element.get_value_for_validation();
                        match input_value_check {
                            Some(input_value) => {
                             if input_value.len() < minlength {
                                    println!("Error - TooShort html area element");
                                    return true;
                                }
                                else {
                                    return false;
                                }
                            },
                            None => {
                                return false;
                            }
                        }
                    },
                    None => {
                        return false;
                    }
                }
            },
            NodeTypeId::Element(_)  => {
            }
            NodeTypeId::CharacterData(_)  => {
            }
            NodeTypeId::Document(_)  => {
            }
            NodeTypeId::DocumentFragment  => {
            }
            NodeTypeId::DocumentType  => {
            }
        };
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-rangeunderflow
    fn RangeUnderflow(&self) -> bool {
        match self.element.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                let attr_value_check = self.element.get_attribute_by_name(DOMString::from("min"))
                    .map(|s| s.Value());
                match attr_value_check {
                    Some(attr_value) => {
                        let min: f32 = attr_value.parse().unwrap();
                        let html_input_element = self.element.downcast::<HTMLInputElement>().unwrap();
                        let input_value_check = html_input_element.get_value_for_validation();
                        match input_value_check {
                            Some(input_value) => {
                                let text_value: f32 = input_value.parse().unwrap();
                                if text_value < min {
                                    println!("Error - RangeUnderflow html input element");
                                    return true;
                                }
                                else {
                                    return false;
                                }
                            },
                            None => {
                                return false;
                            }
                        }
                    },
                    None => {
                        return false;
                    }
                }
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
            },
            NodeTypeId::Element(_)  => {
            }
            NodeTypeId::CharacterData(_)  => {
            }
            NodeTypeId::Document(_)  => {
            }
            NodeTypeId::DocumentFragment  => {
            }
            NodeTypeId::DocumentType  => {
            }
        };
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-rangeoverflow
    fn RangeOverflow(&self) -> bool {
        match self.element.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                let attr_value_check = self.element.get_attribute_by_name(DOMString::from("max"))
                    .map(|s| s.Value());
                match attr_value_check {
                    Some(attr_value) => {
                        let max: f32 = attr_value.parse().unwrap();
                        let html_input_element = self.element.downcast::<HTMLInputElement>().unwrap();
                        let input_value_check = html_input_element.get_value_for_validation();
                        match input_value_check {
                            Some(input_value) => {
                                let text_value: f32 = input_value.parse().unwrap();
                                if text_value > max {
                                    println!("Error - RangeOverflow html input element");
                                    return true;
                                }
                                else {
                                    return false;
                                }
                            },
                            None => {
                                return false;
                            }
                        }
                    },
                    None => {
                        return false;
                    }
                }
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
            },
            NodeTypeId::Element(_)  => {
            }
            NodeTypeId::CharacterData(_)  => {
            }
            NodeTypeId::Document(_)  => {
            }
            NodeTypeId::DocumentFragment  => {
            }
            NodeTypeId::DocumentType  => {
            }
        };
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-stepmismatch
    fn StepMismatch(&self) -> bool {
        match self.element.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                let attr_value_check = self.element.get_attribute_by_name(DOMString::from("step"))
                    .map(|s| s.Value());
                match attr_value_check {
                    Some(attr_value) => {
                        let step: f32 = attr_value.parse().unwrap();
                        let html_input_element = self.element.downcast::<HTMLInputElement>().unwrap();
                        let input_value_check = html_input_element.get_value_for_validation();
                        match input_value_check {
                            Some(input_value) => {
                                let text_value: f32 = input_value.parse().unwrap();
                                if text_value % step == 0.0_f32 {
                                    return false;
                                }
                                else {
                                    println!("Error - StepMismatch html input element");
                                    return true;
                                }
                            },
                            None => {
                                return false;
                            }
                        }
                    },
                    None => {
                        return false;
                    }
                }
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
            },
            NodeTypeId::Element(_)  => {
            }
            NodeTypeId::CharacterData(_)  => {
            }
            NodeTypeId::Document(_)  => {
            }
            NodeTypeId::DocumentFragment  => {
            }
            NodeTypeId::DocumentType  => {
            }
        };
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-badinput
    fn BadInput(&self) -> bool {
        match self.element.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
            },
            NodeTypeId::Element(_)  => {
            }
            NodeTypeId::CharacterData(_)  => {
            }
            NodeTypeId::Document(_)  => {
            }
            NodeTypeId::DocumentFragment  => {
            }
            NodeTypeId::DocumentType  => {
            }
        };
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-customerror
    fn CustomError(&self) -> bool {
        let attr_value_check = self.element.get_attribute_by_name(DOMString::from("validationMessage"))
                    .map(|s| s.Value());


        if attr_value_check.is_some() {
            return true;
        }
        else {
            return false;
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-valid
    fn Valid(&self) -> bool {
        return !(
            self.ValueMissing()|
            self.TypeMismatch()|
            self.PatternMismatch()|
            self.TooLong()|
            self.TooShort()|
            self.RangeUnderflow()|
            self.RangeOverflow()|
            self.StepMismatch()|
            self.BadInput()|
            self.CustomError());
    }
}
