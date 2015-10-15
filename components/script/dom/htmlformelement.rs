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
use dom::bindings::codegen::InheritTypes::{ElementBase, ElementCast};
use dom::bindings::codegen::InheritTypes::{ElementTypeId, EventTargetCast};
use dom::bindings::codegen::InheritTypes::{HTMLDataListElementCast, HTMLElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLElementTypeId, HTMLFormElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLInputElementCast, HTMLTextAreaElementCast};
use dom::bindings::codegen::InheritTypes::{NodeCast, NodeTypeId};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Root};
use dom::bindings::utils::Reflectable;
use dom::document::Document;
use dom::element::Element;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::htmlbuttonelement::{HTMLButtonElement};
use dom::htmlelement::HTMLElement;
use dom::htmlinputelement::HTMLInputElement;
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
    make_url_or_base_getter!(Action);

    // https://html.spec.whatwg.org/multipage/#dom-fs-action
    make_setter!(SetAction, "action");

    // https://html.spec.whatwg.org/multipage/#dom-form-autocomplete
    make_enumerated_getter!(Autocomplete, "on", ("off"));

    // https://html.spec.whatwg.org/multipage/#dom-form-autocomplete
    make_setter!(SetAutocomplete, "autocomplete");

    // https://html.spec.whatwg.org/multipage/#dom-fs-enctype
    make_enumerated_getter!(Enctype, "application/x-www-form-urlencoded", ("text/plain") | ("multipart/form-data"));

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
    make_enumerated_getter!(Method, "get", ("post") | ("dialog"));

    // https://html.spec.whatwg.org/multipage/#dom-fs-method
    make_setter!(SetMethod, "method");

    // https://html.spec.whatwg.org/multipage/#dom-form-name
    make_getter!(Name);

    // https://html.spec.whatwg.org/multipage/#dom-form-name
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-fs-novalidate
    make_bool_getter!(NoValidate);

    // https://html.spec.whatwg.org/multipage/#dom-fs-novalidate
    make_bool_setter!(SetNoValidate, "novalidate");

    // https://html.spec.whatwg.org/multipage/#dom-fs-target
    make_getter!(Target);

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

#[derive(Copy, Clone, HeapSizeOf)]
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
    pub fn submit(&self, _submit_method_flag: SubmittedFrom, submitter: FormSubmitter) {
        // Step 1
        let doc = document_from_node(self);
        let win = window_from_node(self);
        let base = doc.r().url();
        // TODO: Handle browsing contexts
        // TODO: Handle validation
        let event = Event::new(GlobalRef::Window(win.r()),
                               "submit".to_owned(),
                               EventBubbles::Bubbles,
                               EventCancelable::Cancelable);
        let target = EventTargetCast::from_ref(self);
        event.r().fire(target);
        if event.r().DefaultPrevented() {
            return;
        }
        // Step 6
        let form_data = self.get_form_dataset(Some(submitter));
        // Step 7-8
        let mut action = submitter.action();
        if action.is_empty() {
            action = base.serialize();
        }
        // TODO: Resolve the url relative to the submitter element
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
        win.r().main_thread_script_chan().send(MainThreadScriptMsg::Navigate(
            win.r().pipeline(), load_data)).unwrap();
    }

    fn get_unclean_dataset<'a>(&self, submitter: Option<FormSubmitter<'a>>) -> Vec<FormDatum> {
        let node = NodeCast::from_ref(self);
        // TODO: This is an incorrect way of getting controls owned
        //       by the form, but good enough until html5ever lands
        node.traverse_preorder().filter_map(|child| {
            match ElementCast::to_ref(child.r()) {
                Some(el) if !el.get_disabled_state() => (),
                _ => return None,
            }

            if child.r().ancestors()
                        .any(|a| HTMLDataListElementCast::to_root(a).is_some()) {
                return None;
            }
            match child.r().type_id() {
                NodeTypeId::Element(ElementTypeId::HTMLElement(element)) => {
                    match element {
                        HTMLElementTypeId::HTMLInputElement => {
                            let input = HTMLInputElementCast::to_ref(child.r()).unwrap();
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

    pub fn get_form_dataset<'a>(&self, submitter: Option<FormSubmitter<'a>>) -> Vec<FormDatum> {
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
            buf
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
                               "reset".to_owned(),
                               EventBubbles::Bubbles,
                               EventCancelable::Cancelable);
        let target = EventTargetCast::from_ref(self);
        event.r().fire(target);
        if event.r().DefaultPrevented() {
            return;
        }

        let node = NodeCast::from_ref(self);

        // TODO: This is an incorrect way of getting controls owned
        //       by the form, but good enough until html5ever lands
        for child in node.traverse_preorder() {
            match child.r().type_id() {
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                    let input = HTMLInputElementCast::to_ref(child.r()).unwrap();
                    input.reset()
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
                    let textarea = HTMLTextAreaElementCast::to_ref(child.r()).unwrap();
                    textarea.reset()
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
}

pub trait FormControl: ElementBase + Reflectable {
    // FIXME: This is wrong (https://github.com/servo/servo/issues/3553)
    //        but we need html5ever to do it correctly
    fn form_owner(&self) -> Option<Root<HTMLFormElement>> {
        // https://html.spec.whatwg.org/multipage/#reset-the-form-owner
        let elem = self.to_element();
        let owner = elem.get_string_attribute(&atom!("form"));
        if !owner.is_empty() {
            let doc = document_from_node(elem);
            let owner = doc.r().GetElementById(owner);
            match owner {
                Some(ref o) => {
                    let maybe_form = HTMLFormElementCast::to_ref(o.r());
                    if maybe_form.is_some() {
                        return maybe_form.map(Root::from_ref);
                    }
                },
                _ => ()
            }
        }
        let node = NodeCast::from_ref(elem);
        for ancestor in node.ancestors() {
            if let Some(ancestor) = HTMLFormElementCast::to_ref(ancestor.r()) {
                return Some(Root::from_ref(ancestor))
            }
        }
        None
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
            self.form_owner().map_or("".to_owned(), |t| owner(t.r()))
        }
    }

    fn to_element(&self) -> &Element {
        ElementCast::from_ref(self)
    }
}

impl VirtualMethods for HTMLFormElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        Some(HTMLElementCast::from_ref(self) as &VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("name") => AttrValue::from_atomic(value),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}
