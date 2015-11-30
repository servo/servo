/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::AttrValue;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::HTMLButtonElementBinding::HTMLButtonElementMethods;
use dom::bindings::codegen::Bindings::HTMLFormElementBinding;
use dom::bindings::codegen::Bindings::HTMLFormElementBinding::HTMLFormElementMethods;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding::HTMLTextAreaElementMethods;
use dom::bindings::conversions::DerivedFrom;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use dom::bindings::js::{Root};
use dom::bindings::reflector::Reflectable;
use dom::document::Document;
use dom::element::Element;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::htmlbuttonelement::HTMLButtonElement;
use dom::htmldatalistelement::HTMLDataListElement;
use dom::htmlelement::HTMLElement;
use dom::htmlinputelement::HTMLInputElement;
use dom::htmlobjectelement::HTMLObjectElement;
use dom::htmlselectelement::HTMLSelectElement;
use dom::htmltextareaelement::HTMLTextAreaElement;
use dom::node::{Node, document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use hyper::header::ContentType;
use hyper::method::Method;
use hyper::mime;
use msg::constellation_msg::LoadData;
use script_task::{MainThreadScriptMsg, ScriptChan};
use std::borrow::ToOwned;
use std::cell::Cell;
use string_cache::Atom;
use url::UrlParser;
use url::form_urlencoded::serialize;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLFormElement {
    htmlelement: HTMLElement,
    marked_for_reset: Cell<bool>,
}

impl PartialEq for HTMLFormElement {
    fn eq(&self, other: &HTMLFormElement) -> bool {
        self as *const HTMLFormElement == &*other
    }
}

impl HTMLFormElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLFormElement {
        HTMLFormElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document),
            marked_for_reset: Cell::new(false),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLFormElement> {
        let element = HTMLFormElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLFormElementBinding::Wrap)
    }
}

impl HTMLFormElementMethods for HTMLFormElement {
    // https://html.spec.whatwg.org/multipage/#dom-form-acceptcharset
    make_getter!(AcceptCharset, "accept-charset");

    // https://html.spec.whatwg.org/multipage/#dom-form-acceptcharset
    make_setter!(SetAcceptCharset, "accept-charset");

    // https://html.spec.whatwg.org/multipage/#dom-fs-action
    make_url_or_base_getter!(Action, "action");

    // https://html.spec.whatwg.org/multipage/#dom-fs-action
    make_setter!(SetAction, "action");

    // https://html.spec.whatwg.org/multipage/#dom-form-autocomplete
    make_enumerated_getter!(Autocomplete, "autocomplete", "on", ("off"));

    // https://html.spec.whatwg.org/multipage/#dom-form-autocomplete
    make_setter!(SetAutocomplete, "autocomplete");

    // https://html.spec.whatwg.org/multipage/#dom-fs-enctype
    make_enumerated_getter!(Enctype,
                            "enctype",
                            "application/x-www-form-urlencoded",
                            ("text/plain") | ("multipart/form-data"));

    // https://html.spec.whatwg.org/multipage/#dom-fs-enctype
    make_setter!(SetEnctype, "enctype");

    // https://html.spec.whatwg.org/multipage/#dom-fs-encoding
    fn Encoding(&self) -> DOMString {
        self.Enctype()
    }

    // https://html.spec.whatwg.org/multipage/#dom-fs-encoding
    fn SetEncoding(&self, value: DOMString) {
        self.SetEnctype(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-fs-method
    make_enumerated_getter!(Method, "method", "get", ("post") | ("dialog"));

    // https://html.spec.whatwg.org/multipage/#dom-fs-method
    make_setter!(SetMethod, "method");

    // https://html.spec.whatwg.org/multipage/#dom-form-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-form-name
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-fs-novalidate
    make_bool_getter!(NoValidate, "novalidate");

    // https://html.spec.whatwg.org/multipage/#dom-fs-novalidate
    make_bool_setter!(SetNoValidate, "novalidate");

    // https://html.spec.whatwg.org/multipage/#dom-fs-target
    make_getter!(Target, "target");

    // https://html.spec.whatwg.org/multipage/#dom-fs-target
    make_setter!(SetTarget, "target");

    // https://html.spec.whatwg.org/multipage/#the-form-element:concept-form-submit
    fn Submit(&self) {
        self.submit(SubmittedFrom::FromFormSubmitMethod, FormSubmitter::FormElement(self));
    }

    // https://html.spec.whatwg.org/multipage/#dom-form-reset
    fn Reset(&self) {
        self.reset(ResetFrom::FromFormResetMethod);
    }
}

#[derive(Copy, Clone, HeapSizeOf, PartialEq)]
pub enum SubmittedFrom {
    FromFormSubmitMethod,
    NotFromFormSubmitMethod
}

#[derive(Copy, Clone, HeapSizeOf)]
pub enum ResetFrom {
    FromFormResetMethod,
    NotFromFormResetMethod
}


impl HTMLFormElement {
    /// [Form submission](https://html.spec.whatwg.org/multipage/#concept-form-submit)
    pub fn submit(&self, submit_method_flag: SubmittedFrom, submitter: FormSubmitter) {
        // Step 1
        let doc = document_from_node(self);
        let win = window_from_node(self);
        let base = doc.url();
        // TODO: Handle browsing contexts
        // Step 4
        if submit_method_flag == SubmittedFrom::NotFromFormSubmitMethod
           && !submitter.no_validate(self)
        {
            if self.interactive_validation().is_err() {
                let event = Event::new(GlobalRef::Window(win.r()),
                                       DOMString::from("invalid"),
                                       EventBubbles::DoesNotBubble,
                                       EventCancelable::NotCancelable);
                event.fire(self.upcast());
                return;
            }
        }
        // Step 5
        if submit_method_flag == SubmittedFrom::NotFromFormSubmitMethod {
            let event = Event::new(GlobalRef::Window(win.r()),
                                   DOMString::from("submit"),
                                   EventBubbles::Bubbles,
                                   EventCancelable::Cancelable);
            event.fire(self.upcast());
            if event.DefaultPrevented() {
                return;
            }
        }
        // Step 6
        let form_data = self.get_form_dataset(Some(submitter));
        // Step 7
        let mut action = submitter.action();
        // Step 8
        if action.is_empty() {
            action = DOMString::from(base.serialize());
        }
        // Step 9
        action = match UrlParser::new().base_url(base).parse(&*action) {
            Ok(url) => DOMString::from(url.serialize()),
            // FIXME: Report error?
            Err(err) => return
        };
        // Step 10-15
        let action_components =
            UrlParser::new().base_url(base).parse(&action).unwrap_or((*base).clone());
        let _action = action_components.serialize();
        let scheme = action_components.scheme.clone();
        let enctype = submitter.enctype();
        let method = submitter.method();
        let _target = submitter.target();
        // TODO: Handle browsing contexts, partially loaded documents (step 16-17)

        let mut load_data = LoadData::new(action_components);

        let parsed_data = match enctype {
            FormEncType::UrlEncoded => {
                let mime: mime::Mime = "application/x-www-form-urlencoded".parse().unwrap();
                load_data.headers.set(ContentType(mime));
                serialize(form_data.iter().map(|d| (&*d.name, &*d.value)))
            }
            _ => "".to_owned() // TODO: Add serializers for the other encoding types
        };

        // Step 18
        match (&*scheme, method) {
            (_, FormMethod::FormDialog) => return, // Unimplemented
            ("http", FormMethod::FormGet) | ("https", FormMethod::FormGet) => {
                load_data.url.query = Some(parsed_data);
            },
            ("http", FormMethod::FormPost) | ("https", FormMethod::FormPost) => {
                load_data.method = Method::Post;
                load_data.data = Some(parsed_data.into_bytes());
            },
            // https://html.spec.whatwg.org/multipage/#submit-get-action
            ("ftp", _) | ("javascript", _) | ("data", FormMethod::FormGet) => (),
            _ => return // Unimplemented (data and mailto)
        }

        // This is wrong. https://html.spec.whatwg.org/multipage/#planned-navigation
        win.main_thread_script_chan().send(MainThreadScriptMsg::Navigate(
            win.pipeline(), load_data)).unwrap();
    }

    /// Interactively validate the constraints of form elements
    /// https://html.spec.whatwg.org/multipage/#interactively-validate-the-constraints
    fn interactive_validation(&self) -> Result<(), ()> {
        // Step 1
        let result = self.static_validation();
        // Step 2
        if result.is_ok() { return Ok(()); }
        // Step 3
        let unhandled_invalid_controls = result.unwrap_err();
        // TODO: Report the problems with the constraints of at least one of
        //       the elements given in unhandled invalid controls to the user
        // Step 4
        Err(())
    }

    /// Statitically validate the constraints of form elements
    /// https://html.spec.whatwg.org/multipage/#statically-validate-the-constraints
    fn static_validation(&self) -> Result<(), Vec<FormSubmittableElement>> {
        let node = self.upcast::<Node>();
        // TODO: This is an incorrect way of getting controls owned by the
        //       form, refactor this when html5ever's form owner PR lands
        // Step 1-3
        let invalid_controls = node.traverse_preorder().filter_map(|field| {
            let el = field.downcast::<Element>().unwrap();
            if field.ancestors()
                    .any(|a| Root::downcast::<HTMLDataListElement>(a).is_some())
                // XXXKiChjang this may be wrong, this is not checking the ancestor
                // elements to find whether an HTMLFieldSetElement exists and is disabled
               || el.get_disabled_state() {
                return None;
            }

            match field.type_id() {
                NodeTypeId::Element(ElementTypeId::HTMLElement(element)) => {
                    match element {
                        HTMLElementTypeId::HTMLButtonElement => {
                            let button = field.downcast::<HTMLButtonElement>().unwrap();
                            // Substep 1
                    // https://html.spec.whatwg.org/multipage/#the-button-element:barred-from-constraint-validation
                            if button.Type() != "submit" { return None; }
                            // Substep 2
                            // TODO: Check constraints on HTMLButtonElement
                            // Substep 3
                            Some(FormSubmittableElement::ButtonElement(Root::from_ref(&*button)))
                        }
                        HTMLElementTypeId::HTMLInputElement => {
                            let input = field.downcast::<HTMLInputElement>().unwrap();
                            // Substep 1
                            // https://html.spec.whatwg.org/multipage/#candidate-for-constraint-validation
                            if input.type_() == atom!("hidden")
                               || input.type_() == atom!("reset")
                               || input.type_() == atom!("button")
                               || input.ReadOnly() { return None; }
                            // Substep 2
                            // TODO: Check constraints on HTMLInputElement
                            // Substep 3
                            Some(FormSubmittableElement::InputElement(Root::from_ref(&*input)))
                        }
                        HTMLElementTypeId::HTMLSelectElement => {
                            let select = field.downcast::<HTMLSelectElement>().unwrap();
                            // Substep 1 not necessary, HTMLSelectElements are not barred from constraint validation
                            // Substep 2
                            // TODO: Check constraints on HTMLSelectElement
                            // Substep 3
                            Some(FormSubmittableElement::SelectElement(Root::from_ref(&*select)))
                        }
                        HTMLElementTypeId::HTMLTextAreaElement => {
                            let textarea = field.downcast::<HTMLTextAreaElement>().unwrap();
                            // Substep 1
                    // https://html.spec.whatwg.org/multipage/#the-textarea-element:barred-from-constraint-validation
                            if textarea.ReadOnly() { return None; }
                            // Substep 2
                            // TODO: Check constraints on HTMLTextAreaElement
                            // Substep 3
                            Some(FormSubmittableElement::TextAreaElement(Root::from_ref(&*textarea)))
                        }
                        _ => None
                    }
                }
                _ => None
            }
        }).collect::<Vec<FormSubmittableElement>>();
        // Step 4
        if invalid_controls.is_empty() { return Ok(()); }
        // Step 5-6
        let win = window_from_node(self);
        let unhandled_invalid_controls = invalid_controls.into_iter().filter_map(|field| {
            let event = Event::new(GlobalRef::Window(win.r()),
                                   DOMString::from("invalid"),
                                   EventBubbles::DoesNotBubble,
                                   EventCancelable::Cancelable);
            event.fire(field.as_event_target());
            if !event.DefaultPrevented() { return Some(field); }
            None
        }).collect::<Vec<FormSubmittableElement>>();
        // Step 7
        Err(unhandled_invalid_controls)
    }

    fn get_unclean_dataset(&self, submitter: Option<FormSubmitter>) -> Vec<FormDatum> {
        let node = self.upcast::<Node>();
        // TODO: This is an incorrect way of getting controls owned
        //       by the form, but good enough until html5ever lands
        node.traverse_preorder().filter_map(|child| {
            match child.downcast::<Element>() {
                Some(el) if !el.get_disabled_state() => (),
                _ => return None,
            }

            if child.ancestors()
                    .any(|a| Root::downcast::<HTMLDataListElement>(a).is_some()) {
                return None;
            }
            match child.type_id() {
                NodeTypeId::Element(ElementTypeId::HTMLElement(element)) => {
                    match element {
                        HTMLElementTypeId::HTMLInputElement => {
                            let input = child.downcast::<HTMLInputElement>().unwrap();
                            input.get_form_datum(submitter)
                        }
                        HTMLElementTypeId::HTMLButtonElement |
                        HTMLElementTypeId::HTMLSelectElement |
                        HTMLElementTypeId::HTMLObjectElement |
                        HTMLElementTypeId::HTMLTextAreaElement => {
                            // Unimplemented
                            None
                        }
                        _ => None
                    }
                }
                _ => None
            }
        }).collect()
        // TODO: Handle `dirnames` (needs directionality support)
        //       https://html.spec.whatwg.org/multipage/#the-directionality
    }

    pub fn get_form_dataset(&self, submitter: Option<FormSubmitter>) -> Vec<FormDatum> {
        fn clean_crlf(s: &str) -> DOMString {
            // https://html.spec.whatwg.org/multipage/#constructing-the-form-data-set
            // Step 4
            let mut buf = "".to_owned();
            let mut prev = ' ';
            for ch in s.chars() {
                match ch {
                    '\n' if prev != '\r' => {
                        buf.push('\r');
                        buf.push('\n');
                    },
                    '\n' => {
                        buf.push('\n');
                    },
                    // This character isn't LF but is
                    // preceded by CR
                    _ if prev == '\r' => {
                        buf.push('\r');
                        buf.push('\n');
                        buf.push(ch);
                    },
                    _ => buf.push(ch)
                };
                prev = ch;
            }
            // In case the last character was CR
            if prev == '\r' {
                buf.push('\n');
            }
            DOMString::from(buf)
        }

        let mut ret = self.get_unclean_dataset(submitter);
        // https://html.spec.whatwg.org/multipage/#constructing-the-form-data-set
        // Step 4
        for datum in &mut ret {
            match &*datum.ty {
                "file" | "textarea" => (),
                _ => {
                    datum.name = clean_crlf(&datum.name);
                    datum.value = clean_crlf(&datum.value);
                }
            }
        };
        ret
    }

    pub fn reset(&self, _reset_method_flag: ResetFrom) {
        // https://html.spec.whatwg.org/multipage/#locked-for-reset
        if self.marked_for_reset.get() {
            return;
        } else {
            self.marked_for_reset.set(true);
        }

        let win = window_from_node(self);
        let event = Event::new(GlobalRef::Window(win.r()),
                               DOMString::from("reset"),
                               EventBubbles::Bubbles,
                               EventCancelable::Cancelable);
        event.fire(self.upcast());
        if event.DefaultPrevented() {
            return;
        }

        // TODO: This is an incorrect way of getting controls owned
        //       by the form, but good enough until html5ever lands
        for child in self.upcast::<Node>().traverse_preorder() {
            match child.type_id() {
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                    child.downcast::<HTMLInputElement>().unwrap().reset();
                }
                // TODO HTMLKeygenElement unimplemented
                //NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLKeygenElement)) => {
                //    // Unimplemented
                //    {}
                //}
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
                    // Unimplemented
                    {}
                }
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
                    child.downcast::<HTMLTextAreaElement>().unwrap().reset();
                }
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOutputElement)) => {
                    // Unimplemented
                    {}
                }
                _ => {}
            }
        };
        self.marked_for_reset.set(false);
    }
}

// TODO: add file support
#[derive(HeapSizeOf)]
pub struct FormDatum {
    pub ty: DOMString,
    pub name: DOMString,
    pub value: DOMString
}

#[derive(Copy, Clone, HeapSizeOf)]
pub enum FormEncType {
    TextPlainEncoded,
    UrlEncoded,
    FormDataEncoded
}

#[derive(Copy, Clone, HeapSizeOf)]
pub enum FormMethod {
    FormGet,
    FormPost,
    FormDialog
}

#[derive(HeapSizeOf)]
pub enum FormSubmittableElement {
    ButtonElement(Root<HTMLButtonElement>),
    InputElement(Root<HTMLInputElement>),
    // TODO: HTMLKeygenElement unimplemented
    // KeygenElement(&'a HTMLKeygenElement),
    ObjectElement(Root<HTMLObjectElement>),
    SelectElement(Root<HTMLSelectElement>),
    TextAreaElement(Root<HTMLTextAreaElement>)
}

impl FormSubmittableElement {
    fn as_event_target(&self) -> &EventTarget {
        match *self {
            FormSubmittableElement::ButtonElement(ref button) =>  button.r().upcast(),
            FormSubmittableElement::InputElement(ref input) => input.r().upcast(),
            FormSubmittableElement::ObjectElement(ref object) => object.r().upcast(),
            FormSubmittableElement::SelectElement(ref select) => select.r().upcast(),
            FormSubmittableElement::TextAreaElement(ref textarea) => textarea.r().upcast()
        }
    }
}

#[derive(Copy, Clone, HeapSizeOf)]
pub enum FormSubmitter<'a> {
    FormElement(&'a HTMLFormElement),
    InputElement(&'a HTMLInputElement),
    ButtonElement(&'a HTMLButtonElement)
    // TODO: image submit, etc etc
}

impl<'a> FormSubmitter<'a> {
    fn action(&self) -> DOMString {
        match *self {
            FormSubmitter::FormElement(form) => form.Action(),
            FormSubmitter::InputElement(input_element) => {
                input_element.get_form_attribute(&atom!("formaction"),
                                                 |i| i.FormAction(),
                                                 |f| f.Action())
            },
            FormSubmitter::ButtonElement(button_element) => {
                button_element.get_form_attribute(&atom!("formaction"),
                                                  |i| i.FormAction(),
                                                  |f| f.Action())
            }
        }
    }

    fn enctype(&self) -> FormEncType {
        let attr = match *self {
            FormSubmitter::FormElement(form) => form.Enctype(),
            FormSubmitter::InputElement(input_element) => {
                input_element.get_form_attribute(&atom!("formenctype"),
                                                 |i| i.FormEnctype(),
                                                 |f| f.Enctype())
            },
            FormSubmitter::ButtonElement(button_element) => {
                button_element.get_form_attribute(&atom!("formenctype"),
                                                  |i| i.FormAction(),
                                                  |f| f.Action())
            }
        };
        match &*attr {
            "multipart/form-data" => FormEncType::FormDataEncoded,
            "text/plain" => FormEncType::TextPlainEncoded,
            // https://html.spec.whatwg.org/multipage/#attr-fs-enctype
            // urlencoded is the default
            _ => FormEncType::UrlEncoded
        }
    }

    fn method(&self) -> FormMethod {
        let attr = match *self {
            FormSubmitter::FormElement(form) => form.Method(),
            FormSubmitter::InputElement(input_element) => {
                input_element.get_form_attribute(&atom!("formmethod"),
                                                 |i| i.FormMethod(),
                                                 |f| f.Method())
            },
            FormSubmitter::ButtonElement(button_element) => {
                button_element.get_form_attribute(&atom!("formmethod"),
                                                  |i| i.FormAction(),
                                                  |f| f.Action())
            }
        };
        match &*attr {
            "dialog" => FormMethod::FormDialog,
            "post" => FormMethod::FormPost,
            _ => FormMethod::FormGet
        }
    }

    fn target(&self) -> DOMString {
        match *self {
            FormSubmitter::FormElement(form) => form.Target(),
            FormSubmitter::InputElement(input_element) => {
                input_element.get_form_attribute(&atom!("formtarget"),
                                                 |i| i.FormTarget(),
                                                 |f| f.Target())
            },
            FormSubmitter::ButtonElement(button_element) => {
                button_element.get_form_attribute(&atom!("formtarget"),
                                                  |i| i.FormAction(),
                                                  |f| f.Action())
            }
        }
    }

    fn no_validate(&self, form_owner: &HTMLFormElement) -> bool {
        match *self {
            FormSubmitter::FormElement(form) => form.NoValidate(),
            FormSubmitter::InputElement(input_element) => {
                input_element.get_form_boolean_attribute(&atom!("formnovalidate"),
                                                 |i| i.FormNoValidate(),
                                                 |f| f.NoValidate())
            }
            FormSubmitter::ButtonElement(button_element) => {
                button_element.get_form_boolean_attribute(&atom!("formnovalidate"),
                                                  |i| i.FormNoValidate(),
                                                  |f| f.NoValidate())
            }
        }
    }
}

pub trait FormControl: DerivedFrom<Element> + Reflectable {
    // FIXME: This is wrong (https://github.com/servo/servo/issues/3553)
    //        but we need html5ever to do it correctly
    fn form_owner(&self) -> Option<Root<HTMLFormElement>> {
        // https://html.spec.whatwg.org/multipage/#reset-the-form-owner
        let elem = self.to_element();
        let owner = elem.get_string_attribute(&atom!("form"));
        if !owner.is_empty() {
            let doc = document_from_node(elem);
            let owner = doc.GetElementById(owner);
            match owner {
                Some(ref o) => {
                    let maybe_form = o.downcast::<HTMLFormElement>();
                    if maybe_form.is_some() {
                        return maybe_form.map(Root::from_ref);
                    }
                },
                _ => ()
            }
        }
        elem.upcast::<Node>().ancestors().filter_map(Root::downcast).next()
    }

    fn get_form_attribute<InputFn, OwnerFn>(&self,
                                            attr: &Atom,
                                            input: InputFn,
                                            owner: OwnerFn)
                                            -> DOMString
        where InputFn: Fn(&Self) -> DOMString,
              OwnerFn: Fn(&HTMLFormElement) -> DOMString
    {
        if self.to_element().has_attribute(attr) {
            input(self)
        } else {
            self.form_owner().map_or(DOMString::new(), |t| owner(t.r()))
        }
    }

    fn get_form_boolean_attribute<InputFn, OwnerFn>(&self,
                                            attr: &Atom,
                                            input: InputFn,
                                            owner: OwnerFn)
                                            -> bool
        where InputFn: Fn(&Self) -> bool,
              OwnerFn: Fn(&HTMLFormElement) -> bool
    {
        if self.to_element().has_attribute(attr) {
            input(self)
        } else {
            self.form_owner().map_or(false, |t| owner(t.r()))
        }
    }

    fn to_element(&self) -> &Element {
        self.upcast()
    }
}

impl VirtualMethods for HTMLFormElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("name") => AttrValue::from_atomic(value),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}
