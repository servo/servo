/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::AttrBinding::AttrBinding::AttrMethods;
use crate::dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::HTMLButtonElementBinding::HTMLButtonElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLFormControlsCollectionBinding::HTMLFormControlsCollectionMethods;
use crate::dom::bindings::codegen::Bindings::HTMLFormElementBinding;
use crate::dom::bindings::codegen::Bindings::HTMLFormElementBinding::HTMLFormElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding::HTMLTextAreaElementMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use crate::dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::{Dom, DomOnceCell, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::blob::Blob;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::file::File;
use crate::dom::formdata::FormData;
use crate::dom::formdataevent::FormDataEvent;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlbuttonelement::HTMLButtonElement;
use crate::dom::htmlcollection::CollectionFilter;
use crate::dom::htmldatalistelement::HTMLDataListElement;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlfieldsetelement::HTMLFieldSetElement;
use crate::dom::htmlformcontrolscollection::HTMLFormControlsCollection;
use crate::dom::htmlimageelement::HTMLImageElement;
use crate::dom::htmlinputelement::{HTMLInputElement, InputType};
use crate::dom::htmllabelelement::HTMLLabelElement;
use crate::dom::htmllegendelement::HTMLLegendElement;
use crate::dom::htmlobjectelement::HTMLObjectElement;
use crate::dom::htmloutputelement::HTMLOutputElement;
use crate::dom::htmlselectelement::HTMLSelectElement;
use crate::dom::htmltextareaelement::HTMLTextAreaElement;
use crate::dom::node::{document_from_node, window_from_node};
use crate::dom::node::{Node, NodeFlags, ShadowIncluding};
use crate::dom::node::{UnbindContext, VecPreOrderInsertionHelper};
use crate::dom::validitystate::ValidationFlags;
use crate::dom::virtualmethods::VirtualMethods;
use crate::dom::window::Window;
use crate::task_source::TaskSource;
use dom_struct::dom_struct;
use encoding_rs::{Encoding, UTF_8};
use headers::{ContentType, HeaderMapExt};
use html5ever::{LocalName, Prefix};
use hyper::Method;
use mime::{self, Mime};
use net_traits::http_percent_encode;
use net_traits::request::Referrer;
use script_traits::{HistoryEntryReplacement, LoadData, LoadOrigin};
use servo_rand::random;
use std::borrow::ToOwned;
use std::cell::Cell;
use style::attr::AttrValue;
use style::str::split_html_space_chars;

use crate::dom::bindings::codegen::UnionTypes::RadioNodeListOrElement;
use crate::dom::radionodelist::RadioNodeList;
use std::collections::HashMap;
use time::{now, Duration, Tm};

use crate::dom::bindings::codegen::Bindings::NodeBinding::{NodeConstants, NodeMethods};

#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
pub struct GenerationId(u32);

#[dom_struct]
pub struct HTMLFormElement {
    htmlelement: HTMLElement,
    marked_for_reset: Cell<bool>,
    /// https://html.spec.whatwg.org/multipage/#constructing-entry-list
    constructing_entry_list: Cell<bool>,
    elements: DomOnceCell<HTMLFormControlsCollection>,
    generation_id: Cell<GenerationId>,
    controls: DomRefCell<Vec<Dom<Element>>>,
    past_names_map: DomRefCell<HashMap<DOMString, (Dom<Element>, Tm)>>,
}

impl HTMLFormElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLFormElement {
        HTMLFormElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            marked_for_reset: Cell::new(false),
            constructing_entry_list: Cell::new(false),
            elements: Default::default(),
            generation_id: Cell::new(GenerationId(0)),
            controls: DomRefCell::new(Vec::new()),
            past_names_map: DomRefCell::new(HashMap::new()),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> DomRoot<HTMLFormElement> {
        Node::reflect_node(
            Box::new(HTMLFormElement::new_inherited(local_name, prefix, document)),
            document,
            HTMLFormElementBinding::Wrap,
        )
    }
}

impl HTMLFormElementMethods for HTMLFormElement {
    // https://html.spec.whatwg.org/multipage/#dom-form-acceptcharset
    make_getter!(AcceptCharset, "accept-charset");

    // https://html.spec.whatwg.org/multipage/#dom-form-acceptcharset
    make_setter!(SetAcceptCharset, "accept-charset");

    // https://html.spec.whatwg.org/multipage/#dom-fs-action
    make_form_action_getter!(Action, "action");

    // https://html.spec.whatwg.org/multipage/#dom-fs-action
    make_setter!(SetAction, "action");

    // https://html.spec.whatwg.org/multipage/#dom-form-autocomplete
    make_enumerated_getter!(Autocomplete, "autocomplete", "on", "off");

    // https://html.spec.whatwg.org/multipage/#dom-form-autocomplete
    make_setter!(SetAutocomplete, "autocomplete");

    // https://html.spec.whatwg.org/multipage/#dom-fs-enctype
    make_enumerated_getter!(
        Enctype,
        "enctype",
        "application/x-www-form-urlencoded",
        "text/plain" | "multipart/form-data"
    );

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
    make_enumerated_getter!(Method, "method", "get", "post" | "dialog");

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
        self.submit(SubmittedFrom::FromForm, FormSubmitter::FormElement(self));
    }

    // https://html.spec.whatwg.org/multipage/#dom-form-reset
    fn Reset(&self) {
        self.reset(ResetFrom::FromForm);
    }

    // https://html.spec.whatwg.org/multipage/#dom-form-elements
    fn Elements(&self) -> DomRoot<HTMLFormControlsCollection> {
        #[derive(JSTraceable, MallocSizeOf)]
        struct ElementsFilter {
            form: DomRoot<HTMLFormElement>,
        }
        impl CollectionFilter for ElementsFilter {
            fn filter<'a>(&self, elem: &'a Element, _root: &'a Node) -> bool {
                let form_owner = match elem.upcast::<Node>().type_id() {
                    NodeTypeId::Element(ElementTypeId::HTMLElement(t)) => match t {
                        HTMLElementTypeId::HTMLButtonElement => {
                            elem.downcast::<HTMLButtonElement>().unwrap().form_owner()
                        },
                        HTMLElementTypeId::HTMLFieldSetElement => {
                            elem.downcast::<HTMLFieldSetElement>().unwrap().form_owner()
                        },
                        HTMLElementTypeId::HTMLInputElement => {
                            let input_elem = elem.downcast::<HTMLInputElement>().unwrap();
                            if input_elem.input_type() == InputType::Image {
                                return false;
                            }
                            input_elem.form_owner()
                        },
                        HTMLElementTypeId::HTMLObjectElement => {
                            elem.downcast::<HTMLObjectElement>().unwrap().form_owner()
                        },
                        HTMLElementTypeId::HTMLOutputElement => {
                            elem.downcast::<HTMLOutputElement>().unwrap().form_owner()
                        },
                        HTMLElementTypeId::HTMLSelectElement => {
                            elem.downcast::<HTMLSelectElement>().unwrap().form_owner()
                        },
                        HTMLElementTypeId::HTMLTextAreaElement => {
                            elem.downcast::<HTMLTextAreaElement>().unwrap().form_owner()
                        },
                        _ => {
                            debug_assert!(
                                !elem.downcast::<HTMLElement>().unwrap().is_listed_element() ||
                                    elem.local_name() == &local_name!("keygen")
                            );
                            return false;
                        },
                    },
                    _ => return false,
                };

                match form_owner {
                    Some(form_owner) => form_owner == self.form,
                    None => false,
                }
            }
        }
        DomRoot::from_ref(self.elements.init_once(|| {
            let filter = Box::new(ElementsFilter {
                form: DomRoot::from_ref(self),
            });
            let window = window_from_node(self);
            HTMLFormControlsCollection::new(&window, self.upcast(), filter)
        }))
    }

    // https://html.spec.whatwg.org/multipage/#dom-form-length
    fn Length(&self) -> u32 {
        self.Elements().Length() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-form-item
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<Element>> {
        let elements = self.Elements();
        elements.IndexedGetter(index)
    }

    // https://html.spec.whatwg.org/multipage/#the-form-element%3Adetermine-the-value-of-a-named-property
    fn NamedGetter(&self, name: DOMString) -> Option<RadioNodeListOrElement> {
        let mut candidates: Vec<DomRoot<Node>> = Vec::new();

        let controls = self.controls.borrow();
        // Step 1
        for child in controls.iter() {
            if child
                .downcast::<HTMLElement>()
                .map_or(false, |c| c.is_listed_element())
            {
                if (child.has_attribute(&local_name!("id")) &&
                    child.get_string_attribute(&local_name!("id")) == name) ||
                    (child.has_attribute(&local_name!("name")) &&
                        child.get_string_attribute(&local_name!("name")) == name)
                {
                    candidates.push(DomRoot::from_ref(&*child.upcast::<Node>()));
                }
            }
        }
        // Step 2
        if candidates.len() == 0 {
            for child in controls.iter() {
                if child.is::<HTMLImageElement>() {
                    if (child.has_attribute(&local_name!("id")) &&
                        child.get_string_attribute(&local_name!("id")) == name) ||
                        (child.has_attribute(&local_name!("name")) &&
                            child.get_string_attribute(&local_name!("name")) == name)
                    {
                        candidates.push(DomRoot::from_ref(&*child.upcast::<Node>()));
                    }
                }
            }
        }

        let mut past_names_map = self.past_names_map.borrow_mut();

        // Step 3
        if candidates.len() == 0 {
            if past_names_map.contains_key(&name) {
                return Some(RadioNodeListOrElement::Element(DomRoot::from_ref(
                    &*past_names_map.get(&name).unwrap().0,
                )));
            }
            return None;
        }

        // Step 4
        if candidates.len() > 1 {
            let window = window_from_node(self);

            return Some(RadioNodeListOrElement::RadioNodeList(
                RadioNodeList::new_simple_list(&window, candidates.into_iter()),
            ));
        }

        // Step 5
        let element_node = &candidates[0];
        past_names_map.insert(
            name,
            (
                Dom::from_ref(&*element_node.downcast::<Element>().unwrap()),
                now(),
            ),
        );

        // Step 6
        return Some(RadioNodeListOrElement::Element(DomRoot::from_ref(
            &*element_node.downcast::<Element>().unwrap(),
        )));
    }

    // https://html.spec.whatwg.org/multipage/#the-form-element:supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        // Step 1
        #[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
        enum SourcedNameSource {
            Id,
            Name,
            Past(Duration),
        }

        impl SourcedNameSource {
            fn is_past(&self) -> bool {
                match self {
                    SourcedNameSource::Past(..) => true,
                    _ => false,
                }
            }
        }

        struct SourcedName {
            name: DOMString,
            element: DomRoot<Element>,
            source: SourcedNameSource,
        }

        let mut sourcedNamesVec: Vec<SourcedName> = Vec::new();

        let controls = self.controls.borrow();

        // Step 2
        for child in controls.iter() {
            if child
                .downcast::<HTMLElement>()
                .map_or(false, |c| c.is_listed_element())
            {
                if child.has_attribute(&local_name!("id")) {
                    let entry = SourcedName {
                        name: child.get_string_attribute(&local_name!("id")),
                        element: DomRoot::from_ref(&*child),
                        source: SourcedNameSource::Id,
                    };
                    sourcedNamesVec.push(entry);
                }
                if child.has_attribute(&local_name!("name")) {
                    let entry = SourcedName {
                        name: child.get_string_attribute(&local_name!("name")),
                        element: DomRoot::from_ref(&*child),
                        source: SourcedNameSource::Name,
                    };
                    sourcedNamesVec.push(entry);
                }
            }
        }

        // Step 3
        for child in controls.iter() {
            if child.is::<HTMLImageElement>() {
                if child.has_attribute(&local_name!("id")) {
                    let entry = SourcedName {
                        name: child.get_string_attribute(&local_name!("id")),
                        element: DomRoot::from_ref(&*child),
                        source: SourcedNameSource::Id,
                    };
                    sourcedNamesVec.push(entry);
                }
                if child.has_attribute(&local_name!("name")) {
                    let entry = SourcedName {
                        name: child.get_string_attribute(&local_name!("name")),
                        element: DomRoot::from_ref(&*child),
                        source: SourcedNameSource::Name,
                    };
                    sourcedNamesVec.push(entry);
                }
            }
        }

        // Step 4
        let past_names_map = self.past_names_map.borrow();
        for (key, val) in past_names_map.iter() {
            let entry = SourcedName {
                name: key.clone(),
                element: DomRoot::from_ref(&*val.0),
                source: SourcedNameSource::Past(now() - val.1), // calculate difference now()-val.1 to find age
            };
            sourcedNamesVec.push(entry);
        }

        // Step 5
        // TODO need to sort as per spec.
        // if a.CompareDocumentPosition(b) returns 0 that means a=b in which case
        // the remaining part where sorting is to be done by putting entries whose source is id first,
        // then entries whose source is name, and finally entries whose source is past,
        // and sorting entries with the same element and source by their age, oldest first.

        // if a.CompareDocumentPosition(b) has set NodeConstants::DOCUMENT_POSITION_FOLLOWING
        // (this can be checked by bitwise operations) then b would follow a in tree order and
        // Ordering::Less should be returned in the closure else Ordering::Greater

        sourcedNamesVec.sort_by(|a, b| {
            if a.element
                .upcast::<Node>()
                .CompareDocumentPosition(b.element.upcast::<Node>()) ==
                0
            {
                if a.source.is_past() && b.source.is_past() {
                    b.source.cmp(&a.source)
                } else {
                    a.source.cmp(&b.source)
                }
            } else {
                if a.element
                    .upcast::<Node>()
                    .CompareDocumentPosition(b.element.upcast::<Node>()) &
                    NodeConstants::DOCUMENT_POSITION_FOLLOWING ==
                    NodeConstants::DOCUMENT_POSITION_FOLLOWING
                {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            }
        });

        // Step 6
        sourcedNamesVec.retain(|sn| !sn.name.to_string().is_empty());

        // Step 7-8
        let mut namesVec: Vec<DOMString> = Vec::new();
        for elem in sourcedNamesVec.iter() {
            if namesVec
                .iter()
                .find(|name| name.to_string() == elem.name.to_string())
                .is_none()
            {
                namesVec.push(elem.name.clone());
            }
        }

        return namesVec;
    }
}

#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
pub enum SubmittedFrom {
    FromForm,
    NotFromForm,
}

#[derive(Clone, Copy, MallocSizeOf)]
pub enum ResetFrom {
    FromForm,
    NotFromForm,
}

impl HTMLFormElement {
    // https://html.spec.whatwg.org/multipage/#picking-an-encoding-for-the-form
    fn pick_encoding(&self) -> &'static Encoding {
        // Step 2
        if self
            .upcast::<Element>()
            .has_attribute(&local_name!("accept-charset"))
        {
            // Substep 1
            let input = self
                .upcast::<Element>()
                .get_string_attribute(&local_name!("accept-charset"));

            // Substep 2, 3, 4
            let mut candidate_encodings =
                split_html_space_chars(&*input).filter_map(|c| Encoding::for_label(c.as_bytes()));

            // Substep 5, 6
            return candidate_encodings.next().unwrap_or(UTF_8);
        }

        // Step 1, 3
        document_from_node(self).encoding()
    }

    // https://html.spec.whatwg.org/multipage/#text/plain-encoding-algorithm
    fn encode_plaintext(&self, form_data: &mut Vec<FormDatum>) -> String {
        // Step 1
        let mut result = String::new();

        // Step 2
        let encoding = self.pick_encoding();

        // Step 3
        let charset = encoding.name();

        for entry in form_data.iter_mut() {
            // Step 4, 5
            let value = entry.replace_value(charset);

            // Step 6
            result.push_str(&*format!("{}={}\r\n", entry.name, value));
        }

        // Step 7
        result
    }

    /// [Form submission](https://html.spec.whatwg.org/multipage/#concept-form-submit)
    pub fn submit(&self, submit_method_flag: SubmittedFrom, submitter: FormSubmitter) {
        // Step 1
        if self.upcast::<Element>().cannot_navigate() {
            return;
        }

        // Step 2
        if self.constructing_entry_list.get() {
            return;
        }
        // Step 3
        let doc = document_from_node(self);
        let base = doc.base_url();
        // TODO: Handle browsing contexts (Step 4, 5)
        // Step 6
        if submit_method_flag == SubmittedFrom::NotFromForm && !submitter.no_validate(self) {
            if self.interactive_validation().is_err() {
                // TODO: Implement event handlers on all form control elements
                self.upcast::<EventTarget>().fire_event(atom!("invalid"));
                return;
            }
        }
        // Step 7
        if submit_method_flag == SubmittedFrom::NotFromForm {
            let event = self
                .upcast::<EventTarget>()
                .fire_bubbling_cancelable_event(atom!("submit"));
            if event.DefaultPrevented() {
                return;
            }

            // Step 7-3
            if self.upcast::<Element>().cannot_navigate() {
                return;
            }
        }

        // Step 8
        let encoding = self.pick_encoding();

        // Step 9
        let mut form_data = match self.get_form_dataset(Some(submitter), Some(encoding)) {
            Some(form_data) => form_data,
            None => return,
        };

        // Step 10
        if self.upcast::<Element>().cannot_navigate() {
            return;
        }

        // Step 11
        let mut action = submitter.action();

        // Step 12
        if action.is_empty() {
            action = DOMString::from(base.as_str());
        }
        // Step 13-14
        let action_components = match base.join(&action) {
            Ok(url) => url,
            Err(_) => return,
        };
        // Step 15-17
        let scheme = action_components.scheme().to_owned();
        let enctype = submitter.enctype();
        let method = submitter.method();

        // Step 18-21
        let target_attribute_value = submitter.target();
        let source = doc.browsing_context().unwrap();
        let (maybe_chosen, _new) = source.choose_browsing_context(target_attribute_value, false);
        let chosen = match maybe_chosen {
            Some(proxy) => proxy,
            None => return,
        };
        let target_document = match chosen.document() {
            Some(doc) => doc,
            None => return,
        };
        let target_window = target_document.window();
        let mut load_data = LoadData::new(
            LoadOrigin::Script(doc.origin().immutable().clone()),
            action_components,
            None,
            Some(Referrer::ReferrerUrl(target_document.url())),
            target_document.get_referrer_policy(),
        );

        // Step 22
        match (&*scheme, method) {
            (_, FormMethod::FormDialog) => {
                // TODO: Submit dialog
                // https://html.spec.whatwg.org/multipage/#submit-dialog
            },
            // https://html.spec.whatwg.org/multipage/#submit-mutate-action
            ("http", FormMethod::FormGet) |
            ("https", FormMethod::FormGet) |
            ("data", FormMethod::FormGet) => {
                load_data
                    .headers
                    .typed_insert(ContentType::from(mime::APPLICATION_WWW_FORM_URLENCODED));
                self.mutate_action_url(&mut form_data, load_data, encoding, &target_window);
            },
            // https://html.spec.whatwg.org/multipage/#submit-body
            ("http", FormMethod::FormPost) | ("https", FormMethod::FormPost) => {
                load_data.method = Method::POST;
                self.submit_entity_body(
                    &mut form_data,
                    load_data,
                    enctype,
                    encoding,
                    &target_window,
                );
            },
            // https://html.spec.whatwg.org/multipage/#submit-get-action
            ("file", _) |
            ("about", _) |
            ("data", FormMethod::FormPost) |
            ("ftp", _) |
            ("javascript", _) => {
                self.plan_to_navigate(load_data, &target_window);
            },
            ("mailto", FormMethod::FormPost) => {
                // TODO: Mail as body
                // https://html.spec.whatwg.org/multipage/#submit-mailto-body
            },
            ("mailto", FormMethod::FormGet) => {
                // TODO: Mail with headers
                // https://html.spec.whatwg.org/multipage/#submit-mailto-headers
            },
            _ => return,
        }
    }

    // https://html.spec.whatwg.org/multipage/#submit-mutate-action
    fn mutate_action_url(
        &self,
        form_data: &mut Vec<FormDatum>,
        mut load_data: LoadData,
        encoding: &'static Encoding,
        target: &Window,
    ) {
        let charset = encoding.name();

        self.set_url_query_pairs(
            &mut load_data.url,
            form_data
                .iter()
                .map(|field| (&*field.name, field.replace_value(charset))),
        );

        self.plan_to_navigate(load_data, target);
    }

    // https://html.spec.whatwg.org/multipage/#submit-body
    fn submit_entity_body(
        &self,
        form_data: &mut Vec<FormDatum>,
        mut load_data: LoadData,
        enctype: FormEncType,
        encoding: &'static Encoding,
        target: &Window,
    ) {
        let boundary = generate_boundary();
        let bytes = match enctype {
            FormEncType::UrlEncoded => {
                let charset = encoding.name();
                load_data
                    .headers
                    .typed_insert(ContentType::from(mime::APPLICATION_WWW_FORM_URLENCODED));

                let mut url = load_data.url.clone();
                self.set_url_query_pairs(
                    &mut url,
                    form_data
                        .iter()
                        .map(|field| (&*field.name, field.replace_value(charset))),
                );

                url.query().unwrap_or("").to_string().into_bytes()
            },
            FormEncType::FormDataEncoded => {
                let mime: Mime = format!("multipart/form-data; boundary={}", boundary)
                    .parse()
                    .unwrap();
                load_data.headers.typed_insert(ContentType::from(mime));
                encode_multipart_form_data(form_data, boundary, encoding)
            },
            FormEncType::TextPlainEncoded => {
                load_data
                    .headers
                    .typed_insert(ContentType::from(mime::TEXT_PLAIN));
                self.encode_plaintext(form_data).into_bytes()
            },
        };

        load_data.data = Some(bytes);
        self.plan_to_navigate(load_data, target);
    }

    fn set_url_query_pairs<'a>(
        &self,
        url: &mut servo_url::ServoUrl,
        pairs: impl Iterator<Item = (&'a str, String)>,
    ) {
        let encoding = self.pick_encoding();
        url.as_mut_url()
            .query_pairs_mut()
            .encoding_override(Some(&|s| encoding.encode(s).0))
            .clear()
            .extend_pairs(pairs);
    }

    /// [Planned navigation](https://html.spec.whatwg.org/multipage/#planned-navigation)
    fn plan_to_navigate(&self, mut load_data: LoadData, target: &Window) {
        // Step 1
        // Each planned navigation task is tagged with a generation ID, and
        // before the task is handled, it first checks whether the HTMLFormElement's
        // generation ID is the same as its own generation ID.
        let generation_id = GenerationId(self.generation_id.get().0 + 1);
        self.generation_id.set(generation_id);

        // Step 2
        let elem = self.upcast::<Element>();
        let referrer = match elem.get_attribute(&ns!(), &local_name!("rel")) {
            Some(ref link_types) if link_types.Value().contains("noreferrer") => {
                Referrer::NoReferrer
            },
            _ => Referrer::Client,
        };

        let referrer_policy = target.Document().get_referrer_policy();
        let pipeline_id = target.upcast::<GlobalScope>().pipeline_id();
        load_data.creator_pipeline_id = Some(pipeline_id);
        load_data.referrer = Some(referrer);
        load_data.referrer_policy = referrer_policy;

        // Step 4.
        let this = Trusted::new(self);
        let window = Trusted::new(target);
        let task = task!(navigate_to_form_planned_navigation: move || {
            if generation_id != this.root().generation_id.get() {
                return;
            }
            window
                .root()
                .load_url(
                    HistoryEntryReplacement::Disabled,
                    false,
                    load_data,
                );
        });

        // Step 3.
        target
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task, target.upcast())
            .unwrap();
    }

    /// Interactively validate the constraints of form elements
    /// <https://html.spec.whatwg.org/multipage/#interactively-validate-the-constraints>
    fn interactive_validation(&self) -> Result<(), ()> {
        // Step 1-3
        let _unhandled_invalid_controls = match self.static_validation() {
            Ok(()) => return Ok(()),
            Err(err) => err,
        };
        // TODO: Report the problems with the constraints of at least one of
        //       the elements given in unhandled invalid controls to the user
        // Step 4
        Err(())
    }

    /// Statitically validate the constraints of form elements
    /// <https://html.spec.whatwg.org/multipage/#statically-validate-the-constraints>
    fn static_validation(&self) -> Result<(), Vec<FormSubmittableElement>> {
        let node = self.upcast::<Node>();
        // FIXME(#3553): This is an incorrect way of getting controls owned by the
        //               form, refactor this when html5ever's form owner PR lands
        // Step 1-3
        let invalid_controls = node
            .traverse_preorder(ShadowIncluding::No)
            .filter_map(|field| {
                if let Some(el) = field.downcast::<Element>() {
                    if el.disabled_state() {
                        None
                    } else {
                        let validatable = match el.as_maybe_validatable() {
                            Some(v) => v,
                            None => return None,
                        };
                        if !validatable.is_instance_validatable() {
                            None
                        } else if validatable.validate(ValidationFlags::empty()) {
                            None
                        } else {
                            Some(FormSubmittableElement::from_element(&el))
                        }
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<FormSubmittableElement>>();
        // Step 4
        if invalid_controls.is_empty() {
            return Ok(());
        }
        // Step 5-6
        let unhandled_invalid_controls = invalid_controls
            .into_iter()
            .filter_map(|field| {
                let event = field
                    .as_event_target()
                    .fire_cancelable_event(atom!("invalid"));
                if !event.DefaultPrevented() {
                    return Some(field);
                }
                None
            })
            .collect::<Vec<FormSubmittableElement>>();
        // Step 7
        Err(unhandled_invalid_controls)
    }

    /// <https://html.spec.whatwg.org/multipage/#constructing-the-form-data-set>
    /// terminology note:  "form data set" = "entry list"
    /// Steps range from 3 to 5
    /// 5.x substeps are mostly handled inside element-specific methods
    fn get_unclean_dataset(
        &self,
        submitter: Option<FormSubmitter>,
        encoding: Option<&'static Encoding>,
    ) -> Vec<FormDatum> {
        let controls = self.controls.borrow();
        let mut data_set = Vec::new();
        for child in controls.iter() {
            // Step 5.1: The field element is disabled.
            if child.disabled_state() {
                continue;
            }
            let child = child.upcast::<Node>();

            // Step 5.1: The field element has a datalist element ancestor.
            if child
                .ancestors()
                .any(|a| DomRoot::downcast::<HTMLDataListElement>(a).is_some())
            {
                continue;
            }
            if let NodeTypeId::Element(ElementTypeId::HTMLElement(element)) = child.type_id() {
                match element {
                    HTMLElementTypeId::HTMLInputElement => {
                        let input = child.downcast::<HTMLInputElement>().unwrap();

                        data_set.append(&mut input.form_datums(submitter, encoding));
                    },
                    HTMLElementTypeId::HTMLButtonElement => {
                        let button = child.downcast::<HTMLButtonElement>().unwrap();
                        if let Some(datum) = button.form_datum(submitter) {
                            data_set.push(datum);
                        }
                    },
                    HTMLElementTypeId::HTMLObjectElement => {
                        // Unimplemented
                        ()
                    },
                    HTMLElementTypeId::HTMLSelectElement => {
                        let select = child.downcast::<HTMLSelectElement>().unwrap();
                        select.push_form_data(&mut data_set);
                    },
                    HTMLElementTypeId::HTMLTextAreaElement => {
                        let textarea = child.downcast::<HTMLTextAreaElement>().unwrap();
                        let name = textarea.Name();
                        if !name.is_empty() {
                            data_set.push(FormDatum {
                                ty: textarea.Type(),
                                name: name,
                                value: FormDatumValue::String(textarea.Value()),
                            });
                        }
                    },
                    _ => (),
                }
            }
        }
        data_set
        // TODO: Handle `dirnames` (needs directionality support)
        //       https://html.spec.whatwg.org/multipage/#the-directionality
    }

    /// <https://html.spec.whatwg.org/multipage/#constructing-the-form-data-set>
    pub fn get_form_dataset(
        &self,
        submitter: Option<FormSubmitter>,
        encoding: Option<&'static Encoding>,
    ) -> Option<Vec<FormDatum>> {
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
                    _ => buf.push(ch),
                };
                prev = ch;
            }
            // In case the last character was CR
            if prev == '\r' {
                buf.push('\n');
            }
            DOMString::from(buf)
        }

        // Step 1
        if self.constructing_entry_list.get() {
            return None;
        }

        // Step 2
        self.constructing_entry_list.set(true);

        // Step 3-6
        let mut ret = self.get_unclean_dataset(submitter, encoding);
        for datum in &mut ret {
            match &*datum.ty {
                "file" | "textarea" => (), // TODO
                _ => {
                    datum.name = clean_crlf(&datum.name);
                    datum.value = FormDatumValue::String(clean_crlf(match datum.value {
                        FormDatumValue::String(ref s) => s,
                        FormDatumValue::File(_) => unreachable!(),
                    }));
                },
            }
        }

        let window = window_from_node(self);

        // Step 6
        let form_data = FormData::new(Some(ret), &window.global());

        // Step 7
        let event = FormDataEvent::new(
            &window.global(),
            atom!("formdata"),
            EventBubbles::Bubbles,
            EventCancelable::NotCancelable,
            &form_data,
        );

        event.upcast::<Event>().fire(self.upcast::<EventTarget>());

        // Step 8
        self.constructing_entry_list.set(false);

        // Step 9
        Some(form_data.datums())
    }

    pub fn reset(&self, _reset_method_flag: ResetFrom) {
        // https://html.spec.whatwg.org/multipage/#locked-for-reset
        if self.marked_for_reset.get() {
            return;
        } else {
            self.marked_for_reset.set(true);
        }

        let event = self
            .upcast::<EventTarget>()
            .fire_bubbling_cancelable_event(atom!("reset"));
        if event.DefaultPrevented() {
            return;
        }

        let controls = self.controls.borrow();
        for child in controls.iter() {
            let child = child.upcast::<Node>();

            match child.type_id() {
                NodeTypeId::Element(ElementTypeId::HTMLElement(
                    HTMLElementTypeId::HTMLInputElement,
                )) => {
                    child.downcast::<HTMLInputElement>().unwrap().reset();
                },
                // TODO HTMLKeygenElement unimplemented
                //NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLKeygenElement)) => {
                //    // Unimplemented
                //    {}
                //}
                NodeTypeId::Element(ElementTypeId::HTMLElement(
                    HTMLElementTypeId::HTMLSelectElement,
                )) => {
                    child.downcast::<HTMLSelectElement>().unwrap().reset();
                },
                NodeTypeId::Element(ElementTypeId::HTMLElement(
                    HTMLElementTypeId::HTMLTextAreaElement,
                )) => {
                    child.downcast::<HTMLTextAreaElement>().unwrap().reset();
                },
                NodeTypeId::Element(ElementTypeId::HTMLElement(
                    HTMLElementTypeId::HTMLOutputElement,
                )) => {
                    // Unimplemented
                },
                _ => {},
            }
        }
        self.marked_for_reset.set(false);
    }

    fn add_control<T: ?Sized + FormControl>(&self, control: &T) {
        let root = self.upcast::<Element>().root_element();
        let root = root.upcast::<Node>();

        let mut controls = self.controls.borrow_mut();
        controls.insert_pre_order(control.to_element(), root);
    }

    fn remove_control<T: ?Sized + FormControl>(&self, control: &T) {
        let control = control.to_element();
        let mut controls = self.controls.borrow_mut();
        controls
            .iter()
            .position(|c| &**c == control)
            .map(|idx| controls.remove(idx));
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
pub enum FormDatumValue {
    #[allow(dead_code)]
    File(DomRoot<File>),
    String(DOMString),
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
pub struct FormDatum {
    pub ty: DOMString,
    pub name: DOMString,
    pub value: FormDatumValue,
}

impl FormDatum {
    pub fn replace_value(&self, charset: &str) -> String {
        if self.name == "_charset_" && self.ty == "hidden" {
            return charset.to_string();
        }

        match self.value {
            FormDatumValue::File(ref f) => String::from(f.name().clone()),
            FormDatumValue::String(ref s) => String::from(s.clone()),
        }
    }
}

#[derive(Clone, Copy, MallocSizeOf)]
pub enum FormEncType {
    TextPlainEncoded,
    UrlEncoded,
    FormDataEncoded,
}

#[derive(Clone, Copy, MallocSizeOf)]
pub enum FormMethod {
    FormGet,
    FormPost,
    FormDialog,
}

#[derive(MallocSizeOf)]
#[allow(dead_code)]
pub enum FormSubmittableElement {
    ButtonElement(DomRoot<HTMLButtonElement>),
    InputElement(DomRoot<HTMLInputElement>),
    // TODO: HTMLKeygenElement unimplemented
    // KeygenElement(&'a HTMLKeygenElement),
    ObjectElement(DomRoot<HTMLObjectElement>),
    SelectElement(DomRoot<HTMLSelectElement>),
    TextAreaElement(DomRoot<HTMLTextAreaElement>),
}

impl FormSubmittableElement {
    fn as_event_target(&self) -> &EventTarget {
        match *self {
            FormSubmittableElement::ButtonElement(ref button) => button.upcast(),
            FormSubmittableElement::InputElement(ref input) => input.upcast(),
            FormSubmittableElement::ObjectElement(ref object) => object.upcast(),
            FormSubmittableElement::SelectElement(ref select) => select.upcast(),
            FormSubmittableElement::TextAreaElement(ref textarea) => textarea.upcast(),
        }
    }

    fn from_element(element: &Element) -> FormSubmittableElement {
        if let Some(input) = element.downcast::<HTMLInputElement>() {
            FormSubmittableElement::InputElement(DomRoot::from_ref(&input))
        } else if let Some(input) = element.downcast::<HTMLButtonElement>() {
            FormSubmittableElement::ButtonElement(DomRoot::from_ref(&input))
        } else if let Some(input) = element.downcast::<HTMLObjectElement>() {
            FormSubmittableElement::ObjectElement(DomRoot::from_ref(&input))
        } else if let Some(input) = element.downcast::<HTMLSelectElement>() {
            FormSubmittableElement::SelectElement(DomRoot::from_ref(&input))
        } else if let Some(input) = element.downcast::<HTMLTextAreaElement>() {
            FormSubmittableElement::TextAreaElement(DomRoot::from_ref(&input))
        } else {
            unreachable!()
        }
    }
}

#[derive(Clone, Copy, MallocSizeOf)]
pub enum FormSubmitter<'a> {
    FormElement(&'a HTMLFormElement),
    InputElement(&'a HTMLInputElement),
    ButtonElement(&'a HTMLButtonElement), // TODO: image submit, etc etc
}

impl<'a> FormSubmitter<'a> {
    fn action(&self) -> DOMString {
        match *self {
            FormSubmitter::FormElement(form) => form.Action(),
            FormSubmitter::InputElement(input_element) => input_element.get_form_attribute(
                &local_name!("formaction"),
                |i| i.FormAction(),
                |f| f.Action(),
            ),
            FormSubmitter::ButtonElement(button_element) => button_element.get_form_attribute(
                &local_name!("formaction"),
                |i| i.FormAction(),
                |f| f.Action(),
            ),
        }
    }

    fn enctype(&self) -> FormEncType {
        let attr = match *self {
            FormSubmitter::FormElement(form) => form.Enctype(),
            FormSubmitter::InputElement(input_element) => input_element.get_form_attribute(
                &local_name!("formenctype"),
                |i| i.FormEnctype(),
                |f| f.Enctype(),
            ),
            FormSubmitter::ButtonElement(button_element) => button_element.get_form_attribute(
                &local_name!("formenctype"),
                |i| i.FormEnctype(),
                |f| f.Enctype(),
            ),
        };
        match &*attr {
            "multipart/form-data" => FormEncType::FormDataEncoded,
            "text/plain" => FormEncType::TextPlainEncoded,
            // https://html.spec.whatwg.org/multipage/#attr-fs-enctype
            // urlencoded is the default
            _ => FormEncType::UrlEncoded,
        }
    }

    fn method(&self) -> FormMethod {
        let attr = match *self {
            FormSubmitter::FormElement(form) => form.Method(),
            FormSubmitter::InputElement(input_element) => input_element.get_form_attribute(
                &local_name!("formmethod"),
                |i| i.FormMethod(),
                |f| f.Method(),
            ),
            FormSubmitter::ButtonElement(button_element) => button_element.get_form_attribute(
                &local_name!("formmethod"),
                |i| i.FormMethod(),
                |f| f.Method(),
            ),
        };
        match &*attr {
            "dialog" => FormMethod::FormDialog,
            "post" => FormMethod::FormPost,
            _ => FormMethod::FormGet,
        }
    }

    fn target(&self) -> DOMString {
        match *self {
            FormSubmitter::FormElement(form) => form.Target(),
            FormSubmitter::InputElement(input_element) => input_element.get_form_attribute(
                &local_name!("formtarget"),
                |i| i.FormTarget(),
                |f| f.Target(),
            ),
            FormSubmitter::ButtonElement(button_element) => button_element.get_form_attribute(
                &local_name!("formtarget"),
                |i| i.FormTarget(),
                |f| f.Target(),
            ),
        }
    }

    fn no_validate(&self, _form_owner: &HTMLFormElement) -> bool {
        match *self {
            FormSubmitter::FormElement(form) => form.NoValidate(),
            FormSubmitter::InputElement(input_element) => input_element.get_form_boolean_attribute(
                &local_name!("formnovalidate"),
                |i| i.FormNoValidate(),
                |f| f.NoValidate(),
            ),
            FormSubmitter::ButtonElement(button_element) => button_element
                .get_form_boolean_attribute(
                    &local_name!("formnovalidate"),
                    |i| i.FormNoValidate(),
                    |f| f.NoValidate(),
                ),
        }
    }
}

pub trait FormControl: DomObject {
    fn form_owner(&self) -> Option<DomRoot<HTMLFormElement>>;

    fn set_form_owner(&self, form: Option<&HTMLFormElement>);

    fn to_element<'a>(&'a self) -> &'a Element;

    fn is_listed(&self) -> bool {
        true
    }

    // https://html.spec.whatwg.org/multipage/#create-an-element-for-the-token
    // Part of step 12.
    // '..suppress the running of the reset the form owner algorithm
    // when the parser subsequently attempts to insert the element..'
    fn set_form_owner_from_parser(&self, form: &HTMLFormElement) {
        let elem = self.to_element();
        let node = elem.upcast::<Node>();
        node.set_flag(NodeFlags::PARSER_ASSOCIATED_FORM_OWNER, true);
        form.add_control(self);
        self.set_form_owner(Some(form));
    }

    // https://html.spec.whatwg.org/multipage/#reset-the-form-owner
    fn reset_form_owner(&self) {
        let elem = self.to_element();
        let node = elem.upcast::<Node>();
        let old_owner = self.form_owner();
        let has_form_id = elem.has_attribute(&local_name!("form"));
        let nearest_form_ancestor = node
            .ancestors()
            .filter_map(DomRoot::downcast::<HTMLFormElement>)
            .next();

        // Step 1
        if old_owner.is_some() && !(self.is_listed() && has_form_id) {
            if nearest_form_ancestor == old_owner {
                return;
            }
        }

        let new_owner = if self.is_listed() && has_form_id && elem.is_connected() {
            // Step 3
            let doc = document_from_node(node);
            let form_id = elem.get_string_attribute(&local_name!("form"));
            doc.GetElementById(form_id)
                .and_then(DomRoot::downcast::<HTMLFormElement>)
        } else {
            // Step 4
            nearest_form_ancestor
        };

        if old_owner != new_owner {
            if let Some(o) = old_owner {
                o.remove_control(self);
            }
            if let Some(ref new_owner) = new_owner {
                new_owner.add_control(self);
            }
            self.set_form_owner(new_owner.as_deref());
        }
    }

    // https://html.spec.whatwg.org/multipage/#association-of-controls-and-forms
    fn form_attribute_mutated(&self, mutation: AttributeMutation) {
        match mutation {
            AttributeMutation::Set(_) => {
                self.register_if_necessary();
            },
            AttributeMutation::Removed => {
                self.unregister_if_necessary();
            },
        }

        self.reset_form_owner();
    }

    // https://html.spec.whatwg.org/multipage/#association-of-controls-and-forms
    fn register_if_necessary(&self) {
        let elem = self.to_element();
        let form_id = elem.get_string_attribute(&local_name!("form"));
        let node = elem.upcast::<Node>();

        if self.is_listed() && !form_id.is_empty() && node.is_connected() {
            let doc = document_from_node(node);
            doc.register_form_id_listener(form_id, self);
        }
    }

    fn unregister_if_necessary(&self) {
        let elem = self.to_element();
        let form_id = elem.get_string_attribute(&local_name!("form"));

        if self.is_listed() && !form_id.is_empty() {
            let doc = document_from_node(elem.upcast::<Node>());
            doc.unregister_form_id_listener(form_id, self);
        }
    }

    // https://html.spec.whatwg.org/multipage/#association-of-controls-and-forms
    fn bind_form_control_to_tree(&self) {
        let elem = self.to_element();
        let node = elem.upcast::<Node>();

        // https://html.spec.whatwg.org/multipage/#create-an-element-for-the-token
        // Part of step 12.
        // '..suppress the running of the reset the form owner algorithm
        // when the parser subsequently attempts to insert the element..'
        let must_skip_reset = node.get_flag(NodeFlags::PARSER_ASSOCIATED_FORM_OWNER);
        node.set_flag(NodeFlags::PARSER_ASSOCIATED_FORM_OWNER, false);

        if !must_skip_reset {
            self.form_attribute_mutated(AttributeMutation::Set(None));
        }
    }

    // https://html.spec.whatwg.org/multipage/#association-of-controls-and-forms
    fn unbind_form_control_from_tree(&self) {
        let elem = self.to_element();
        let has_form_attr = elem.has_attribute(&local_name!("form"));
        let same_subtree = self
            .form_owner()
            .map_or(true, |form| elem.is_in_same_home_subtree(&*form));

        self.unregister_if_necessary();

        // Since this control has been unregistered from the id->listener map
        // in the previous step, reset_form_owner will not be invoked on it
        // when the form owner element is unbound (i.e it is in the same
        // subtree) if it appears later in the tree order. Hence invoke
        // reset from here if this control has the form attribute set.
        if !same_subtree || (self.is_listed() && has_form_attr) {
            self.reset_form_owner();
        }
    }

    fn get_form_attribute<InputFn, OwnerFn>(
        &self,
        attr: &LocalName,
        input: InputFn,
        owner: OwnerFn,
    ) -> DOMString
    where
        InputFn: Fn(&Self) -> DOMString,
        OwnerFn: Fn(&HTMLFormElement) -> DOMString,
        Self: Sized,
    {
        if self.to_element().has_attribute(attr) {
            input(self)
        } else {
            self.form_owner().map_or(DOMString::new(), |t| owner(&t))
        }
    }

    fn get_form_boolean_attribute<InputFn, OwnerFn>(
        &self,
        attr: &LocalName,
        input: InputFn,
        owner: OwnerFn,
    ) -> bool
    where
        InputFn: Fn(&Self) -> bool,
        OwnerFn: Fn(&HTMLFormElement) -> bool,
        Self: Sized,
    {
        if self.to_element().has_attribute(attr) {
            input(self)
        } else {
            self.form_owner().map_or(false, |t| owner(&t))
        }
    }

    // XXXKiChjang: Implement these on inheritors
    // fn candidate_for_validation(&self) -> bool;
    // fn satisfies_constraints(&self) -> bool;
}

impl VirtualMethods for HTMLFormElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("name") => AttrValue::from_atomic(value.into()),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);

        // Collect the controls to reset because reset_form_owner
        // will mutably borrow self.controls
        rooted_vec!(let mut to_reset);
        to_reset.extend(
            self.controls
                .borrow()
                .iter()
                .filter(|c| !c.is_in_same_home_subtree(self))
                .map(|c| c.clone()),
        );

        for control in to_reset.iter() {
            control
                .as_maybe_form_control()
                .expect("Element must be a form control")
                .reset_form_owner();
        }
    }
}

pub trait FormControlElementHelpers {
    fn as_maybe_form_control<'a>(&'a self) -> Option<&'a dyn FormControl>;
}

impl FormControlElementHelpers for Element {
    fn as_maybe_form_control<'a>(&'a self) -> Option<&'a dyn FormControl> {
        let node = self.upcast::<Node>();

        match node.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLButtonElement,
            )) => Some(self.downcast::<HTMLButtonElement>().unwrap() as &dyn FormControl),
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLFieldSetElement,
            )) => Some(self.downcast::<HTMLFieldSetElement>().unwrap() as &dyn FormControl),
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLImageElement,
            )) => Some(self.downcast::<HTMLImageElement>().unwrap() as &dyn FormControl),
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLInputElement,
            )) => Some(self.downcast::<HTMLInputElement>().unwrap() as &dyn FormControl),
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLLabelElement,
            )) => Some(self.downcast::<HTMLLabelElement>().unwrap() as &dyn FormControl),
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLLegendElement,
            )) => Some(self.downcast::<HTMLLegendElement>().unwrap() as &dyn FormControl),
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLObjectElement,
            )) => Some(self.downcast::<HTMLObjectElement>().unwrap() as &dyn FormControl),
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLOutputElement,
            )) => Some(self.downcast::<HTMLOutputElement>().unwrap() as &dyn FormControl),
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLSelectElement,
            )) => Some(self.downcast::<HTMLSelectElement>().unwrap() as &dyn FormControl),
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLTextAreaElement,
            )) => Some(self.downcast::<HTMLTextAreaElement>().unwrap() as &dyn FormControl),
            _ => None,
        }
    }
}

// https://html.spec.whatwg.org/multipage/#multipart/form-data-encoding-algorithm
pub fn encode_multipart_form_data(
    form_data: &mut Vec<FormDatum>,
    boundary: String,
    encoding: &'static Encoding,
) -> Vec<u8> {
    // Step 1
    let mut result = vec![];

    // Step 2
    let charset = encoding.name();

    // Step 3
    for entry in form_data.iter_mut() {
        // 3.1
        if entry.name == "_charset_" && entry.ty == "hidden" {
            entry.value = FormDatumValue::String(DOMString::from(charset.clone()));
        }
        // TODO: 3.2

        // Step 4
        // https://tools.ietf.org/html/rfc7578#section-4
        // NOTE(izgzhen): The encoding here expected by most servers seems different from
        // what spec says (that it should start with a '\r\n').
        let mut boundary_bytes = format!("--{}\r\n", boundary).into_bytes();
        result.append(&mut boundary_bytes);

        // TODO(eijebong): Everthing related to content-disposition it to redo once typed headers
        // are capable of it.
        match entry.value {
            FormDatumValue::String(ref s) => {
                let content_disposition = format!("form-data; name=\"{}\"", entry.name);
                let mut bytes =
                    format!("Content-Disposition: {}\r\n\r\n{}", content_disposition, s)
                        .into_bytes();
                result.append(&mut bytes);
            },
            FormDatumValue::File(ref f) => {
                let extra = if charset.to_lowercase() == "utf-8" {
                    format!(
                        "filename=\"{}\"",
                        String::from_utf8(f.name().as_bytes().into()).unwrap()
                    )
                } else {
                    format!(
                        "filename*=\"{}\"''{}",
                        charset,
                        http_percent_encode(f.name().as_bytes())
                    )
                };

                let content_disposition = format!("form-data; name=\"{}\"; {}", entry.name, extra);
                // https://tools.ietf.org/html/rfc7578#section-4.4
                let content_type: Mime = f
                    .upcast::<Blob>()
                    .Type()
                    .parse()
                    .unwrap_or(mime::TEXT_PLAIN);
                let mut type_bytes = format!(
                    "Content-Disposition: {}\r\ncontent-type: {}\r\n\r\n",
                    content_disposition, content_type
                )
                .into_bytes();
                result.append(&mut type_bytes);

                let mut bytes = f.upcast::<Blob>().get_bytes().unwrap_or(vec![]);

                result.append(&mut bytes);
            },
        }
    }

    let mut boundary_bytes = format!("\r\n--{}--\r\n", boundary).into_bytes();
    result.append(&mut boundary_bytes);

    result
}

// https://tools.ietf.org/html/rfc7578#section-4.1
pub fn generate_boundary() -> String {
    let i1 = random::<u32>();
    let i2 = random::<u32>();

    format!("---------------------------{0}{1}", i1, i2)
}
