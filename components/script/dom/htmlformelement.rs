/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::EventTargetBinding::EventTargetMethods;
use dom::bindings::codegen::Bindings::HTMLFormElementBinding;
use dom::bindings::codegen::Bindings::HTMLFormElementBinding::HTMLFormElementMethods;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, EventTargetCast, HTMLFormElementDerived, NodeCast};
use dom::bindings::codegen::InheritTypes::HTMLInputElementCast;
use dom::bindings::global::Window;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::{Document, DocumentHelpers};
use dom::element::{Element, AttributeHandlers, HTMLFormElementTypeId, HTMLTextAreaElementTypeId, HTMLDataListElementTypeId};
use dom::element::{HTMLInputElementTypeId, HTMLButtonElementTypeId, HTMLObjectElementTypeId, HTMLSelectElementTypeId};
use dom::event::Event;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::htmlinputelement::HTMLInputElement;
use dom::node::{Node, NodeHelpers, ElementNodeTypeId, document_from_node, window_from_node};
use http::method::Post;
use servo_msg::constellation_msg::LoadData;
use servo_util::str::DOMString;
use script_task::{ScriptChan, TriggerLoadMsg};
use std::ascii::OwnedStrAsciiExt;
use std::str::StrSlice;
use url::UrlParser;
use url::form_urlencoded::serialize;

#[jstraceable]
#[must_root]
pub struct HTMLFormElement {
    pub htmlelement: HTMLElement,
}

impl HTMLFormElementDerived for EventTarget {
    fn is_htmlformelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLFormElementTypeId))
    }
}

impl HTMLFormElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLFormElement {
        HTMLFormElement {
            htmlelement: HTMLElement::new_inherited(HTMLFormElementTypeId, localName, prefix, document)
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
    make_getter!(AcceptCharset, "accept-charset")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-form-acceptcharset
    make_setter!(SetAcceptCharset, "accept-charset")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-action
    fn Action(self) -> DOMString {
        let element: JSRef<Element> = ElementCast::from_ref(self);
        let url = element.get_url_attribute("action");
        match url.as_slice() {
            "" => {
                let window = window_from_node(self).root();
                window.get_url().serialize()
            },
            _ => url
        }
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-action
    make_setter!(SetAction, "action")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-form-autocomplete
    fn Autocomplete(self) -> DOMString {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        let ac = elem.get_string_attribute("autocomplete").into_ascii_lower();
        // https://html.spec.whatwg.org/multipage/forms.html#attr-form-autocomplete
        match ac.as_slice() {
            "off" => ac,
            _ => "on".to_string()
        }
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-form-autocomplete
    make_setter!(SetAutocomplete, "autocomplete")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-enctype
    fn Enctype(self) -> DOMString {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        let enctype = elem.get_string_attribute("enctype").into_ascii_lower();
        // https://html.spec.whatwg.org/multipage/forms.html#attr-fs-enctype
        match enctype.as_slice() {
            "text/plain" | "multipart/form-data" => enctype,
            _ => "application/x-www-form-urlencoded".to_string()
        }
    }


    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-enctype
    make_setter!(SetEnctype, "enctype")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-encoding
    fn Encoding(self) -> DOMString {
        self.Enctype()
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-encoding
    fn SetEncoding(self, value: DOMString) {
        self.SetEnctype(value)
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-method
    fn Method(self) -> DOMString {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        let method = elem.get_string_attribute("method").into_ascii_lower();
        // https://html.spec.whatwg.org/multipage/forms.html#attr-fs-method
        match method.as_slice() {
            "post" | "dialog" => method,
            _ => "get".to_string()
        }
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-method
    make_setter!(SetMethod, "method")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-form-name
    make_getter!(Name)

    // https://html.spec.whatwg.org/multipage/forms.html#dom-form-name
    make_setter!(SetName, "name")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-novalidate
    make_bool_getter!(NoValidate)

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-novalidate
    make_bool_setter!(SetNoValidate, "novalidate")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-target
    make_getter!(Target)

    // https://html.spec.whatwg.org/multipage/forms.html#dom-fs-target
    make_setter!(SetTarget, "target")

    // https://html.spec.whatwg.org/multipage/forms.html#the-form-element:concept-form-submit
    fn Submit(self) {
        self.submit(true, FormElement(self));
    }
}

pub trait HTMLFormElementHelpers {
    // https://html.spec.whatwg.org/multipage/forms.html#concept-form-submit
    fn submit(self, from_submit_method: bool, submitter: FormSubmitter);
    // https://html.spec.whatwg.org/multipage/forms.html#constructing-the-form-data-set
    fn get_form_dataset(self, submitter: Option<FormSubmitter>) -> Vec<FormDatum>;
}

impl<'a> HTMLFormElementHelpers for JSRef<'a, HTMLFormElement> {
    fn submit(self, _from_submit_method: bool, submitter: FormSubmitter) {
        // Step 1
        let doc = document_from_node(self).root();
        let win = window_from_node(self).root();
        let base = doc.url();
        // TODO: Handle browsing contexts
        // TODO: Handle validation
        let event = Event::new(&Window(*win),
                               "submit".to_string(),
                               true, true).root();
        let target: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        target.DispatchEvent(*event).ok();
        if event.DefaultPrevented() {
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
        let action_components = UrlParser::new().base_url(base).parse(action.as_slice()).unwrap_or(base.clone());
        let _action = action_components.serialize();
        let scheme = action_components.scheme.clone();
        let enctype = submitter.enctype();
        let method = submitter.method();
        let _target = submitter.target();
        // TODO: Handle browsing contexts, partially loaded documents (step 16-17)

        let parsed_data = match enctype {
            UrlEncoded => serialize(form_data.iter().map(|d| (d.name.as_slice(), d.value.as_slice())), None),
            _ => "".to_string() // TODO: Add serializers for the other encoding types
        };

        let mut load_data = LoadData::new(action_components);
        // Step 18
        match (scheme.as_slice(), method) {
            (_, FormDialog) => return, // Unimplemented
            ("http", FormGet) | ("https", FormGet) => {
                load_data.url.query = Some(parsed_data);
            },
            ("http", FormPost) | ("https", FormPost) => {
                load_data.method = Post;
                load_data.data = Some(parsed_data.into_bytes());
            },
            // https://html.spec.whatwg.org/multipage/forms.html#submit-get-action
            ("ftp", _) | ("javascript", _) | ("data", FormGet) => (),
            _ => return // Unimplemented (data and mailto)
        }

        // This is wrong. https://html.spec.whatwg.org/multipage/forms.html#planned-navigation
        let ScriptChan(ref script_chan) = win.script_chan;
        script_chan.send(TriggerLoadMsg(win.page.id, load_data));
    }

    fn get_form_dataset(self, _submitter: Option<FormSubmitter>) -> Vec<FormDatum> {
        fn clean_crlf(s: &str) -> DOMString {
            // https://html.spec.whatwg.org/multipage/forms.html#constructing-the-form-data-set
            // Step 4
            let mut buf = "".to_string();
            let mut prev = ' ';
            for ch in s.chars() {
                match ch {
                    '\n' if prev != '\r' => {
                        buf.push_char('\r');
                        buf.push_char('\n');
                    },
                    '\n' => {
                        buf.push_char('\n');
                    },
                    // This character isn't LF but is
                    // preceded by CR
                    _ if prev == '\r' => {
                        buf.push_char('\r');
                        buf.push_char('\n');
                        buf.push_char(ch);
                    },
                    _ => buf.push_char(ch)
                };
                prev = ch;
            }
            // In case the last character was CR
            if prev == '\r' {
                buf.push_char('\n');
            }
            buf
        }

        let node: JSRef<Node> = NodeCast::from_ref(self);
        // TODO: This is an incorrect way of getting controls owned
        //       by the form, but good enough until html5ever lands
        let mut data_set = node.traverse_preorder().filter_map(|child| {
            if child.get_disabled_state() {
                return None;
            }
            if child.ancestors().any(|a| a.type_id() == ElementNodeTypeId(HTMLDataListElementTypeId)) {
                return None;
            }
            // XXXManishearth don't include it if it is a button but not the submitter
            match child.type_id() {
                ElementNodeTypeId(HTMLInputElementTypeId) => {
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
                    match ty.as_slice() {
                        "image" => None, // Unimplemented
                        "radio" | "checkbox" => {
                            if value.is_empty() {
                                value = "on".to_string();
                            }
                            Some(FormDatum {
                                ty: ty,
                                name: name,
                                value: value
                            })
                        },
                        "file" => None, // Unimplemented
                        _ => Some(FormDatum {
                            ty: ty,
                            name: name,
                            value: input.Value()
                        })
                    }
                }
                ElementNodeTypeId(HTMLButtonElementTypeId) => {
                    // Unimplemented
                    None
                }
                ElementNodeTypeId(HTMLSelectElementTypeId) => {
                    // Unimplemented
                    None
                }
                ElementNodeTypeId(HTMLObjectElementTypeId) => {
                    // Unimplemented
                    None
                }
                ElementNodeTypeId(HTMLTextAreaElementTypeId) => {
                    // Unimplemented
                    None
                }
                _ => None
            }
        });
        // TODO: Handle `dirnames` (needs directionality support)
        //       https://html.spec.whatwg.org/multipage/dom.html#the-directionality
        let mut ret: Vec<FormDatum> = data_set.collect();
        for mut datum in ret.iter_mut() {
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
}

impl Reflectable for HTMLFormElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}

// TODO: add file support
pub struct FormDatum {
    pub ty: DOMString,
    pub name: DOMString,
    pub value: DOMString
}

pub enum FormEncType {
    TextPlainEncoded,
    UrlEncoded,
    FormDataEncoded
}

pub enum FormMethod {
    FormGet,
    FormPost,
    FormDialog
}

pub enum FormSubmitter<'a> {
    FormElement(JSRef<'a, HTMLFormElement>)
    // TODO: Submit buttons, image submit, etc etc
}

impl<'a> FormSubmitter<'a> {
    fn action(&self) -> DOMString {
        match *self {
            FormElement(form) => form.Action()
        }
    }

    fn enctype(&self) -> FormEncType {
        match *self {
            FormElement(form) => {
                match form.Enctype().as_slice() {
                    "multipart/form-data" => FormDataEncoded,
                    "text/plain" => TextPlainEncoded,
                    // https://html.spec.whatwg.org/multipage/forms.html#attr-fs-enctype
                    // urlencoded is the default
                    _ => UrlEncoded
                }
            }
        }
    }

    fn method(&self) -> FormMethod {
        match *self {
            FormElement(form) => {
                match form.Method().as_slice() {
                    "dialog" => FormDialog,
                    "post" => FormPost,
                    _ => FormGet
                }
            }
        }
    }

    fn target(&self) -> DOMString {
        match *self {
            FormElement(form) => form.Target()
        }
    }
}
