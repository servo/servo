/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::HTMLFormElementBinding;
use dom::bindings::codegen::Bindings::HTMLFormElementBinding::HTMLFormElementMethods;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use dom::bindings::codegen::Bindings::HTMLButtonElementBinding::HTMLButtonElementMethods;
use dom::bindings::codegen::InheritTypes::{EventTargetCast, HTMLFormElementDerived, NodeCast};
use dom::bindings::codegen::InheritTypes::{HTMLInputElementCast, HTMLTextAreaElementCast, HTMLFormElementCast};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary, OptionalRootable};
use dom::document::{Document, DocumentHelpers};
use dom::element::{Element, AttributeHandlers};
use dom::event::{Event, EventHelpers, EventBubbles, EventCancelable};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::htmlinputelement::{HTMLInputElement, HTMLInputElementHelpers};
use dom::htmlbuttonelement::{HTMLButtonElement};
use dom::htmltextareaelement::{HTMLTextAreaElement, HTMLTextAreaElementHelpers};
use dom::node::{Node, NodeHelpers, NodeTypeId, document_from_node, window_from_node};
use hyper::method::Method;
use hyper::header::ContentType;
use hyper::mime;
use msg::constellation_msg::LoadData;
use util::str::DOMString;
use script_task::{ScriptChan, ScriptMsg};
use std::ascii::OwnedAsciiExt;
use url::UrlParser;
use url::form_urlencoded::serialize;
use string_cache::Atom;

use std::borrow::ToOwned;
use std::cell::Cell;

#[dom_struct]
pub struct HTMLFormElement {
    htmlelement: HTMLElement,
    marked_for_reset: Cell<bool>,
}

impl HTMLFormElementDerived for EventTarget {
    fn is_htmlformelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFormElement)))
    }
}

impl HTMLFormElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLFormElement {
        HTMLFormElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLFormElement, localName, prefix, document),
            marked_for_reset: Cell::new(false),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLFormElement> {
        let element = HTMLFormElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLFormElementBinding::Wrap)
    }
}

impl<'a> HTMLFormElementMethods for JSRef<'a, HTMLFormElement> {
    // https://html.spec.whatwg.org/multipage/forms.html#dom-form-acceptcharset
    make_getter!(AcceptCharset, "accept-charset");

    // https://html.spec.whatwg.org/multipage/forms.html#dom-form-acceptcharset
    make_setter!(SetAcceptCharset, "accept-charset");

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-action
    make_url_or_base_getter!(Action);

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-action
    make_setter!(SetAction, "action");

    // https://html.spec.whatwg.org/multipage/forms.html#dom-form-autocomplete
    make_enumerated_getter!(Autocomplete, "on", ("off"));

    // https://html.spec.whatwg.org/multipage/forms.html#dom-form-autocomplete
    make_setter!(SetAutocomplete, "autocomplete");

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-enctype
    make_enumerated_getter!(Enctype, "application/x-www-form-urlencoded", ("text/plain") | ("multipart/form-data"));

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-enctype
    make_setter!(SetEnctype, "enctype");

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-encoding
    fn Encoding(self) -> DOMString {
        self.Enctype()
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-encoding
    fn SetEncoding(self, value: DOMString) {
        self.SetEnctype(value)
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-method
    make_enumerated_getter!(Method, "get", ("post") | ("dialog"));

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-method
    make_setter!(SetMethod, "method");

    // https://html.spec.whatwg.org/multipage/forms.html#dom-form-name
    make_getter!(Name);

    // https://html.spec.whatwg.org/multipage/forms.html#dom-form-name
    make_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-novalidate
    make_bool_getter!(NoValidate);

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-novalidate
    make_bool_setter!(SetNoValidate, "novalidate");

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-target
    make_getter!(Target);

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-target
    make_setter!(SetTarget, "target");

    // https://html.spec.whatwg.org/multipage/forms.html#the-form-element:concept-form-submit
    fn Submit(self) {
        self.submit(SubmittedFrom::FromFormSubmitMethod, FormSubmitter::FormElement(self));
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-form-reset
    fn Reset(self) {
        self.reset(ResetFrom::FromFormResetMethod);
    }
}

#[derive(Copy)]
pub enum SubmittedFrom {
    FromFormSubmitMethod,
    NotFromFormSubmitMethod
}

#[derive(Copy)]
pub enum ResetFrom {
    FromFormResetMethod,
    NotFromFormResetMethod
}

pub trait HTMLFormElementHelpers {
    // https://html.spec.whatwg.org/multipage/forms.html#concept-form-submit
    fn submit(self, submit_method_flag: SubmittedFrom, submitter: FormSubmitter);
    // https://html.spec.whatwg.org/multipage/forms.html#constructing-the-form-data-set
    fn get_form_dataset(self, submitter: Option<FormSubmitter>) -> Vec<FormDatum>;
    // https://html.spec.whatwg.org/multipage/forms.html#dom-form-reset
    fn reset(self, submit_method_flag: ResetFrom);
}

impl<'a> HTMLFormElementHelpers for JSRef<'a, HTMLFormElement> {
    fn submit(self, _submit_method_flag: SubmittedFrom, submitter: FormSubmitter) {
        // Step 1
        let doc = document_from_node(self).root();
        let win = window_from_node(self).root();
        let base = doc.r().url();
        // TODO: Handle browsing contexts
        // TODO: Handle validation
        let event = Event::new(GlobalRef::Window(win.r()),
                               "submit".to_owned(),
                               EventBubbles::Bubbles,
                               EventCancelable::Cancelable).root();
        let target: JSRef<EventTarget> = EventTargetCast::from_ref(self);
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
        let action_components = UrlParser::new().base_url(&base).parse(action.as_slice()).unwrap_or(base);
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
                serialize(form_data.iter().map(|d| (d.name.as_slice(), d.value.as_slice())))
            }
            _ => "".to_owned() // TODO: Add serializers for the other encoding types
        };

        // Step 18
        match (scheme.as_slice(), method) {
            (_, FormMethod::FormDialog) => return, // Unimplemented
            ("http", FormMethod::FormGet) | ("https", FormMethod::FormGet) => {
                load_data.url.query = Some(parsed_data);
            },
            ("http", FormMethod::FormPost) | ("https", FormMethod::FormPost) => {
                load_data.method = Method::Post;
                load_data.data = Some(parsed_data.into_bytes());
            },
            // https://html.spec.whatwg.org/multipage/forms.html#submit-get-action
            ("ftp", _) | ("javascript", _) | ("data", FormMethod::FormGet) => (),
            _ => return // Unimplemented (data and mailto)
        }

        // This is wrong. https://html.spec.whatwg.org/multipage/forms.html#planned-navigation
        win.r().script_chan().send(ScriptMsg::TriggerLoad(win.r().page().id, load_data));
    }

    fn get_form_dataset<'b>(self, submitter: Option<FormSubmitter<'b>>) -> Vec<FormDatum> {
        fn clean_crlf(s: &str) -> DOMString {
            // https://html.spec.whatwg.org/multipage/forms.html#constructing-the-form-data-set
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

        let node: JSRef<Node> = NodeCast::from_ref(self);
        // TODO: This is an incorrect way of getting controls owned
        //       by the form, but good enough until html5ever lands
        let data_set = node.traverse_preorder().filter_map(|child| {
            if child.get_disabled_state() {
                return None;
            }
            if child.ancestors().any(|a| a.type_id() == NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLDataListElement))) {
                return None;
            }
            // XXXManishearth don't include it if it is a button but not the submitter
            match child.type_id() {
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                    let input: JSRef<HTMLInputElement> = HTMLInputElementCast::to_ref(child).unwrap();
                    let ty = input.Type();
                    let name = input.Name();
                    match ty.as_slice() {
                        "radio" | "checkbox" => {
                            if !input.Checked() || name.is_empty() {
                                return None;
                            }
                        },
                        "image" => (),
                        _ => {
                            if name.is_empty() {
                                return None;
                            }
                        }
                    }

                    let mut value = input.Value();
                    let is_submitter = match submitter {
                        Some(FormSubmitter::InputElement(s)) => {
                            input == s
                        },
                        _ => false
                    };
                    match ty.as_slice() {
                        "image" => None, // Unimplemented
                        "radio" | "checkbox" => {
                            if value.is_empty() {
                                value = "on".to_owned();
                            }
                            Some(FormDatum {
                                ty: ty,
                                name: name,
                                value: value
                            })
                        },
                        // Discard buttons which are not the submitter
                        "submit" | "button" | "reset" if !is_submitter => None,
                        "file" => None, // Unimplemented
                        _ => Some(FormDatum {
                            ty: ty,
                            name: name,
                            value: input.Value()
                        })
                    }
                }
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) => {
                    // Unimplemented
                    None
                }
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
                    // Unimplemented
                    None
                }
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement)) => {
                    // Unimplemented
                    None
                }
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
                    // Unimplemented
                    None
                }
                _ => None
            }
        });
        // TODO: Handle `dirnames` (needs directionality support)
        //       https://html.spec.whatwg.org/multipage/dom.html#the-directionality
        let mut ret: Vec<FormDatum> = data_set.collect();
        for datum in ret.iter_mut() {
            match datum.ty.as_slice() {
                "file" | "textarea" => (),
                _ => {
                    datum.name = clean_crlf(datum.name.as_slice());
                    datum.value = clean_crlf(datum.value.as_slice());
                }
            }
        };
        ret
    }

    fn reset(self, _reset_method_flag: ResetFrom) {
        // https://html.spec.whatwg.org/multipage/forms.html#locked-for-reset
        if self.marked_for_reset.get() {
            return;
        } else {
            self.marked_for_reset.set(true);
        }

        let win = window_from_node(self).root();
        let event = Event::new(GlobalRef::Window(win.r()),
                               "reset".to_owned(),
                               EventBubbles::Bubbles,
                               EventCancelable::Cancelable).root();
        let target: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        event.r().fire(target);
        if event.r().DefaultPrevented() {
            return;
        }

        let node: JSRef<Node> = NodeCast::from_ref(self);

        // TODO: This is an incorrect way of getting controls owned
        //       by the form, but good enough until html5ever lands
        for child in node.traverse_preorder() {
            match child.type_id() {
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                    let input: JSRef<HTMLInputElement> = HTMLInputElementCast::to_ref(child)
                                                                               .unwrap();
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
                    let textarea: JSRef<HTMLTextAreaElement> = HTMLTextAreaElementCast::to_ref(child)
                                                                                        .unwrap();
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
pub struct FormDatum {
    pub ty: DOMString,
    pub name: DOMString,
    pub value: DOMString
}

#[derive(Copy)]
pub enum FormEncType {
    TextPlainEncoded,
    UrlEncoded,
    FormDataEncoded
}

#[derive(Copy)]
pub enum FormMethod {
    FormGet,
    FormPost,
    FormDialog
}

#[derive(Copy)]
pub enum FormSubmitter<'a> {
    FormElement(JSRef<'a, HTMLFormElement>),
    InputElement(JSRef<'a, HTMLInputElement>),
    ButtonElement(JSRef<'a, HTMLButtonElement>)
    // TODO: image submit, etc etc
}

impl<'a> FormSubmitter<'a> {
    fn action(&self) -> DOMString {
        match *self {
            FormSubmitter::FormElement(form) => form.Action(),
            FormSubmitter::InputElement(input_element) => {
                // FIXME(pcwalton): Make this a static atom.
                input_element.get_form_attribute(&Atom::from_slice("formaction"),
                                                 |i| i.FormAction(),
                                                 |f| f.Action())
            },
            FormSubmitter::ButtonElement(button_element) => {
                button_element.get_form_attribute(&Atom::from_slice("formaction"),
                                                  |i| i.FormAction(),
                                                  |f| f.Action())
            }
        }
    }

    fn enctype(&self) -> FormEncType {
        let attr = match *self {
            FormSubmitter::FormElement(form) => form.Enctype(),
            FormSubmitter::InputElement(input_element) => {
                // FIXME(pcwalton): Make this a static atom.
                input_element.get_form_attribute(&Atom::from_slice("formenctype"),
                                                 |i| i.FormEnctype(),
                                                 |f| f.Enctype())
            },
            FormSubmitter::ButtonElement(button_element) => {
                button_element.get_form_attribute(&Atom::from_slice("formenctype"),
                                                  |i| i.FormAction(),
                                                  |f| f.Action())
            }
        };
        match attr.as_slice() {
            "multipart/form-data" => FormEncType::FormDataEncoded,
            "text/plain" => FormEncType::TextPlainEncoded,
            // https://html.spec.whatwg.org/multipage/forms.html#attr-fs-enctype
            // urlencoded is the default
            _ => FormEncType::UrlEncoded
        }
    }

    fn method(&self) -> FormMethod {
        let attr = match *self {
            FormSubmitter::FormElement(form) => form.Method(),
            FormSubmitter::InputElement(input_element) => {
                // FIXME(pcwalton): Make this a static atom.
                input_element.get_form_attribute(&Atom::from_slice("formmethod"),
                                                 |i| i.FormMethod(),
                                                 |f| f.Method())
            },
            FormSubmitter::ButtonElement(button_element) => {
                button_element.get_form_attribute(&Atom::from_slice("formmethod"),
                                                  |i| i.FormAction(),
                                                  |f| f.Action())
            }
        };
        match attr.as_slice() {
            "dialog" => FormMethod::FormDialog,
            "post" => FormMethod::FormPost,
            _ => FormMethod::FormGet
        }
    }

    fn target(&self) -> DOMString {
        match *self {
            FormSubmitter::FormElement(form) => form.Target(),
            FormSubmitter::InputElement(input_element) => {
                // FIXME(pcwalton): Make this a static atom.
                input_element.get_form_attribute(&Atom::from_slice("formtarget"),
                                                 |i| i.FormTarget(),
                                                 |f| f.Target())
            },
            FormSubmitter::ButtonElement(button_element) => {
                button_element.get_form_attribute(&Atom::from_slice("formtarget"),
                                                  |i| i.FormAction(),
                                                  |f| f.Action())
            }
        }
    }
}

pub trait FormControl<'a> : Copy + Sized {
    // FIXME: This is wrong (https://github.com/servo/servo/issues/3553)
    //        but we need html5ever to do it correctly
    fn form_owner(self) -> Option<Temporary<HTMLFormElement>> {
        // https://html.spec.whatwg.org/multipage/forms.html#reset-the-form-owner
        let elem = self.to_element();
        let owner = elem.get_string_attribute(&atom!("form"));
        if !owner.is_empty() {
            let doc = document_from_node(elem).root();
            let owner = doc.r().GetElementById(owner).root();
            match owner {
                Some(o) => {
                    let maybe_form: Option<JSRef<HTMLFormElement>> = HTMLFormElementCast::to_ref(o.r());
                    if maybe_form.is_some() {
                        return maybe_form.map(Temporary::from_rooted);
                    }
                },
                _ => ()
            }
        }
        let node: JSRef<Node> = NodeCast::from_ref(elem);
        node.ancestors().filter_map(|a| HTMLFormElementCast::to_ref(a)).next()
            .map(Temporary::from_rooted)
    }

    fn get_form_attribute<InputFn, OwnerFn>(self,
                                            attr: &Atom,
                                            input: InputFn,
                                            owner: OwnerFn)
                                            -> DOMString
        where InputFn: Fn(Self) -> DOMString,
              OwnerFn: Fn(JSRef<HTMLFormElement>) -> DOMString
    {
        if self.to_element().has_attribute(attr) {
            input(self)
        } else {
            self.form_owner().map_or("".to_owned(), |t| owner(t.root().r()))
        }
    }

    fn to_element(self) -> JSRef<'a, Element>;
}
