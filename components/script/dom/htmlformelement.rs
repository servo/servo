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
use dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::Reflectable;
use dom::document::Document;
use dom::element::Element;
use dom::event::{EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::htmlbuttonelement::HTMLButtonElement;
use dom::htmlcollection::CollectionFilter;
use dom::htmldatalistelement::HTMLDataListElement;
use dom::htmlelement::HTMLElement;
use dom::htmlfieldsetelement::HTMLFieldSetElement;
use dom::htmlformcontrolscollection::HTMLFormControlsCollection;
use dom::htmlinputelement::HTMLInputElement;
use dom::htmlobjectelement::HTMLObjectElement;
use dom::htmloutputelement::HTMLOutputElement;
use dom::htmlselectelement::HTMLSelectElement;
use dom::htmltextareaelement::HTMLTextAreaElement;
use dom::node::{Node, document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom::window::Window;
use hyper::header::ContentType;
use hyper::method::Method;
use hyper::mime;
use msg::constellation_msg::{LoadData, PipelineId};
use script_runtime::ScriptChan;
use script_thread::{MainThreadScriptChan, MainThreadScriptMsg, Runnable};
use std::borrow::ToOwned;
use std::cell::Cell;
use std::sync::mpsc::Sender;
use string_cache::Atom;
use task_source::dom_manipulation::DOMManipulationTask;
use url::form_urlencoded::serialize;
use util::str::DOMString;

#[derive(JSTraceable, PartialEq, Clone, Copy, HeapSizeOf)]
pub struct GenerationId(u32);

#[dom_struct]
pub struct HTMLFormElement {
    htmlelement: HTMLElement,
    marked_for_reset: Cell<bool>,
    elements: MutNullableHeap<JS<HTMLFormControlsCollection>>,
    generation_id: Cell<GenerationId>
}

impl HTMLFormElement {
    fn new_inherited(localName: Atom,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLFormElement {
        HTMLFormElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document),
            marked_for_reset: Cell::new(false),
            elements: Default::default(),
            generation_id: Cell::new(GenerationId(0))
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLFormElement> {
        let element = HTMLFormElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLFormElementBinding::Wrap)
    }

    pub fn generation_id(&self) -> GenerationId {
        self.generation_id.get()
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

    // https://html.spec.whatwg.org/multipage/#dom-form-elements
    fn Elements(&self) -> Root<HTMLFormControlsCollection> {
        if let Some(elements) = self.elements.get() {
            return elements;
        }

        #[derive(JSTraceable, HeapSizeOf)]
        struct ElementsFilter {
            form: Root<HTMLFormElement>
        }
        impl CollectionFilter for ElementsFilter {
            fn filter<'a>(&self, elem: &'a Element, _root: &'a Node) -> bool {
                let form_owner = match elem.upcast::<Node>().type_id() {
                    NodeTypeId::Element(ElementTypeId::HTMLElement(t)) => {
                        match t {
                            HTMLElementTypeId::HTMLButtonElement => {
                                elem.downcast::<HTMLButtonElement>().unwrap().form_owner()
                            }
                            HTMLElementTypeId::HTMLFieldSetElement => {
                                elem.downcast::<HTMLFieldSetElement>().unwrap().form_owner()
                            }
                            HTMLElementTypeId::HTMLInputElement => {
                                let input_elem = elem.downcast::<HTMLInputElement>().unwrap();
                                if input_elem.type_() == atom!("image") {
                                    return false;
                                }
                                input_elem.form_owner()
                            }
                            HTMLElementTypeId::HTMLObjectElement => {
                                elem.downcast::<HTMLObjectElement>().unwrap().form_owner()
                            }
                            HTMLElementTypeId::HTMLOutputElement => {
                                elem.downcast::<HTMLOutputElement>().unwrap().form_owner()
                            }
                            HTMLElementTypeId::HTMLSelectElement => {
                                elem.downcast::<HTMLSelectElement>().unwrap().form_owner()
                            }
                            HTMLElementTypeId::HTMLTextAreaElement => {
                                elem.downcast::<HTMLTextAreaElement>().unwrap().form_owner()
                            }
                            _ => {
                                debug_assert!(!elem.downcast::<HTMLElement>().unwrap().is_listed_element());
                                return false;
                            }
                        }
                    }
                    _ => return false,
                };

                match form_owner {
                    Some(form_owner) => form_owner == self.form,
                    None => false,
                }
            }
        }
        let filter = box ElementsFilter { form: Root::from_ref(self) };
        let window = window_from_node(self);
        let elements = HTMLFormControlsCollection::new(window.r(), self.upcast(), filter);
        self.elements.set(Some(&elements));
        elements
    }

    // https://html.spec.whatwg.org/multipage/#dom-form-length
    fn Length(&self) -> u32 {
        self.Elements().Length() as u32
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
        let base = doc.url();
        // TODO: Handle browsing contexts
        // Step 4
        if submit_method_flag == SubmittedFrom::NotFromFormSubmitMethod
           && !submitter.no_validate(self)
        {
            if self.interactive_validation().is_err() {
                // TODO: Implement event handlers on all form control elements
                self.upcast::<EventTarget>().fire_simple_event("invalid");
                return;
            }
        }
        // Step 5
        if submit_method_flag == SubmittedFrom::NotFromFormSubmitMethod {
            let event = self.upcast::<EventTarget>()
                .fire_event("submit",
                            EventBubbles::Bubbles,
                            EventCancelable::Cancelable);
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
        // Step 9-11
        let action_components = match base.join(&action) {
            Ok(url) => url,
            Err(_) => return
        };
        // Step 12-15
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
        let win = window_from_node(self);
        match (&*scheme, method) {
            // https://html.spec.whatwg.org/multipage/#submit-dialog
            (_, FormMethod::FormDialog) => return, // Unimplemented
            // https://html.spec.whatwg.org/multipage/#submit-mutate-action
            ("http", FormMethod::FormGet) | ("https", FormMethod::FormGet) => {
                load_data.url.query = Some(parsed_data);
                self.plan_to_navigate(load_data, &win);
            }
            // https://html.spec.whatwg.org/multipage/#submit-body
            ("http", FormMethod::FormPost) | ("https", FormMethod::FormPost) => {
                load_data.method = Method::Post;
                load_data.data = Some(parsed_data.into_bytes());
            }
            // https://html.spec.whatwg.org/multipage/#submit-get-action
            ("file", _) | ("about", _) | ("data", FormMethod::FormGet) |
            ("ftp", _) | ("javascript", _) => {
                self.plan_to_navigate(load_data, &win);
            }
            _ => return // Unimplemented (data and mailto)
        }
    }

    /// [Planned navigation](https://html.spec.whatwg.org/multipage/#planned-navigation)
    fn plan_to_navigate(&self, load_data: LoadData, window: &Window) {
        // Step 1
        // Each planned navigation runnable is tagged with a generation ID, and
        // before the runnable is handled, it first checks whether the HTMLFormElement's
        // generation ID is the same as its own generation ID.
        let GenerationId(prev_id) = self.generation_id.get();
        self.generation_id.set(GenerationId(prev_id + 1));

        // Step 2
        let chan = MainThreadScriptChan(window.main_thread_script_chan().clone()).clone();
        let nav = box PlannedNavigation {
            load_data: load_data,
            pipeline_id: window.pipeline(),
            script_chan: window.main_thread_script_chan().clone(),
            generation_id: self.generation_id.get(),
            form: Trusted::new(self, chan)
        };

        // Step 3
        window.dom_manipulation_task_source().queue(
            DOMManipulationTask::PlannedNavigation(nav)).unwrap();
    }

    /// Interactively validate the constraints of form elements
    /// https://html.spec.whatwg.org/multipage/#interactively-validate-the-constraints
    fn interactive_validation(&self) -> Result<(), ()> {
        // Step 1-3
        let _unhandled_invalid_controls = match self.static_validation() {
            Ok(()) => return Ok(()),
            Err(err) => err
        };
        // TODO: Report the problems with the constraints of at least one of
        //       the elements given in unhandled invalid controls to the user
        // Step 4
        Err(())
    }

    /// Statitically validate the constraints of form elements
    /// https://html.spec.whatwg.org/multipage/#statically-validate-the-constraints
    fn static_validation(&self) -> Result<(), Vec<FormSubmittableElement>> {
        let node = self.upcast::<Node>();
        // FIXME(#3553): This is an incorrect way of getting controls owned by the
        //               form, refactor this when html5ever's form owner PR lands
        // Step 1-3
        let invalid_controls = node.traverse_preorder().filter_map(|field| {
            if let Some(_el) = field.downcast::<Element>() {
                None // Remove this line if you decide to refactor

                // XXXKiChjang: Form control elements should each have a candidate_for_validation
                //              and satisfies_constraints methods

            } else {
                None
            }
        }).collect::<Vec<FormSubmittableElement>>();
        // Step 4
        if invalid_controls.is_empty() { return Ok(()); }
        // Step 5-6
        let unhandled_invalid_controls = invalid_controls.into_iter().filter_map(|field| {
            let event = field.as_event_target()
                .fire_event("invalid",
                            EventBubbles::DoesNotBubble,
                            EventCancelable::Cancelable);
            if !event.DefaultPrevented() { return Some(field); }
            None
        }).collect::<Vec<FormSubmittableElement>>();
        // Step 7
        Err(unhandled_invalid_controls)
    }

    /// https://html.spec.whatwg.org/multipage/#constructing-the-form-data-set
    /// Steps range from 1 to 3
    fn get_unclean_dataset(&self, submitter: Option<FormSubmitter>) -> Vec<FormDatum> {
        let node = self.upcast::<Node>();
        // FIXME(#3553): This is an incorrect way of getting controls owned
        //               by the form, but good enough until html5ever lands
        let mut data_set = Vec::new();
        for child in node.traverse_preorder() {
            // Step 3.1: The field element is disabled.
            match child.downcast::<Element>() {
                Some(el) if !el.disabled_state() => (),
                _ => continue,
            }

            // Step 3.1: The field element has a datalist element ancestor.
            if child.ancestors()
                    .any(|a| Root::downcast::<HTMLDataListElement>(a).is_some()) {
                continue;
            }
            if let NodeTypeId::Element(ElementTypeId::HTMLElement(element)) = child.type_id() {
                match element {
                    HTMLElementTypeId::HTMLInputElement => {
                        let input = child.downcast::<HTMLInputElement>().unwrap();
                        // Step 3.2-3.7
                        if let Some(datum) = input.get_form_datum(submitter) {
                            data_set.push(datum);
                        }
                    }
                    HTMLElementTypeId::HTMLButtonElement |
                    HTMLElementTypeId::HTMLObjectElement => {
                        // Unimplemented
                        ()
                    }
                    HTMLElementTypeId::HTMLSelectElement => {
                        let select = child.downcast::<HTMLSelectElement>().unwrap();
                        select.push_form_data(&mut data_set);
                    }
                    HTMLElementTypeId::HTMLTextAreaElement => {
                        let textarea = child.downcast::<HTMLTextAreaElement>().unwrap();
                        let name = textarea.Name();
                        if !name.is_empty() {
                            data_set.push(FormDatum {
                                ty: textarea.Type(),
                                name: name,
                                value: textarea.Value()
                            });
                        }
                    }
                    _ => ()
                }
            }
        }
        data_set
        // TODO: Handle `dirnames` (needs directionality support)
        //       https://html.spec.whatwg.org/multipage/#the-directionality
    }

    /// https://html.spec.whatwg.org/multipage/#constructing-the-form-data-set
    pub fn get_form_dataset(&self, submitter: Option<FormSubmitter>) -> Vec<FormDatum> {
        fn clean_crlf(s: &str) -> DOMString {
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

        // Step 1-3
        let mut ret = self.get_unclean_dataset(submitter);
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
        // Step 5
        ret
    }

    pub fn reset(&self, _reset_method_flag: ResetFrom) {
        // https://html.spec.whatwg.org/multipage/#locked-for-reset
        if self.marked_for_reset.get() {
            return;
        } else {
            self.marked_for_reset.set(true);
        }

        let event = self.upcast::<EventTarget>()
            .fire_event("reset",
                        EventBubbles::Bubbles,
                        EventCancelable::Cancelable);
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
            FormSubmittableElement::ButtonElement(ref button) => button.r().upcast(),
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

    fn no_validate(&self, _form_owner: &HTMLFormElement) -> bool {
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
            if let Some(ref o) = owner {
                let maybe_form = o.downcast::<HTMLFormElement>();
                if maybe_form.is_some() {
                    return maybe_form.map(Root::from_ref);
                }
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

    // XXXKiChjang: Implement these on inheritors
    // fn candidate_for_validation(&self) -> bool;
    // fn satisfies_constraints(&self) -> bool;
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

struct PlannedNavigation {
    load_data: LoadData,
    pipeline_id: PipelineId,
    script_chan: Sender<MainThreadScriptMsg>,
    generation_id: GenerationId,
    form: Trusted<HTMLFormElement>
}

impl Runnable for PlannedNavigation {
    fn handler(self: Box<PlannedNavigation>) {
        if self.generation_id == self.form.root().generation_id.get() {
            let script_chan = self.script_chan.clone();
            script_chan.send(MainThreadScriptMsg::Navigate(self.pipeline_id, self.load_data)).unwrap();
        }
    }
}
