/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::cell::Cell;

use constellation_traits::{LoadData, LoadOrigin, NavigationHistoryBehavior};
use content_security_policy::sandboxing_directive::SandboxingFlagSet;
use dom_struct::dom_struct;
use encoding_rs::{Encoding, UTF_8};
use headers::{ContentType, HeaderMapExt};
use html5ever::{LocalName, Prefix, local_name, ns};
use http::Method;
use js::rust::HandleObject;
use mime::{self, Mime};
use net_traits::http_percent_encode;
use net_traits::request::Referrer;
use rand::random;
use rustc_hash::FxBuildHasher;
use script_bindings::match_domstring_ascii;
use style::attr::AttrValue;
use style::str::split_html_space_chars;
use stylo_atoms::Atom;
use stylo_dom::ElementState;

use crate::body::Extractable;
use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::AttrBinding::Attr_Binding::AttrMethods;
use crate::dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::HTMLButtonElementBinding::HTMLButtonElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLFormControlsCollectionBinding::HTMLFormControlsCollectionMethods;
use crate::dom::bindings::codegen::Bindings::HTMLFormElementBinding::HTMLFormElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLOrSVGElementBinding::FocusOptions;
use crate::dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding::HTMLTextAreaElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::{NodeConstants, NodeMethods};
use crate::dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use crate::dom::bindings::codegen::Bindings::RadioNodeListBinding::RadioNodeListMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::Window_Binding::WindowMethods;
use crate::dom::bindings::codegen::UnionTypes::RadioNodeListOrElement;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, DomObject};
use crate::dom::bindings::root::{Dom, DomOnceCell, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::{HashMapTracedValues, NoTrace};
use crate::dom::blob::Blob;
use crate::dom::customelementregistry::CallbackReaction;
use crate::dom::document::Document;
use crate::dom::domtokenlist::DOMTokenList;
use crate::dom::element::{AttributeMutation, AttributeMutationReason, Element};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::file::File;
use crate::dom::formdata::FormData;
use crate::dom::formdataevent::FormDataEvent;
use crate::dom::html::htmlbuttonelement::HTMLButtonElement;
use crate::dom::html::htmlcollection::CollectionFilter;
use crate::dom::html::htmldatalistelement::HTMLDataListElement;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlfieldsetelement::HTMLFieldSetElement;
use crate::dom::html::htmlformcontrolscollection::HTMLFormControlsCollection;
use crate::dom::html::htmlimageelement::HTMLImageElement;
use crate::dom::html::htmlinputelement::{HTMLInputElement, InputType};
use crate::dom::html::htmllabelelement::HTMLLabelElement;
use crate::dom::html::htmllegendelement::HTMLLegendElement;
use crate::dom::html::htmlobjectelement::HTMLObjectElement;
use crate::dom::html::htmloutputelement::HTMLOutputElement;
use crate::dom::html::htmlselectelement::HTMLSelectElement;
use crate::dom::html::htmltextareaelement::HTMLTextAreaElement;
use crate::dom::node::{
    BindContext, Node, NodeFlags, NodeTraits, UnbindContext, VecPreOrderInsertionHelper,
};
use crate::dom::nodelist::{NodeList, RadioListMode};
use crate::dom::radionodelist::RadioNodeList;
use crate::dom::submitevent::SubmitEvent;
use crate::dom::types::HTMLIFrameElement;
use crate::dom::virtualmethods::VirtualMethods;
use crate::dom::window::Window;
use crate::links::{LinkRelations, get_element_target, valid_navigable_target_name_or_keyword};
use crate::script_runtime::CanGc;
use crate::script_thread::ScriptThread;

#[dom_struct]
pub(crate) struct HTMLFormElement {
    htmlelement: HTMLElement,
    marked_for_reset: Cell<bool>,
    /// <https://html.spec.whatwg.org/multipage/#constructing-entry-list>
    constructing_entry_list: Cell<bool>,
    elements: DomOnceCell<HTMLFormControlsCollection>,
    controls: DomRefCell<Vec<Dom<Element>>>,

    /// It is safe to use FxBuildHasher here as `Atom` is in the string_cache.
    #[expect(clippy::type_complexity)]
    past_names_map:
        DomRefCell<HashMapTracedValues<Atom, (Dom<Element>, NoTrace<usize>), FxBuildHasher>>,

    /// The current generation of past names, i.e., the number of name changes to the name.
    current_name_generation: Cell<usize>,

    firing_submission_events: Cell<bool>,
    rel_list: MutNullableDom<DOMTokenList>,

    /// <https://html.spec.whatwg.org/multipage/#planned-navigation>
    planned_navigation: Cell<usize>,

    #[no_trace]
    relations: Cell<LinkRelations>,
}

impl HTMLFormElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLFormElement {
        HTMLFormElement {
            htmlelement: HTMLElement::new_inherited_with_state(
                ElementState::VALID,
                local_name,
                prefix,
                document,
            ),
            marked_for_reset: Cell::new(false),
            constructing_entry_list: Cell::new(false),
            elements: Default::default(),
            controls: DomRefCell::new(Vec::new()),
            past_names_map: DomRefCell::new(HashMapTracedValues::new_fx()),
            current_name_generation: Cell::new(0),
            firing_submission_events: Cell::new(false),
            rel_list: Default::default(),
            planned_navigation: Default::default(),
            relations: Cell::new(LinkRelations::empty()),
        }
    }

    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLFormElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLFormElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
            can_gc,
        )
    }

    fn filter_for_radio_list(mode: RadioListMode, child: &Element, name: &Atom) -> bool {
        if let Some(child) = child.downcast::<Element>() {
            match mode {
                RadioListMode::ControlsExceptImageInputs => {
                    if child
                        .downcast::<HTMLElement>()
                        .is_some_and(|c| c.is_listed_element()) &&
                        (child.get_id().is_some_and(|i| i == *name) ||
                            child.get_name().is_some_and(|n| n == *name))
                    {
                        if let Some(inp) = child.downcast::<HTMLInputElement>() {
                            // input, only return it if it's not image-button state
                            return inp.input_type() != InputType::Image;
                        } else {
                            // control, but not an input
                            return true;
                        }
                    }
                    return false;
                },
                RadioListMode::Images => {
                    return child.is::<HTMLImageElement>() &&
                        (child.get_id().is_some_and(|i| i == *name) ||
                            child.get_name().is_some_and(|n| n == *name));
                },
            }
        }
        false
    }

    pub(crate) fn nth_for_radio_list(
        &self,
        index: u32,
        mode: RadioListMode,
        name: &Atom,
    ) -> Option<DomRoot<Node>> {
        self.controls
            .borrow()
            .iter()
            .filter(|n| HTMLFormElement::filter_for_radio_list(mode, n, name))
            .nth(index as usize)
            .map(|n| DomRoot::from_ref(n.upcast::<Node>()))
    }

    pub(crate) fn count_for_radio_list(&self, mode: RadioListMode, name: &Atom) -> u32 {
        self.controls
            .borrow()
            .iter()
            .filter(|n| HTMLFormElement::filter_for_radio_list(mode, n, name))
            .count() as u32
    }
}

impl HTMLFormElementMethods<crate::DomTypeHolder> for HTMLFormElement {
    // https://html.spec.whatwg.org/multipage/#dom-form-acceptcharset
    make_getter!(AcceptCharset, "accept-charset");

    // https://html.spec.whatwg.org/multipage/#dom-form-acceptcharset
    make_setter!(SetAcceptCharset, "accept-charset");

    // https://html.spec.whatwg.org/multipage/#dom-fs-action
    make_form_action_getter!(Action, "action");

    // https://html.spec.whatwg.org/multipage/#dom-fs-action
    make_setter!(SetAction, "action");

    // https://html.spec.whatwg.org/multipage/#dom-form-autocomplete
    make_enumerated_getter!(
        Autocomplete,
        "autocomplete",
        "on" | "off",
        missing => "on",
        invalid => "on"
    );

    // https://html.spec.whatwg.org/multipage/#dom-form-autocomplete
    make_setter!(SetAutocomplete, "autocomplete");

    // https://html.spec.whatwg.org/multipage/#dom-fs-enctype
    make_enumerated_getter!(
        Enctype,
        "enctype",
        "application/x-www-form-urlencoded" | "text/plain" | "multipart/form-data",
        missing => "application/x-www-form-urlencoded",
        invalid => "application/x-www-form-urlencoded"
    );

    // https://html.spec.whatwg.org/multipage/#dom-fs-enctype
    make_setter!(SetEnctype, "enctype");

    /// <https://html.spec.whatwg.org/multipage/#dom-fs-encoding>
    fn Encoding(&self) -> DOMString {
        self.Enctype()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-fs-encoding>
    fn SetEncoding(&self, value: DOMString) {
        self.SetEnctype(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-fs-method
    make_enumerated_getter!(
        Method,
        "method",
        "get" | "post" | "dialog",
        missing => "get",
        invalid => "get"
    );

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

    // https://html.spec.whatwg.org/multipage/#dom-a-rel
    make_getter!(Rel, "rel");

    /// <https://html.spec.whatwg.org/multipage/#the-form-element:concept-form-submit>
    fn Submit(&self, can_gc: CanGc) {
        self.submit(
            SubmittedFrom::FromForm,
            FormSubmitterElement::Form(self),
            can_gc,
        );
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-form-requestsubmit>
    fn RequestSubmit(&self, submitter: Option<&HTMLElement>, can_gc: CanGc) -> Fallible<()> {
        let submitter: FormSubmitterElement = match submitter {
            Some(submitter_element) => {
                // Step 1.1
                let error_not_a_submit_button =
                    Err(Error::Type("submitter must be a submit button".to_string()));

                let element = match submitter_element.upcast::<Node>().type_id() {
                    NodeTypeId::Element(ElementTypeId::HTMLElement(element)) => element,
                    _ => {
                        return error_not_a_submit_button;
                    },
                };

                let submit_button = match element {
                    HTMLElementTypeId::HTMLInputElement => FormSubmitterElement::Input(
                        submitter_element
                            .downcast::<HTMLInputElement>()
                            .expect("Failed to downcast submitter elem to HTMLInputElement."),
                    ),
                    HTMLElementTypeId::HTMLButtonElement => FormSubmitterElement::Button(
                        submitter_element
                            .downcast::<HTMLButtonElement>()
                            .expect("Failed to downcast submitter elem to HTMLButtonElement."),
                    ),
                    _ => {
                        return error_not_a_submit_button;
                    },
                };

                if !submit_button.is_submit_button() {
                    return error_not_a_submit_button;
                }

                let submitters_owner = submit_button.form_owner();

                // Step 1.2
                let owner = match submitters_owner {
                    Some(owner) => owner,
                    None => {
                        return Err(Error::NotFound(None));
                    },
                };

                if *owner != *self {
                    return Err(Error::NotFound(None));
                }

                submit_button
            },
            None => {
                // Step 2
                FormSubmitterElement::Form(self)
            },
        };
        // Step 3
        self.submit(SubmittedFrom::NotFromForm, submitter, can_gc);
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-form-reset>
    fn Reset(&self, can_gc: CanGc) {
        self.reset(ResetFrom::FromForm, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-form-elements>
    fn Elements(&self, can_gc: CanGc) -> DomRoot<HTMLFormControlsCollection> {
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
                        HTMLElementTypeId::HTMLElement => {
                            let html_element = elem.downcast::<HTMLElement>().unwrap();
                            if html_element.is_form_associated_custom_element() {
                                html_element.form_owner()
                            } else {
                                return false;
                            }
                        },
                        _ => {
                            debug_assert!(
                                !elem.downcast::<HTMLElement>().unwrap().is_listed_element()
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
            let window = self.owner_window();
            HTMLFormControlsCollection::new(&window, self, filter, can_gc)
        }))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-form-length>
    fn Length(&self) -> u32 {
        self.Elements(CanGc::note()).Length()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-form-item>
    fn IndexedGetter(&self, index: u32, can_gc: CanGc) -> Option<DomRoot<Element>> {
        let elements = self.Elements(can_gc);
        elements.IndexedGetter(index)
    }

    /// <https://html.spec.whatwg.org/multipage/#the-form-element%3Adetermine-the-value-of-a-named-property>
    fn NamedGetter(&self, name: DOMString, can_gc: CanGc) -> Option<RadioNodeListOrElement> {
        let window = self.owner_window();

        let name = Atom::from(name);

        // Step 1
        let mut candidates =
            RadioNodeList::new_controls_except_image_inputs(&window, self, &name, can_gc);
        let mut candidates_length = candidates.Length();

        // Step 2
        if candidates_length == 0 {
            candidates = RadioNodeList::new_images(&window, self, &name, can_gc);
            candidates_length = candidates.Length();
        }

        let mut past_names_map = self.past_names_map.borrow_mut();

        // Step 3
        if candidates_length == 0 {
            if past_names_map.contains_key(&name) {
                return Some(RadioNodeListOrElement::Element(DomRoot::from_ref(
                    &*past_names_map.get(&name).unwrap().0,
                )));
            }
            return None;
        }

        // Step 4
        if candidates_length > 1 {
            return Some(RadioNodeListOrElement::RadioNodeList(candidates));
        }

        // Step 5
        // candidates_length is 1, so we can unwrap item 0
        let element_node = candidates.upcast::<NodeList>().Item(0).unwrap();
        past_names_map.insert(
            name,
            (
                Dom::from_ref(element_node.downcast::<Element>().unwrap()),
                NoTrace(self.current_name_generation.get() + 1),
            ),
        );
        self.current_name_generation
            .set(self.current_name_generation.get() + 1);

        // Step 6
        Some(RadioNodeListOrElement::Element(DomRoot::from_ref(
            element_node.downcast::<Element>().unwrap(),
        )))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-a-rel>
    fn SetRel(&self, rel: DOMString, can_gc: CanGc) {
        self.upcast::<Element>()
            .set_tokenlist_attribute(&local_name!("rel"), rel, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-a-rellist>
    fn RelList(&self, can_gc: CanGc) -> DomRoot<DOMTokenList> {
        self.rel_list.or_init(|| {
            DOMTokenList::new(
                self.upcast(),
                &local_name!("rel"),
                Some(vec![
                    Atom::from("noopener"),
                    Atom::from("noreferrer"),
                    Atom::from("opener"),
                ]),
                can_gc,
            )
        })
    }

    // https://html.spec.whatwg.org/multipage/#the-form-element:supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        // Step 1
        #[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
        enum SourcedNameSource {
            Id,
            Name,
            Past(usize),
        }

        impl SourcedNameSource {
            fn is_past(&self) -> bool {
                matches!(self, SourcedNameSource::Past(..))
            }
        }

        struct SourcedName {
            name: Atom,
            element: DomRoot<Element>,
            source: SourcedNameSource,
        }

        let mut sourced_names_vec: Vec<SourcedName> = Vec::new();

        // Step 2
        for child in self.controls.borrow().iter() {
            if child
                .downcast::<HTMLElement>()
                .is_some_and(|c| c.is_listed_element())
            {
                if let Some(id_atom) = child.get_id() {
                    let entry = SourcedName {
                        name: id_atom,
                        element: DomRoot::from_ref(child),
                        source: SourcedNameSource::Id,
                    };
                    sourced_names_vec.push(entry);
                }
                if let Some(name_atom) = child.get_name() {
                    let entry = SourcedName {
                        name: name_atom,
                        element: DomRoot::from_ref(child),
                        source: SourcedNameSource::Name,
                    };
                    sourced_names_vec.push(entry);
                }
            }
        }

        // Step 3
        for child in self.controls.borrow().iter() {
            if child.is::<HTMLImageElement>() {
                if let Some(id_atom) = child.get_id() {
                    let entry = SourcedName {
                        name: id_atom,
                        element: DomRoot::from_ref(child),
                        source: SourcedNameSource::Id,
                    };
                    sourced_names_vec.push(entry);
                }
                if let Some(name_atom) = child.get_name() {
                    let entry = SourcedName {
                        name: name_atom,
                        element: DomRoot::from_ref(child),
                        source: SourcedNameSource::Name,
                    };
                    sourced_names_vec.push(entry);
                }
            }
        }

        // Step 4
        let past_names_map = self.past_names_map.borrow();
        for (key, val) in past_names_map.iter() {
            let entry = SourcedName {
                name: key.clone(),
                element: DomRoot::from_ref(&*val.0),
                source: SourcedNameSource::Past(self.current_name_generation.get() - val.1.0),
            };
            sourced_names_vec.push(entry);
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

        sourced_names_vec.sort_by(|a, b| {
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
            } else if a
                .element
                .upcast::<Node>()
                .CompareDocumentPosition(b.element.upcast::<Node>()) &
                NodeConstants::DOCUMENT_POSITION_FOLLOWING ==
                NodeConstants::DOCUMENT_POSITION_FOLLOWING
            {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });

        // Step 6
        sourced_names_vec.retain(|sn| !sn.name.to_string().is_empty());

        // Step 7-8
        let mut names_vec: Vec<DOMString> = Vec::new();
        for elem in sourced_names_vec.iter() {
            if !names_vec.iter().any(|name| *name == *elem.name) {
                names_vec.push(DOMString::from(&*elem.name));
            }
        }

        names_vec
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-form-checkvalidity>
    fn CheckValidity(&self, can_gc: CanGc) -> bool {
        self.static_validation(can_gc).is_ok()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-form-reportvalidity>
    fn ReportValidity(&self, can_gc: CanGc) -> bool {
        self.interactive_validation(can_gc).is_ok()
    }
}

#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
pub(crate) enum SubmittedFrom {
    FromForm,
    NotFromForm,
}

#[derive(Clone, Copy, MallocSizeOf)]
pub(crate) enum ResetFrom {
    FromForm,
    NotFromForm,
}

impl HTMLFormElement {
    /// <https://html.spec.whatwg.org/multipage/#picking-an-encoding-for-the-form>
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
            let input = input.str();
            let mut candidate_encodings =
                split_html_space_chars(&input).filter_map(|c| Encoding::for_label(c.as_bytes()));

            // Substep 5, 6
            return candidate_encodings.next().unwrap_or(UTF_8);
        }

        // Step 1, 3
        self.owner_document().encoding()
    }

    /// <https://html.spec.whatwg.org/multipage/#text/plain-encoding-algorithm>
    fn encode_plaintext(&self, form_data: &mut [FormDatum]) -> String {
        // Step 1
        let mut result = String::new();

        // Step 2
        for entry in form_data.iter() {
            let value = match &entry.value {
                FormDatumValue::File(f) => f.name(),
                FormDatumValue::String(s) => s,
            };
            result.push_str(&format!("{}={}\r\n", entry.name, value));
        }

        // Step 3
        result
    }

    pub(crate) fn update_validity(&self, can_gc: CanGc) {
        let is_any_invalid = self
            .controls
            .borrow()
            .iter()
            .any(|control| control.is_invalid(false, can_gc));

        self.upcast::<Element>()
            .set_state(ElementState::VALID, !is_any_invalid);
        self.upcast::<Element>()
            .set_state(ElementState::INVALID, is_any_invalid);
    }

    /// [Form submission](https://html.spec.whatwg.org/multipage/#concept-form-submit)
    pub(crate) fn submit(
        &self,
        submit_method_flag: SubmittedFrom,
        submitter: FormSubmitterElement,
        can_gc: CanGc,
    ) {
        // Step 1
        if self.upcast::<Element>().cannot_navigate() {
            return;
        }

        // Step 2
        if self.constructing_entry_list.get() {
            return;
        }
        // Step 3. Let form document be form's node document.
        let doc = self.owner_document();

        // Step 4. If form document's active sandboxing flag set has its sandboxed forms browsing
        // context flag set, then return.
        if doc.has_active_sandboxing_flag(SandboxingFlagSet::SANDBOXED_FORMS_BROWSING_CONTEXT_FLAG)
        {
            return;
        }

        let base = doc.base_url();
        // TODO: Handle browsing contexts (Step 5)
        // Step 6
        if submit_method_flag == SubmittedFrom::NotFromForm {
            // Step 6.1
            if self.firing_submission_events.get() {
                return;
            }
            // Step 6.2
            self.firing_submission_events.set(true);
            // Step 6.3
            if !submitter.no_validate(self) && self.interactive_validation(can_gc).is_err() {
                self.firing_submission_events.set(false);
                return;
            }
            // Step 6.4
            // spec calls this "submitterButton" but it doesn't have to be a button,
            // just not be the form itself
            let submitter_button = match submitter {
                FormSubmitterElement::Form(f) => {
                    if f == self {
                        None
                    } else {
                        Some(f.upcast::<HTMLElement>())
                    }
                },
                FormSubmitterElement::Input(i) => Some(i.upcast::<HTMLElement>()),
                FormSubmitterElement::Button(b) => Some(b.upcast::<HTMLElement>()),
            };

            // Step 6.5
            let event = SubmitEvent::new(
                self.global().as_window(),
                atom!("submit"),
                true,
                true,
                submitter_button.map(DomRoot::from_ref),
                can_gc,
            );
            let event = event.upcast::<Event>();
            event.fire(self.upcast::<EventTarget>(), can_gc);

            // Step 6.6
            self.firing_submission_events.set(false);
            // Step 6.7
            if event.DefaultPrevented() {
                return;
            }
            // Step 6.8
            if self.upcast::<Element>().cannot_navigate() {
                return;
            }
        }

        // Step 7
        let encoding = self.pick_encoding();

        // Step 8
        let mut form_data = match self.get_form_dataset(Some(submitter), Some(encoding), can_gc) {
            Some(form_data) => form_data,
            None => return,
        };

        // Step 9. If form cannot navigate, then return.
        if self.upcast::<Element>().cannot_navigate() {
            return;
        }

        // Step 10. Let method be the submitter element's method.
        let method = submitter.method();
        // Step 11. If method is dialog, then:
        // TODO

        // Step 12. Let action be the submitter element's action.
        let mut action = submitter.action();

        // Step 13. If action is the empty string, let action be the URL of the form document.
        if action.is_empty() {
            action = DOMString::from(base.as_str());
        }
        // Step 14. Let parsed action be the result of encoding-parsing a URL given action, relative to submitter's node document.
        let action_components = match base.join(&action.str()) {
            Ok(url) => url,
            // Step 15. If parsed action is failure, then return.
            Err(_) => return,
        };
        // Step 16. Let scheme be the scheme of parsed action.
        let scheme = action_components.scheme().to_owned();
        // Step 17. Let enctype be the submitter element's enctype.
        let enctype = submitter.enctype();

        // Step 19. If the submitter element is a submit button and it has a formtarget attribute,
        // then set formTarget to the formtarget attribute value.
        let form_target_attribute = submitter.target();
        let form_target = if submitter.is_submit_button() &&
            valid_navigable_target_name_or_keyword(&form_target_attribute)
        {
            Some(form_target_attribute)
        } else {
            // Step 18. Let formTarget be null.
            None
        };
        // Step 20. Let target be the result of getting an element's target given submitter's form owner and formTarget.
        let form_owner = submitter.form_owner();
        let form = form_owner.as_deref().unwrap_or(self);
        let target = get_element_target(form.upcast::<Element>(), form_target);

        // Step 21. Let noopener be the result of getting an element's noopener with form, parsed action, and target.
        let noopener = self.relations.get().get_element_noopener(target.as_ref());

        // Step 22. Let targetNavigable be the first return value of applying the rules for choosing a navigable given target,
        // form's node navigable, and noopener.
        let source = doc.browsing_context().unwrap();
        let (maybe_chosen, _new) =
            source.choose_browsing_context(target.unwrap_or_default(), noopener);

        let Some(chosen) = maybe_chosen else {
            // Step 23. If targetNavigable is null, then return.
            return;
        };
        // Step 24. Let historyHandling be "auto".
        // TODO
        // Step 25. If form document equals targetNavigable's active document, and form document has not yet completely loaded,
        // then set historyHandling to "replace".
        // TODO

        let target_document = match chosen.document() {
            Some(doc) => doc,
            None => return,
        };
        let target_window = target_document.window();
        let mut load_data = LoadData::new(
            LoadOrigin::Script(doc.origin().snapshot()),
            action_components,
            target_document.about_base_url(),
            None,
            target_window.as_global_scope().get_referrer(),
            target_document.get_referrer_policy(),
            Some(target_window.as_global_scope().is_secure_context()),
            Some(target_document.insecure_requests_policy()),
            target_document.has_trustworthy_ancestor_origin(),
            target_document.creation_sandboxing_flag_set_considering_parent_iframe(),
        );

        // Step 26. Select the appropriate row in the table below based on scheme as given by the first cell of each row.
        // Then, select the appropriate cell on that row based on method as given in the first cell of each column.
        // Then, jump to the steps named in that cell and defined below the table.
        match (&*scheme, method) {
            (_, FormMethod::Dialog) => {
                // TODO: Submit dialog
                // https://html.spec.whatwg.org/multipage/#submit-dialog
            },
            // https://html.spec.whatwg.org/multipage/#submit-mutate-action
            ("http", FormMethod::Get) | ("https", FormMethod::Get) | ("data", FormMethod::Get) => {
                load_data
                    .headers
                    .typed_insert(ContentType::from(mime::APPLICATION_WWW_FORM_URLENCODED));
                self.mutate_action_url(&mut form_data, load_data, encoding, target_window);
            },
            // https://html.spec.whatwg.org/multipage/#submit-body
            ("http", FormMethod::Post) | ("https", FormMethod::Post) => {
                load_data.method = Method::POST;
                self.submit_entity_body(
                    &mut form_data,
                    load_data,
                    enctype,
                    encoding,
                    target_window,
                    can_gc,
                );
            },
            // https://html.spec.whatwg.org/multipage/#submit-get-action
            ("file", _) |
            ("about", _) |
            ("data", FormMethod::Post) |
            ("ftp", _) |
            ("javascript", _) => {
                self.plan_to_navigate(load_data, target_window);
            },
            ("mailto", FormMethod::Post) => {
                // TODO: Mail as body
                // https://html.spec.whatwg.org/multipage/#submit-mailto-body
            },
            ("mailto", FormMethod::Get) => {
                // TODO: Mail with headers
                // https://html.spec.whatwg.org/multipage/#submit-mailto-headers
            },
            _ => (),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#submit-mutate-action>
    fn mutate_action_url(
        &self,
        form_data: &mut [FormDatum],
        mut load_data: LoadData,
        encoding: &'static Encoding,
        target: &Window,
    ) {
        let charset = encoding.name();

        self.set_url_query_pairs(
            &mut load_data.url,
            form_data
                .iter()
                .map(|field| (field.name.str(), field.replace_value(charset))),
        );

        self.plan_to_navigate(load_data, target);
    }

    /// <https://html.spec.whatwg.org/multipage/#submit-body>
    fn submit_entity_body(
        &self,
        form_data: &mut [FormDatum],
        mut load_data: LoadData,
        enctype: FormEncType,
        encoding: &'static Encoding,
        target: &Window,
        can_gc: CanGc,
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
                        .map(|field| (field.name.str(), field.replace_value(charset))),
                );

                url.query().unwrap_or("").to_string().into_bytes()
            },
            FormEncType::MultipartFormData => {
                let mime: Mime = format!("multipart/form-data; boundary={}", boundary)
                    .parse()
                    .unwrap();
                load_data.headers.typed_insert(ContentType::from(mime));
                encode_multipart_form_data(form_data, boundary, encoding)
            },
            FormEncType::TextPlain => {
                load_data
                    .headers
                    .typed_insert(ContentType::from(mime::TEXT_PLAIN));
                self.encode_plaintext(form_data).into_bytes()
            },
        };

        let global = self.global();

        let request_body = bytes
            .extract(&global, false, can_gc)
            .expect("Couldn't extract body.")
            .into_net_request_body()
            .0;
        load_data.data = Some(request_body);

        self.plan_to_navigate(load_data, target);
    }

    fn set_url_query_pairs<T>(
        &self,
        url: &mut servo_url::ServoUrl,
        pairs: impl Iterator<Item = (T, String)>,
    ) where
        T: AsRef<str>,
    {
        let encoding = self.pick_encoding();
        url.as_mut_url()
            .query_pairs_mut()
            .encoding_override(Some(&|s| encoding.encode(s).0))
            .clear()
            .extend_pairs(pairs);
    }

    /// [Planned navigation](https://html.spec.whatwg.org/multipage/#planned-navigation)
    fn plan_to_navigate(&self, mut load_data: LoadData, target: &Window) {
        // 1. Let referrerPolicy be the empty string.
        // 2. If the form element's link types include the noreferrer keyword,
        //    then set referrerPolicy to "no-referrer".
        // Note: both steps done below.
        let elem = self.upcast::<Element>();
        let referrer = match elem.get_attribute(&ns!(), &local_name!("rel")) {
            Some(ref link_types) if link_types.Value().contains("noreferrer") => {
                Referrer::NoReferrer
            },
            _ => target.as_global_scope().get_referrer(),
        };

        // 3. If the form has a non-null planned navigation, remove it from its task queue.
        // Note: done by incrementing `planned_navigation`.
        self.planned_navigation
            .set(self.planned_navigation.get().wrapping_add(1));
        let planned_navigation = self.planned_navigation.get();

        // Note: we start to use
        // the beginnings of an `ongoing_navigation` concept,
        // to cancel planned navigations as part of
        // <https://html.spec.whatwg.org/multipage/#nav-stop>
        //
        // The concept of ongoing navigation must be separated from the form's
        // planned navigation concept, because each planned navigation cancels the previous one
        // for a given form, whereas an ongoing navigation is a per navigable (read: window for now)
        // concept.
        //
        // Setting the ongoing navigation now means the navigation could be cancelled
        // even if the below task has not run yet. This is not how the spec is written: it
        // seems instead to imply that a `window.stop` should only cancel the navigation
        // that has already started (here the task is queued, but the navigation starts only
        // in the task). See <https://github.com/whatwg/html/issues/11562>.
        let ongoing_navigation = target.set_ongoing_navigation();

        let referrer_policy = target.Document().get_referrer_policy();
        load_data.creator_pipeline_id = Some(target.pipeline_id());
        load_data.referrer = referrer;
        load_data.referrer_policy = referrer_policy;

        // Note the pending form navigation if this is an iframe;
        // necessary for deciding whether to run the iframe load event steps.
        if let Some(window_proxy) = target.undiscarded_window_proxy() {
            if let Some(frame) = window_proxy
                .frame_element()
                .and_then(|e| e.downcast::<HTMLIFrameElement>())
            {
                frame.note_pending_navigation()
            }
        }

        // 4. Queue an element task on the DOM manipulation task source
        // given the form element and the following steps:
        let form = Trusted::new(self);
        let window = Trusted::new(target);
        let task = task!(navigate_to_form_planned_navigation: move || {
            // 4.1 Set the form's planned navigation to null.
            // Note: we implement the equivalent by incrementing the counter above,
            // and checking it here.
            if planned_navigation != form.root().planned_navigation.get() {
                return;
            }

            // Note: we also check if the navigation has been cancelled,
            // see https://github.com/whatwg/html/issues/11562
            if ongoing_navigation != window.root().ongoing_navigation() {
                return;
            }

            // 4.2 Navigate targetNavigable to url
            window
                .root()
                .load_url(
                    NavigationHistoryBehavior::Push,
                    false,
                    load_data,
                    CanGc::note(),
                );
        });

        // 5. Set the form's planned navigation to the just-queued task.
        // Done above as part of incrementing the planned navigation counter.

        // Note: task queued here.
        target
            .global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task)
    }

    /// Interactively validate the constraints of form elements
    /// <https://html.spec.whatwg.org/multipage/#interactively-validate-the-constraints>
    fn interactive_validation(&self, can_gc: CanGc) -> Result<(), ()> {
        // Step 1 - 2: Statically validate the constraints of form,
        // and let `unhandled invalid controls` be the list of elements
        // returned if the result was negative.
        // If the result was positive, then return that result.
        let unhandled_invalid_controls = match self.static_validation(can_gc) {
            Ok(()) => return Ok(()),
            Err(err) => err,
        };

        // Step 3: Report the problems with the constraints of at least one of the elements
        // given in unhandled invalid controls to the user.
        let mut first = true;

        for elem in unhandled_invalid_controls {
            if let Some(validatable) = elem.as_maybe_validatable() {
                error!("Validation error: {}", validatable.validation_message());
            }
            if first {
                if let Some(html_elem) = elem.downcast::<HTMLElement>() {
                    // Step 3.1: User agents may focus one of those elements in the process,
                    // by running the focusing steps for that element,
                    // and may change the scrolling position of the document, or perform
                    // some other action that brings the element to the user's attention.

                    // Here we run focusing steps and scroll element into view.
                    html_elem.Focus(&FocusOptions::default(), can_gc);
                    first = false;
                }
            }
        }

        // If it's form-associated and has a validation anchor, point the
        //  user there instead of the element itself.
        // Step 4
        Err(())
    }

    /// Statitically validate the constraints of form elements
    /// <https://html.spec.whatwg.org/multipage/#statically-validate-the-constraints>
    fn static_validation(&self, can_gc: CanGc) -> Result<(), Vec<DomRoot<Element>>> {
        // Step 1-3
        let invalid_controls = self
            .controls
            .borrow()
            .iter()
            .filter_map(|field| {
                if let Some(element) = field.downcast::<Element>() {
                    if element.is_invalid(true, can_gc) {
                        Some(DomRoot::from_ref(element))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<DomRoot<Element>>>();
        // Step 4: If invalid controls is empty, then return a positive result.
        if invalid_controls.is_empty() {
            return Ok(());
        }
        // Step 5-6
        let unhandled_invalid_controls = invalid_controls
            .into_iter()
            .filter_map(|field| {
                // Step 6.1: Let notCanceled be the result of firing an event named invalid at
                // field, with the cancelable attribute initialized to true.
                let not_canceled = field
                    .upcast::<EventTarget>()
                    .fire_cancelable_event(atom!("invalid"), can_gc);
                // Step 6.2: If notCanceled is true, then add field to unhandled invalid controls.
                if not_canceled {
                    return Some(field);
                }
                None
            })
            .collect::<Vec<DomRoot<Element>>>();
        // Step 7
        Err(unhandled_invalid_controls)
    }

    /// <https://html.spec.whatwg.org/multipage/#constructing-the-form-data-set>
    /// terminology note:  "form data set" = "entry list"
    /// Steps range from 3 to 5
    /// 5.x substeps are mostly handled inside element-specific methods
    fn get_unclean_dataset(
        &self,
        submitter: Option<FormSubmitterElement>,
        encoding: Option<&'static Encoding>,
        can_gc: CanGc,
    ) -> Vec<FormDatum> {
        let mut data_set = Vec::new();
        for child in self.controls.borrow().iter() {
            // Step 5.1: The field element is disabled.
            if child.disabled_state() {
                continue;
            }
            let child = child.upcast::<Node>();

            // Step 5.1: The field element has a datalist element ancestor.
            if child.ancestors().any(|a| a.is::<HTMLDataListElement>()) {
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
                                name,
                                value: FormDatumValue::String(textarea.Value()),
                            });
                        }
                    },
                    HTMLElementTypeId::HTMLElement => {
                        let custom = child.downcast::<HTMLElement>().unwrap();
                        if custom.is_form_associated_custom_element() {
                            // https://html.spec.whatwg.org/multipage/#face-entry-construction
                            let internals =
                                custom.upcast::<Element>().ensure_element_internals(can_gc);
                            internals.perform_entry_construction(&mut data_set);
                            // Otherwise no form value has been set so there is nothing to do.
                        }
                    },
                    _ => (),
                }
            }

            // Step: 5.13. Add an entry if element has dirname attribute
            // An element can only have a dirname attribute if it is a textarea element
            // or an input element whose type attribute is in either the Text state or the Search state
            let child_element = child.downcast::<Element>().unwrap();
            let input_matches = child_element
                .downcast::<HTMLInputElement>()
                .is_some_and(|input| {
                    matches!(input.input_type(), InputType::Text | InputType::Search)
                });
            let textarea_matches = child_element.is::<HTMLTextAreaElement>();
            let dirname = child_element.get_string_attribute(&local_name!("dirname"));
            if (input_matches || textarea_matches) && !dirname.is_empty() {
                let dir = DOMString::from(child_element.directionality());
                data_set.push(FormDatum {
                    ty: DOMString::from("string"),
                    name: dirname,
                    value: FormDatumValue::String(dir),
                });
            }
        }
        data_set
    }

    /// <https://html.spec.whatwg.org/multipage/#constructing-the-form-data-set>
    pub(crate) fn get_form_dataset(
        &self,
        submitter: Option<FormSubmitterElement>,
        encoding: Option<&'static Encoding>,
        can_gc: CanGc,
    ) -> Option<Vec<FormDatum>> {
        // Step 1
        if self.constructing_entry_list.get() {
            return None;
        }

        // Step 2
        self.constructing_entry_list.set(true);

        // Step 3-6
        let ret = self.get_unclean_dataset(submitter, encoding, can_gc);

        let window = self.owner_window();

        // Step 6
        let form_data = FormData::new(Some(ret), &window.global(), can_gc);

        // Step 7
        let event = FormDataEvent::new(
            &window,
            atom!("formdata"),
            EventBubbles::Bubbles,
            EventCancelable::NotCancelable,
            &form_data,
            can_gc,
        );

        event
            .upcast::<Event>()
            .fire(self.upcast::<EventTarget>(), can_gc);

        // Step 8
        self.constructing_entry_list.set(false);

        // Step 9
        Some(form_data.datums())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-form-reset>
    pub(crate) fn reset(&self, _reset_method_flag: ResetFrom, can_gc: CanGc) {
        // https://html.spec.whatwg.org/multipage/#locked-for-reset
        if self.marked_for_reset.get() {
            return;
        } else {
            self.marked_for_reset.set(true);
        }

        // https://html.spec.whatwg.org/multipage/#concept-form-reset
        // Let reset be the result of firing an event named reset at form,
        // with the bubbles and cancelable attributes initialized to true.
        let reset = self
            .upcast::<EventTarget>()
            .fire_bubbling_cancelable_event(atom!("reset"), can_gc);
        if !reset {
            return;
        }

        let controls: Vec<_> = self
            .controls
            .borrow()
            .iter()
            .map(|c| c.as_rooted())
            .collect();

        for child in controls {
            child.reset(can_gc);
        }
        self.marked_for_reset.set(false);
    }

    fn add_control<T: ?Sized + FormControl>(&self, control: &T, can_gc: CanGc) {
        {
            let root = self.upcast::<Element>().root_element();
            let root = root.upcast::<Node>();
            let mut controls = self.controls.borrow_mut();
            controls.insert_pre_order(control.to_element(), root);
        }
        self.update_validity(can_gc);
    }

    fn remove_control<T: ?Sized + FormControl>(&self, control: &T, can_gc: CanGc) {
        {
            let control = control.to_element();
            let mut controls = self.controls.borrow_mut();
            controls
                .iter()
                .position(|c| &**c == control)
                .map(|idx| controls.remove(idx));

            // https://html.spec.whatwg.org/multipage#forms.html#the-form-element:past-names-map-5
            // "If an element listed in a form element's past names map
            // changes form owner, then its entries must be removed
            // from that map."
            let mut past_names_map = self.past_names_map.borrow_mut();
            past_names_map.0.retain(|_k, v| v.0 != control);
        }
        self.update_validity(can_gc);
    }
}

impl Element {
    pub(crate) fn is_resettable(&self) -> bool {
        let NodeTypeId::Element(ElementTypeId::HTMLElement(element_type)) =
            self.upcast::<Node>().type_id()
        else {
            return false;
        };
        matches!(
            element_type,
            HTMLElementTypeId::HTMLInputElement |
                HTMLElementTypeId::HTMLSelectElement |
                HTMLElementTypeId::HTMLTextAreaElement |
                HTMLElementTypeId::HTMLOutputElement |
                HTMLElementTypeId::HTMLElement
        )
    }

    pub(crate) fn reset(&self, can_gc: CanGc) {
        if !self.is_resettable() {
            return;
        }

        if let Some(input_element) = self.downcast::<HTMLInputElement>() {
            input_element.reset(can_gc);
        } else if let Some(select_element) = self.downcast::<HTMLSelectElement>() {
            select_element.reset();
        } else if let Some(textarea_element) = self.downcast::<HTMLTextAreaElement>() {
            textarea_element.reset(can_gc);
        } else if let Some(output_element) = self.downcast::<HTMLOutputElement>() {
            output_element.reset(can_gc);
        } else if let Some(html_element) = self.downcast::<HTMLElement>() {
            if html_element.is_form_associated_custom_element() {
                ScriptThread::enqueue_callback_reaction(
                    html_element.upcast::<Element>(),
                    CallbackReaction::FormReset,
                    None,
                )
            }
        }
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
pub(crate) enum FormDatumValue {
    File(DomRoot<File>),
    String(DOMString),
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
pub(crate) struct FormDatum {
    pub(crate) ty: DOMString,
    pub(crate) name: DOMString,
    pub(crate) value: FormDatumValue,
}

impl FormDatum {
    pub(crate) fn replace_value(&self, charset: &str) -> String {
        if self.name.to_ascii_lowercase() == "_charset_" && self.ty == "hidden" {
            return charset.to_string();
        }

        match self.value {
            FormDatumValue::File(ref f) => String::from(f.name().clone()),
            FormDatumValue::String(ref s) => String::from(s.clone()),
        }
    }
}

#[derive(Clone, Copy, MallocSizeOf)]
pub(crate) enum FormEncType {
    TextPlain,
    UrlEncoded,
    MultipartFormData,
}

#[derive(Clone, Copy, MallocSizeOf)]
pub(crate) enum FormMethod {
    Get,
    Post,
    Dialog,
}

/// <https://html.spec.whatwg.org/multipage/#form-associated-element>
#[derive(Clone, Copy, MallocSizeOf)]
pub(crate) enum FormSubmitterElement<'a> {
    Form(&'a HTMLFormElement),
    Input(&'a HTMLInputElement),
    Button(&'a HTMLButtonElement),
    // TODO: implement other types of form associated elements
    // (including custom elements) that can be passed as submitter.
}

impl FormSubmitterElement<'_> {
    fn action(&self) -> DOMString {
        match *self {
            FormSubmitterElement::Form(form) => form.Action(),
            FormSubmitterElement::Input(input_element) => input_element.get_form_attribute(
                &local_name!("formaction"),
                |i| i.FormAction(),
                |f| f.Action(),
            ),
            FormSubmitterElement::Button(button_element) => button_element.get_form_attribute(
                &local_name!("formaction"),
                |i| i.FormAction(),
                |f| f.Action(),
            ),
        }
    }

    fn enctype(&self) -> FormEncType {
        let attr = match *self {
            FormSubmitterElement::Form(form) => form.Enctype(),
            FormSubmitterElement::Input(input_element) => input_element.get_form_attribute(
                &local_name!("formenctype"),
                |i| i.FormEnctype(),
                |f| f.Enctype(),
            ),
            FormSubmitterElement::Button(button_element) => button_element.get_form_attribute(
                &local_name!("formenctype"),
                |i| i.FormEnctype(),
                |f| f.Enctype(),
            ),
        };
        // https://html.spec.whatwg.org/multipage/#attr-fs-enctype
        // urlencoded is the default
        match_domstring_ascii!(attr,
            "multipart/form-data" => FormEncType::MultipartFormData,
            "text/plain" => FormEncType::TextPlain,
            _ => FormEncType::UrlEncoded,
        )
    }

    fn method(&self) -> FormMethod {
        let attr = match *self {
            FormSubmitterElement::Form(form) => form.Method(),
            FormSubmitterElement::Input(input_element) => input_element.get_form_attribute(
                &local_name!("formmethod"),
                |i| i.FormMethod(),
                |f| f.Method(),
            ),
            FormSubmitterElement::Button(button_element) => button_element.get_form_attribute(
                &local_name!("formmethod"),
                |i| i.FormMethod(),
                |f| f.Method(),
            ),
        };
        match_domstring_ascii!(attr,
            "dialog" => FormMethod::Dialog,
            "post" => FormMethod::Post,
            _ => FormMethod::Get,
        )
    }

    fn target(&self) -> DOMString {
        match *self {
            FormSubmitterElement::Form(form) => form.Target(),
            FormSubmitterElement::Input(input_element) => input_element.get_form_attribute(
                &local_name!("formtarget"),
                |i| i.FormTarget(),
                |f| f.Target(),
            ),
            FormSubmitterElement::Button(button_element) => button_element.get_form_attribute(
                &local_name!("formtarget"),
                |i| i.FormTarget(),
                |f| f.Target(),
            ),
        }
    }

    fn no_validate(&self, _form_owner: &HTMLFormElement) -> bool {
        match *self {
            FormSubmitterElement::Form(form) => form.NoValidate(),
            FormSubmitterElement::Input(input_element) => input_element.get_form_boolean_attribute(
                &local_name!("formnovalidate"),
                |i| i.FormNoValidate(),
                |f| f.NoValidate(),
            ),
            FormSubmitterElement::Button(button_element) => button_element
                .get_form_boolean_attribute(
                    &local_name!("formnovalidate"),
                    |i| i.FormNoValidate(),
                    |f| f.NoValidate(),
                ),
        }
    }

    // https://html.spec.whatwg.org/multipage/#concept-submit-button
    pub(crate) fn is_submit_button(&self) -> bool {
        match *self {
            // https://html.spec.whatwg.org/multipage/#image-button-state-(type=image)
            // https://html.spec.whatwg.org/multipage/#submit-button-state-(type=submit)
            FormSubmitterElement::Input(input_element) => input_element.is_submit_button(),
            // https://html.spec.whatwg.org/multipage/#attr-button-type-submit-state
            FormSubmitterElement::Button(button_element) => button_element.is_submit_button(),
            _ => false,
        }
    }

    // https://html.spec.whatwg.org/multipage/#form-owner
    pub(crate) fn form_owner(&self) -> Option<DomRoot<HTMLFormElement>> {
        match *self {
            FormSubmitterElement::Button(button_el) => button_el.form_owner(),
            FormSubmitterElement::Input(input_el) => input_el.form_owner(),
            _ => None,
        }
    }
}

pub(crate) trait FormControl: DomObject<ReflectorType = ()> {
    fn form_owner(&self) -> Option<DomRoot<HTMLFormElement>>;

    fn set_form_owner(&self, form: Option<&HTMLFormElement>);

    fn to_element(&self) -> &Element;

    fn is_listed(&self) -> bool {
        true
    }

    // https://html.spec.whatwg.org/multipage/#create-an-element-for-the-token
    // Part of step 12.
    // '..suppress the running of the reset the form owner algorithm
    // when the parser subsequently attempts to insert the element..'
    fn set_form_owner_from_parser(&self, form: &HTMLFormElement, can_gc: CanGc) {
        let elem = self.to_element();
        let node = elem.upcast::<Node>();
        node.set_flag(NodeFlags::PARSER_ASSOCIATED_FORM_OWNER, true);
        form.add_control(self, can_gc);
        self.set_form_owner(Some(form));
    }

    /// <https://html.spec.whatwg.org/multipage/#reset-the-form-owner>
    fn reset_form_owner(&self, can_gc: CanGc) {
        let elem = self.to_element();
        let node = elem.upcast::<Node>();
        let old_owner = self.form_owner();
        let has_form_id = elem.has_attribute(&local_name!("form"));
        let nearest_form_ancestor = node
            .ancestors()
            .find_map(DomRoot::downcast::<HTMLFormElement>);

        // Step 1
        if old_owner.is_some() &&
            !(self.is_listed() && has_form_id) &&
            nearest_form_ancestor == old_owner
        {
            return;
        }

        let new_owner = if self.is_listed() && has_form_id && elem.is_connected() {
            // Step 3
            let doc = node.owner_document();
            let form_id = elem.get_string_attribute(&local_name!("form"));
            doc.GetElementById(form_id)
                .and_then(DomRoot::downcast::<HTMLFormElement>)
        } else {
            // Step 4
            nearest_form_ancestor
        };

        if old_owner != new_owner {
            if let Some(o) = old_owner {
                o.remove_control(self, can_gc);
            }
            if let Some(ref new_owner) = new_owner {
                new_owner.add_control(self, can_gc);
            }
            // https://html.spec.whatwg.org/multipage/#custom-element-reactions:reset-the-form-owner
            if let Some(html_elem) = elem.downcast::<HTMLElement>() {
                if html_elem.is_form_associated_custom_element() {
                    ScriptThread::enqueue_callback_reaction(
                        elem,
                        CallbackReaction::FormAssociated(
                            new_owner.as_ref().map(|form| DomRoot::from_ref(&**form)),
                        ),
                        None,
                    )
                }
            }
            self.set_form_owner(new_owner.as_deref());
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#association-of-controls-and-forms>
    fn form_attribute_mutated(&self, mutation: AttributeMutation, can_gc: CanGc) {
        match mutation {
            AttributeMutation::Set(..) => {
                self.register_if_necessary();
            },
            AttributeMutation::Removed => {
                self.unregister_if_necessary();
            },
        }

        self.reset_form_owner(can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#association-of-controls-and-forms>
    fn register_if_necessary(&self) {
        let elem = self.to_element();
        let form_id = elem.get_string_attribute(&local_name!("form"));
        let node = elem.upcast::<Node>();

        if self.is_listed() && !form_id.is_empty() && node.is_connected() {
            node.owner_document()
                .register_form_id_listener(form_id, self);
        }
    }

    fn unregister_if_necessary(&self) {
        let elem = self.to_element();
        let form_id = elem.get_string_attribute(&local_name!("form"));

        if self.is_listed() && !form_id.is_empty() {
            elem.owner_document()
                .unregister_form_id_listener(form_id, self);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#association-of-controls-and-forms>
    fn bind_form_control_to_tree(&self, can_gc: CanGc) {
        let elem = self.to_element();
        let node = elem.upcast::<Node>();

        // https://html.spec.whatwg.org/multipage/#create-an-element-for-the-token
        // Part of step 12.
        // '..suppress the running of the reset the form owner algorithm
        // when the parser subsequently attempts to insert the element..'
        let must_skip_reset = node.get_flag(NodeFlags::PARSER_ASSOCIATED_FORM_OWNER);
        node.set_flag(NodeFlags::PARSER_ASSOCIATED_FORM_OWNER, false);

        if !must_skip_reset {
            self.form_attribute_mutated(
                AttributeMutation::Set(None, AttributeMutationReason::Directly),
                can_gc,
            );
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#association-of-controls-and-forms>
    fn unbind_form_control_from_tree(&self, can_gc: CanGc) {
        let elem = self.to_element();
        let has_form_attr = elem.has_attribute(&local_name!("form"));
        let same_subtree = self
            .form_owner()
            .is_none_or(|form| elem.is_in_same_home_subtree(&*form));

        self.unregister_if_necessary();

        // Since this control has been unregistered from the id->listener map
        // in the previous step, reset_form_owner will not be invoked on it
        // when the form owner element is unbound (i.e it is in the same
        // subtree) if it appears later in the tree order. Hence invoke
        // reset from here if this control has the form attribute set.
        if !same_subtree || (self.is_listed() && has_form_attr) {
            self.reset_form_owner(can_gc);
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
            self.form_owner().is_some_and(|t| owner(&t))
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#candidate-for-constraint-validation>
    fn is_candidate_for_constraint_validation(&self) -> bool {
        let element = self.to_element();
        let html_element = element.downcast::<HTMLElement>();
        if let Some(html_element) = html_element {
            html_element.is_submittable_element() || element.is_instance_validatable()
        } else {
            false
        }
    }

    // XXXKiChjang: Implement these on inheritors
    // fn satisfies_constraints(&self) -> bool;
}

impl VirtualMethods for HTMLFormElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        self.super_type().unwrap().unbind_from_tree(context, can_gc);

        // Collect the controls to reset because reset_form_owner
        // will mutably borrow self.controls
        rooted_vec!(let mut to_reset);
        to_reset.extend(
            self.controls
                .borrow()
                .iter()
                .filter(|c| !c.is_in_same_home_subtree(self))
                .cloned(),
        );

        for control in to_reset.iter() {
            control
                .as_maybe_form_control()
                .expect("Element must be a form control")
                .reset_form_owner(can_gc);
        }
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("rel") => AttrValue::from_serialized_tokenlist(value.into()),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .attribute_mutated(attr, mutation, can_gc);

        match *attr.local_name() {
            local_name!("rel") | local_name!("rev") => {
                self.relations
                    .set(LinkRelations::for_element(self.upcast()));
            },
            _ => {},
        }
    }

    fn bind_to_tree(&self, context: &BindContext, can_gc: CanGc) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context, can_gc);
        }

        self.relations
            .set(LinkRelations::for_element(self.upcast()));
    }
}

pub(crate) trait FormControlElementHelpers {
    fn as_maybe_form_control(&self) -> Option<&dyn FormControl>;
}

impl FormControlElementHelpers for Element {
    fn as_maybe_form_control(&self) -> Option<&dyn FormControl> {
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
            _ => self.downcast::<HTMLElement>().and_then(|elem| {
                if elem.is_form_associated_custom_element() {
                    Some(elem as &dyn FormControl)
                } else {
                    None
                }
            }),
        }
    }
}

/// <https://html.spec.whatwg.org/multipage/#multipart/form-data-encoding-algorithm>
pub(crate) fn encode_multipart_form_data(
    form_data: &mut [FormDatum],
    boundary: String,
    encoding: &'static Encoding,
) -> Vec<u8> {
    let mut result = vec![];

    // Newline replacement routine as described in Step 1
    fn clean_crlf(s: &DOMString) -> DOMString {
        let mut buf = "".to_owned();
        let mut prev = ' ';
        for ch in s.str().chars() {
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

    for entry in form_data.iter_mut() {
        // Step 1.1: Perform newline replacement on entry's name
        entry.name = clean_crlf(&entry.name);

        // Step 1.2: If entry's value is not a File object, perform newline replacement on entry's
        // value
        if let FormDatumValue::String(ref s) = entry.value {
            entry.value = FormDatumValue::String(clean_crlf(s));
        }

        // Step 2: Return the byte sequence resulting from encoding the entry list.
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
                    format!("Content-Disposition: {content_disposition}\r\n\r\n{s}\r\n",)
                        .into_bytes();
                result.append(&mut bytes);
            },
            FormDatumValue::File(ref f) => {
                let charset = encoding.name();
                let extra = if charset.to_lowercase() == "utf-8" {
                    format!("filename=\"{}\"", String::from(f.name().str()))
                } else {
                    format!(
                        "filename*=\"{}\"''{}",
                        charset,
                        http_percent_encode(&f.name().as_bytes())
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
                    "Content-Disposition: {}\r\nContent-Type: {}\r\n\r\n",
                    content_disposition, content_type
                )
                .into_bytes();
                result.append(&mut type_bytes);

                let mut bytes = f.upcast::<Blob>().get_bytes().unwrap_or(vec![]);

                result.append(&mut bytes);
                result.extend(b"\r\n");
            },
        }
    }

    let mut boundary_bytes = format!("--{boundary}--\r\n").into_bytes();
    result.append(&mut boundary_bytes);

    result
}

// https://tools.ietf.org/html/rfc7578#section-4.1
pub(crate) fn generate_boundary() -> String {
    let i1 = random::<u32>();
    let i2 = random::<u32>();

    format!("---------------------------{0}{1}", i1, i2)
}
