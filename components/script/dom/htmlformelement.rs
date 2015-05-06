/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::AttrValue;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::HTMLFormElementBinding;
use dom::bindings::codegen::Bindings::HTMLFormElementBinding::HTMLFormElementMethods;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use dom::bindings::codegen::Bindings::HTMLButtonElementBinding::HTMLButtonElementMethods;
use dom::bindings::codegen::InheritTypes::ElementCast;
use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::codegen::InheritTypes::HTMLButtonElementCast;
use dom::bindings::codegen::InheritTypes::HTMLDataListElementCast;
use dom::bindings::codegen::InheritTypes::HTMLElementCast;
use dom::bindings::codegen::InheritTypes::HTMLFieldSetElementCast;
use dom::bindings::codegen::InheritTypes::HTMLFormElementCast;
use dom::bindings::codegen::InheritTypes::HTMLFormElementDerived;
use dom::bindings::codegen::InheritTypes::HTMLImageElementCast;
use dom::bindings::codegen::InheritTypes::HTMLInputElementCast;
use dom::bindings::codegen::InheritTypes::HTMLLabelElementCast;
use dom::bindings::codegen::InheritTypes::HTMLObjectElementCast;
use dom::bindings::codegen::InheritTypes::HTMLOutputElementCast;
use dom::bindings::codegen::InheritTypes::HTMLSelectElementCast;
use dom::bindings::codegen::InheritTypes::{HTMLTextAreaElementCast, NodeCast};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root, RootedReference};
use dom::bindings::trace::RootedVec;
use dom::document::{Document, DocumentHelpers};
use dom::element::{Element, ElementHelpers, AttributeHandlers};
use dom::event::{Event, EventHelpers, EventBubbles, EventCancelable};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::htmlinputelement::{HTMLInputElement, HTMLInputElementHelpers};
use dom::htmlbuttonelement::{HTMLButtonElement};
use dom::htmltextareaelement::HTMLTextAreaElementHelpers;
use dom::node::{Node, NodeHelpers, NodeTypeId, document_from_node, window_from_node};
use dom::node::{VecPreOrderInsertionHelper, PARSER_ASSOCIATED_FORM_OWNER};
use dom::virtualmethods::VirtualMethods;
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
    controls: DOMRefCell<Vec<JS<Element>>>,
}

impl PartialEq for HTMLFormElement {
    fn eq(&self, other: &HTMLFormElement) -> bool {
        self as *const HTMLFormElement == &*other
    }
}

impl HTMLFormElementDerived for EventTarget {
    fn is_htmlformelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFormElement)))
    }
}

impl HTMLFormElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLFormElement {
        HTMLFormElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLFormElement, localName, prefix, document),
            marked_for_reset: Cell::new(false),
            controls: DOMRefCell::new(Vec::new()),
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

impl<'a> HTMLFormElementMethods for &'a HTMLFormElement {
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
    fn Encoding(self) -> DOMString {
        self.Enctype()
    }

    // https://html.spec.whatwg.org/multipage/#dom-fs-encoding
    fn SetEncoding(self, value: DOMString) {
        self.SetEnctype(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-fs-method
    make_enumerated_getter!(Method, "get", ("post") | ("dialog"));

    // https://html.spec.whatwg.org/multipage/#dom-fs-method
    make_setter!(SetMethod, "method");

    // https://html.spec.whatwg.org/multipage/#dom-form-name
    make_getter!(Name);
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
    fn Submit(self) {
        self.submit(SubmittedFrom::FromFormSubmitMethod, FormSubmitter::FormElement(self));
    }

    // https://html.spec.whatwg.org/multipage/#dom-form-reset
    fn Reset(self) {
        self.reset(ResetFrom::FromFormResetMethod);
    }
}

#[derive(Copy, Clone)]
pub enum SubmittedFrom {
    FromFormSubmitMethod,
    NotFromFormSubmitMethod
}

#[derive(Copy, Clone)]
pub enum ResetFrom {
    FromFormResetMethod,
    NotFromFormResetMethod
}

pub trait HTMLFormElementHelpers {
    // https://html.spec.whatwg.org/multipage/#concept-form-submit
    fn submit(self, submit_method_flag: SubmittedFrom, submitter: FormSubmitter);
    // https://html.spec.whatwg.org/multipage/#constructing-the-form-data-set
    fn get_form_dataset(self, submitter: Option<FormSubmitter>) -> Vec<FormDatum>;
    // https://html.spec.whatwg.org/multipage/#dom-form-reset
    fn reset(self, submit_method_flag: ResetFrom);

    fn add_control<T: ?Sized + FormControl>(self, control: &T);
    fn remove_control<T: ?Sized + FormControl>(self, control: &T);
}

impl<'a> HTMLFormElementHelpers for &'a HTMLFormElement {
    fn submit(self, _submit_method_flag: SubmittedFrom, submitter: FormSubmitter) {
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
        let action_components = UrlParser::new().base_url(&base).parse(&action).unwrap_or(base);
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
        win.r().script_chan().send(ScriptMsg::Navigate(win.r().pipeline(), load_data)).unwrap();
    }

    fn get_form_dataset<'b>(self, submitter: Option<FormSubmitter<'b>>) -> Vec<FormDatum> {
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

        let controls = self.controls.borrow();
        let data_set = controls.iter().filter_map(|child| {
            let child = child.root();
            let child = NodeCast::from_ref(child.r());
            if child.get_disabled_state() {
                return None;
            }
            if child.ancestors()
                    .any(|a| HTMLDataListElementCast::to_root(a).is_some()) {
                return None;
            }
            // XXXManishearth don't include it if it is a button but not the submitter
            match child.type_id() {
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                    let input = HTMLInputElementCast::to_ref(child).unwrap();
                    let ty = input.Type();
                    let name = input.Name();
                    match &*ty {
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
                    match &*ty {
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
        //       https://html.spec.whatwg.org/multipage/#the-directionality
        let mut ret: Vec<FormDatum> = data_set.collect();
        for datum in ret.iter_mut() {
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

    fn reset(self, _reset_method_flag: ResetFrom) {
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

        let controls = self.controls.borrow();
        for child in controls.iter() {
            let child = child.root();
            let child = NodeCast::from_ref(child.r());
            match child.type_id() {
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                    let input = HTMLInputElementCast::to_ref(child).unwrap();
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
                    let textarea = HTMLTextAreaElementCast::to_ref(child).unwrap();
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

    fn add_control<T: ?Sized + FormControl>(self, control: &T) {
        let elem = ElementCast::from_ref(self);
        let root = elem.get_root_element();
        let root = NodeCast::from_ref(root.r());

        let mut controls = self.controls.borrow_mut();
        controls.insert_pre_order(control.to_element(), root);
    }

    fn remove_control<T: ?Sized + FormControl>(self, control: &T) {
        let control = control.to_element();
        let mut controls = self.controls.borrow_mut();
        controls.iter().map(|c| c.root())
                       .position(|c| c.r() == control)
                       .map(|idx| controls.remove(idx));
    }
}

// TODO: add file support
pub struct FormDatum {
    pub ty: DOMString,
    pub name: DOMString,
    pub value: DOMString
}

#[derive(Copy, Clone)]
pub enum FormEncType {
    TextPlainEncoded,
    UrlEncoded,
    FormDataEncoded
}

#[derive(Copy, Clone)]
pub enum FormMethod {
    FormGet,
    FormPost,
    FormDialog
}

#[derive(Copy, Clone)]
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

pub trait FormControl {

    fn form_owner(&self) -> Option<Root<HTMLFormElement>>;

    fn set_form_owner(&self, form: Option<&HTMLFormElement>);

    fn to_element<'a>(&'a self) -> &'a Element;

    fn is_reassociatable(&self) -> bool {
        true
    }

    // https://html.spec.whatwg.org/multipage/syntax.html#create-an-element-for-the-token
    // Part of step 4.
    // '..suppress the running of the reset the form owner algorithm
    // when the parser subsequently attempts to insert the element..'
    fn set_form_owner_from_parser(&self, form: &HTMLFormElement) {
        let elem = self.to_element();
        let node = NodeCast::from_ref(elem);
        node.set_flag(PARSER_ASSOCIATED_FORM_OWNER, true);
        form.add_control(self);
        self.set_form_owner(Some(form));
    }

    // https://html.spec.whatwg.org/multipage/#reset-the-form-owner
    fn reset_form_owner(&self) {
        let elem = self.to_element();
        let node = NodeCast::from_ref(elem);
        let old_owner = self.form_owner();
        let has_form_id = elem.has_attribute(&atom!(form));
        let nearest_form_ancestor = node.ancestors()
                                        .filter_map(HTMLFormElementCast::to_root)
                                        .next();

        if (!self.is_reassociatable() || !has_form_id) && old_owner.is_some() {
            if nearest_form_ancestor == old_owner {
                return;
            }
        }

        let new_owner = if self.is_reassociatable() && has_form_id && node.is_in_doc() {
            // Step 3
            let doc = document_from_node(node);
            let form_id = elem.get_string_attribute(&atom!(form));
            doc.GetElementById(form_id).and_then(HTMLFormElementCast::to_root)
        } else {
            // Step 4
            nearest_form_ancestor
        };

        if old_owner != new_owner {
            old_owner.r().map(|o| o.remove_control(self));
            new_owner.r().map(|o| o.add_control(self));
            self.set_form_owner(new_owner.r());
        }
    }

    // https://html.spec.whatwg.org/multipage/forms.html#form-owner
    fn after_set_form_attr(&self) {
        let elem = self.to_element();
        let form_id = elem.get_string_attribute(&atom!(form));
        let node = NodeCast::from_ref(elem);

        if self.is_reassociatable() && !form_id.is_empty() && node.is_in_doc() {
            let doc = document_from_node(node);
            doc.register_form_id_listener(form_id, self);
        }

        self.reset_form_owner();
    }

    fn before_remove_form_attr(&self) {
        let elem = self.to_element();
        let form_id = elem.get_string_attribute(&atom!(form));

        if self.is_reassociatable() && !form_id.is_empty() {
            let doc = document_from_node(NodeCast::from_ref(elem));
            doc.unregister_form_id_listener(form_id, self);
        }
    }

    fn after_remove_form_attr(&self) {
        self.reset_form_owner();
    }

    fn bind_form_control_to_tree(&self) {
        let elem = self.to_element();
        let node = NodeCast::from_ref(elem);

        // https://html.spec.whatwg.org/multipage/syntax.html#create-an-element-for-the-token
        // Part of step 4.
        // '..suppress the running of the reset the form owner algorithm
        // when the parser subsequently attempts to insert the element..'
        let must_skip_reset = node.get_flag(PARSER_ASSOCIATED_FORM_OWNER);
        node.set_flag(PARSER_ASSOCIATED_FORM_OWNER, false);

        if !must_skip_reset {
            self.after_set_form_attr();
        }
    }

    fn unbind_form_control_from_tree(&self) {
        let elem = self.to_element();
        let has_form_attr = elem.has_attribute(&atom!(form));
        let same_subtree = self.form_owner().map_or(true, |form| {
            elem.is_in_same_home_subtree(form.r())
        });

        self.before_remove_form_attr();

        // Since this control has been unregistered from the id->listener map
        // in the previous step, reset_form_owner will not be invoked on it
        // when the form owner element is unbound (i.e it is in the same
        // subtree) if it appears later in the tree order. Hence invoke
        // reset from here if this control has the form attribute set.
        if !same_subtree || (self.is_reassociatable() && has_form_attr) {
            self.reset_form_owner();
        }
    }

    fn get_form_attribute<InputFn, OwnerFn>(self,
                                            attr: &Atom,
                                            input: InputFn,
                                            owner: OwnerFn)
                                            -> DOMString
        where InputFn: Fn(Self) -> DOMString,
              OwnerFn: Fn(&HTMLFormElement) -> DOMString,
              Self: Sized
    {
        if self.to_element().has_attribute(attr) {
            input(self)
        } else {
            self.form_owner().map_or("".to_owned(), |t| owner(t.r()))
        }
    }
}

impl<'a> VirtualMethods for &'a HTMLFormElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        Some(HTMLElementCast::from_borrowed_ref(self) as &VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("name") => AttrValue::from_atomic(value),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }

    fn unbind_from_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.unbind_from_tree(tree_in_doc);
        }

        // Collect the controls to reset because reset_form_owner
        // will mutably borrow self.controls
        let mut to_reset: RootedVec<JS<Element>> = RootedVec::new();
        to_reset.extend(self.controls.borrow().iter()
                        .filter(|c| !c.root().is_in_same_home_subtree(*self))
                        .map(|c| c.clone()));

        for control in to_reset.iter() {
            let control = control.root();
            control.r().as_maybe_form_control()
                       .expect("Element must be a form control")
                       .reset_form_owner();
        }
    }
}

pub trait FormControlElementHelpers {
    fn as_maybe_form_control<'a>(&'a self) -> Option<&'a FormControl>;
}

impl<'a> FormControlElementHelpers for &'a Element {
    fn as_maybe_form_control<'b>(&'b self) -> Option<&'b FormControl> {
        let node = NodeCast::from_ref(*self);

        match node.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) => {
                let element = HTMLButtonElementCast::to_borrowed_ref(self).unwrap();
                Some(element as &FormControl)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFieldSetElement)) => {
                let element = HTMLFieldSetElementCast::to_borrowed_ref(self).unwrap();
                Some(element as &FormControl)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLImageElement)) => {
                let element = HTMLImageElementCast::to_borrowed_ref(self).unwrap();
                Some(element as &FormControl)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                let element = HTMLInputElementCast::to_borrowed_ref(self).unwrap();
                Some(element as &FormControl)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLabelElement)) => {
                let element = HTMLLabelElementCast::to_borrowed_ref(self).unwrap();
                Some(element as &FormControl)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement)) => {
                let element = HTMLObjectElementCast::to_borrowed_ref(self).unwrap();
                Some(element as &FormControl)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOutputElement)) => {
                let element = HTMLOutputElementCast::to_borrowed_ref(self).unwrap();
                Some(element as &FormControl)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
                let element = HTMLSelectElementCast::to_borrowed_ref(self).unwrap();
                Some(element as &FormControl)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
                let element = HTMLTextAreaElementCast::to_borrowed_ref(self).unwrap();
                Some(element as &FormControl)
            },
            _ => {
                None
            }
        }
    }
}
