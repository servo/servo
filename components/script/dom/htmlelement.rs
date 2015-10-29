/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::attr::AttrValue;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::HTMLElementBinding;
use dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::inheritance::Castable;
use dom::bindings::inheritance::{ElementTypeId, HTMLElementTypeId, NodeTypeId};
use dom::bindings::js::{JS, MutNullableHeap, Root, RootedReference};
use dom::bindings::reflector::Reflectable;
use dom::cssstyledeclaration::{CSSModificationAccess, CSSStyleDeclaration};
use dom::document::Document;
use dom::domstringmap::DOMStringMap;
use dom::element::{AttributeMutation, Element};
use dom::eventtarget::EventTarget;
use dom::htmlbodyelement::HTMLBodyElement;
use dom::htmlframesetelement::HTMLFrameSetElement;
use dom::htmlhtmlelement::HTMLHtmlElement;
use dom::htmlinputelement::HTMLInputElement;
use dom::htmllabelelement::HTMLLabelElement;
use dom::node::{Node, SEQUENTIALLY_FOCUSABLE};
use dom::node::{document_from_node, window_from_node};
use dom::nodelist::NodeList;
use dom::virtualmethods::VirtualMethods;
use msg::constellation_msg::FocusType;
use selectors::states::*;
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::default::Default;
use std::intrinsics;
use std::rc::Rc;
use string_cache::Atom;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLElement {
    element: Element,
    style_decl: MutNullableHeap<JS<CSSStyleDeclaration>>,
    dataset: MutNullableHeap<JS<DOMStringMap>>,
}

impl PartialEq for HTMLElement {
    fn eq(&self, other: &HTMLElement) -> bool {
        self as *const HTMLElement == &*other
    }
}

impl HTMLElement {
    pub fn new_inherited(tag_name: DOMString, prefix: Option<DOMString>,
                         document: &Document) -> HTMLElement {
        HTMLElement::new_inherited_with_state(ElementState::empty(), tag_name, prefix, document)
    }

    pub fn new_inherited_with_state(state: ElementState, tag_name: DOMString,
                                    prefix: Option<DOMString>, document: &Document)
                                    -> HTMLElement {
        HTMLElement {
            element:
                Element::new_inherited_with_state(state, tag_name, ns!(HTML), prefix, document),
            style_decl: Default::default(),
            dataset: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: &Document) -> Root<HTMLElement> {
        let element = HTMLElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLElementBinding::Wrap)
    }

    fn is_body_or_frameset(&self) -> bool {
        let eventtarget = self.upcast::<EventTarget>();
        eventtarget.is::<HTMLBodyElement>() || eventtarget.is::<HTMLFrameSetElement>()
    }

    fn update_sequentially_focusable_status(&self) {
        let element = self.upcast::<Element>();
        let node = self.upcast::<Node>();
        if element.has_attribute(&atom!("tabindex")) {
            node.set_flag(SEQUENTIALLY_FOCUSABLE, true);
        } else {
            match node.type_id() {
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) |
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) |
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLIFrameElement)) |
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement))
                    => node.set_flag(SEQUENTIALLY_FOCUSABLE, true),
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLinkElement)) |
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAnchorElement)) => {
                    if element.has_attribute(&atom!("href")) {
                        node.set_flag(SEQUENTIALLY_FOCUSABLE, true);
                    }
                },
                _ => {
                    if let Some(attr) = element.get_attribute(&ns!(""), &atom!("draggable")) {
                        let attr = attr.r();
                        let value = attr.value();
                        let is_true = match *value {
                            AttrValue::String(ref string) => string == "true",
                            _ => false,
                        };
                        node.set_flag(SEQUENTIALLY_FOCUSABLE, is_true);
                    } else {
                        node.set_flag(SEQUENTIALLY_FOCUSABLE, false);
                    }
                    //TODO set SEQUENTIALLY_FOCUSABLE flag if editing host
                    //TODO set SEQUENTIALLY_FOCUSABLE flag if "sorting interface th elements"
                },
            }
        }
    }
}

impl HTMLElementMethods for HTMLElement {
    // https://html.spec.whatwg.org/multipage/#the-style-attribute
    fn Style(&self) -> Root<CSSStyleDeclaration> {
        self.style_decl.or_init(|| {
            let global = window_from_node(self);
            CSSStyleDeclaration::new(global.r(), self.upcast::<Element>(), None, CSSModificationAccess::ReadWrite)
        })
    }

    // https://html.spec.whatwg.org/multipage/#attr-title
    make_getter!(Title);
    // https://html.spec.whatwg.org/multipage/#attr-title
    make_setter!(SetTitle, "title");

    // https://html.spec.whatwg.org/multipage/#attr-lang
    make_getter!(Lang);
    // https://html.spec.whatwg.org/multipage/#attr-lang
    make_setter!(SetLang, "lang");

    // https://html.spec.whatwg.org/multipage/#dom-hidden
    make_bool_getter!(Hidden);
    // https://html.spec.whatwg.org/multipage/#dom-hidden
    make_bool_setter!(SetHidden, "hidden");

    // https://html.spec.whatwg.org/multipage/#globaleventhandlers
    global_event_handlers!(NoOnload);

    // https://html.spec.whatwg.org/multipage/#dom-dataset
    fn Dataset(&self) -> Root<DOMStringMap> {
        self.dataset.or_init(|| DOMStringMap::new(self))
    }

    // https://html.spec.whatwg.org/multipage/#handler-onload
    fn GetOnload(&self) -> Option<Rc<EventHandlerNonNull>> {
        if self.is_body_or_frameset() {
            window_from_node(self).GetOnload()
        } else {
            self.upcast::<EventTarget>().get_event_handler_common("load")
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onload
    fn SetOnload(&self, listener: Option<Rc<EventHandlerNonNull>>) {
        if self.is_body_or_frameset() {
            window_from_node(self).SetOnload(listener)
        } else {
            self.upcast::<EventTarget>().set_event_handler_common("load", listener)
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-click
    fn Click(&self) {
        if let Some(i) = self.downcast::<HTMLInputElement>() {
            if i.Disabled() {
                return;
            }
        }
        // https://www.w3.org/Bugs/Public/show_bug.cgi?id=27430 ?
        self.upcast::<Element>()
            .as_maybe_activatable()
            .map(|a| a.synthetic_click_activation(false, false, false, false));
    }

    // https://html.spec.whatwg.org/multipage/#dom-focus
    fn Focus(&self) {
        // TODO: Mark the element as locked for focus and run the focusing steps.
        // https://html.spec.whatwg.org/multipage/#focusing-steps
        let document = document_from_node(self);
        document.begin_focus_transaction();
        document.request_focus(self.upcast());
        document.commit_focus_transaction(FocusType::Element);
    }

    // https://html.spec.whatwg.org/multipage/#dom-blur
    fn Blur(&self) {
        // TODO: Run the unfocusing steps.
        if !self.upcast::<Element>().get_focus_state() {
            return;
        }
        // https://html.spec.whatwg.org/multipage/#unfocusing-steps
        let document = document_from_node(self);
        document.begin_focus_transaction();
        // If `request_focus` is not called, focus will be set to None.
        document.commit_focus_transaction(FocusType::Element);
    }

    // https://drafts.csswg.org/cssom-view/#extensions-to-the-htmlelement-interface
    fn GetOffsetParent(&self) -> Option<Root<Element>> {
        if self.is::<HTMLBodyElement>() || self.is::<HTMLHtmlElement>() {
            return None;
        }

        let node = self.upcast::<Node>();
        let window = window_from_node(self);
        let (element, _) = window.offset_parent_query(node.to_trusted_node_address());

        element
    }

    // https://drafts.csswg.org/cssom-view/#extensions-to-the-htmlelement-interface
    fn OffsetTop(&self) -> i32 {
        if self.is::<HTMLBodyElement>() {
            return 0;
        }

        let node = self.upcast::<Node>();
        let window = window_from_node(self);
        let (_, rect) = window.offset_parent_query(node.to_trusted_node_address());

        rect.origin.y.to_nearest_px()
    }

    // https://drafts.csswg.org/cssom-view/#extensions-to-the-htmlelement-interface
    fn OffsetLeft(&self) -> i32 {
        if self.is::<HTMLBodyElement>() {
            return 0;
        }

        let node = self.upcast::<Node>();
        let window = window_from_node(self);
        let (_, rect) = window.offset_parent_query(node.to_trusted_node_address());

        rect.origin.x.to_nearest_px()
    }

    // https://drafts.csswg.org/cssom-view/#extensions-to-the-htmlelement-interface
    fn OffsetWidth(&self) -> i32 {
        let node = self.upcast::<Node>();
        let window = window_from_node(self);
        let (_, rect) = window.offset_parent_query(node.to_trusted_node_address());

        rect.size.width.to_nearest_px()
    }

    // https://drafts.csswg.org/cssom-view/#extensions-to-the-htmlelement-interface
    fn OffsetHeight(&self) -> i32 {
        let node = self.upcast::<Node>();
        let window = window_from_node(self);
        let (_, rect) = window.offset_parent_query(node.to_trusted_node_address());

        rect.size.height.to_nearest_px()
    }
}

// https://html.spec.whatwg.org/multipage/#attr-data-*

fn to_snake_case(name: DOMString) -> DOMString {
    let mut attr_name = "data-".to_owned();
    for ch in name.chars() {
        if ch.is_uppercase() {
            attr_name.push('\x2d');
            attr_name.extend(ch.to_lowercase());
        } else {
            attr_name.push(ch);
        }
    }
    DOMString(attr_name)
}


// https://html.spec.whatwg.org/multipage/#attr-data-*
// if this attribute is in snake case with a data- prefix,
// this function returns a name converted to camel case
// without the data prefix.

fn to_camel_case(name: &str) -> Option<DOMString> {
    if !name.starts_with("data-") {
        return None;
    }
    let name = &name[5..];
    let has_uppercase = name.chars().any(|curr_char| {
        curr_char.is_ascii() && curr_char.is_uppercase()
    });
    if has_uppercase {
        return None;
    }
    let mut result = "".to_owned();
    let mut name_chars = name.chars();
    while let Some(curr_char) = name_chars.next() {
        //check for hyphen followed by character
        if curr_char == '\x2d' {
            if let Some(next_char) = name_chars.next() {
                if next_char.is_ascii() && next_char.is_lowercase() {
                    result.push(next_char.to_ascii_uppercase());
                } else {
                    result.push(curr_char);
                    result.push(next_char);
                }
            } else {
                result.push(curr_char);
            }
        } else {
            result.push(curr_char);
        }
    }
    Some(DOMString(result))
}

impl HTMLElement {
    pub fn set_custom_attr(&self, name: DOMString, value: DOMString) -> ErrorResult {
        if name.chars()
               .skip_while(|&ch| ch != '\u{2d}')
               .nth(1).map_or(false, |ch| ch >= 'a' && ch <= 'z') {
            return Err(Error::Syntax);
        }
        self.upcast::<Element>().set_custom_attribute(to_snake_case(name), value)
    }

    pub fn get_custom_attr(&self, local_name: DOMString) -> Option<DOMString> {
        let local_name = Atom::from_slice(&to_snake_case(local_name));
        self.upcast::<Element>().get_attribute(&ns!(""), &local_name).map(|attr| {
            DOMString((**attr.value()).to_owned())
        })
    }

    pub fn delete_custom_attr(&self, local_name: DOMString) {
        let local_name = Atom::from_slice(&to_snake_case(local_name));
        self.upcast::<Element>().remove_attribute(&ns!(""), &local_name);
    }

    // https://html.spec.whatwg.org/multipage/#category-label
    pub fn is_labelable_element(&self) -> bool {
        // Note: HTMLKeygenElement is omitted because Servo doesn't currently implement it
        match self.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(type_id)) =>
                match type_id {
                    HTMLElementTypeId::HTMLInputElement =>
                        self.downcast::<HTMLInputElement>().unwrap().type_() != atom!("hidden"),
                    HTMLElementTypeId::HTMLButtonElement |
                        HTMLElementTypeId::HTMLMeterElement |
                        HTMLElementTypeId::HTMLOutputElement |
                        HTMLElementTypeId::HTMLProgressElement |
                        HTMLElementTypeId::HTMLSelectElement |
                        HTMLElementTypeId::HTMLTextAreaElement => true,
                    _ => false,
                },
            _ => false,
        }
    }

    pub fn supported_prop_names_custom_attr(&self) -> Vec<DOMString> {
        let element = self.upcast::<Element>();
        element.attrs().iter().filter_map(|attr| {
            let raw_name = attr.local_name();
            to_camel_case(&raw_name)
        }).collect()
    }

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    pub fn labels(&self) -> Root<NodeList> {
        debug_assert!(self.is_labelable_element());

        let element = self.upcast::<Element>();
        let window = window_from_node(element);

        // Traverse ancestors for implicitly associated <label> elements
        // https://html.spec.whatwg.org/multipage/#the-label-element:attr-label-for-4
        let ancestors =
            self.upcast::<Node>()
                .ancestors()
                .filter_map(Root::downcast::<HTMLElement>)
                // If we reach a labelable element, we have a guarantee no ancestors above it
                // will be a label for this HTMLElement
                .take_while(|elem| !elem.is_labelable_element())
                .filter_map(Root::downcast::<HTMLLabelElement>)
                .filter(|elem| !elem.upcast::<Element>().has_attribute(&atom!("for")))
                .filter(|elem| elem.first_labelable_descendant().r() == Some(self))
                .map(Root::upcast::<Node>);

        let id = element.Id();
        let id = match &id as &str {
            "" => return NodeList::new_simple_list(window.r(), ancestors),
            id => id,
        };

        // Traverse entire tree for <label> elements with `for` attribute matching `id`
        let root_element = element.get_root_element();
        let root_node = root_element.upcast::<Node>();
        let children = root_node.traverse_preorder()
                                .filter_map(Root::downcast::<Element>)
                                .filter(|elem| elem.is::<HTMLLabelElement>())
                                .filter(|elem| elem.get_string_attribute(&atom!("for")) == id)
                                .map(Root::upcast::<Node>);

        NodeList::new_simple_list(window.r(), children.chain(ancestors))
    }
}

impl VirtualMethods for HTMLElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<Element>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match (attr.local_name(), mutation) {
            (name, AttributeMutation::Set(_)) if name.starts_with("on") => {
                let window = window_from_node(self);
                let (cx, url, reflector) = (window.get_cx(),
                                            window.get_url(),
                                            window.reflector().get_jsobject());
                let evtarget = self.upcast::<EventTarget>();
                evtarget.set_event_handler_uncompiled(cx, url, reflector,
                                                      &name[2..],
                                                      DOMString((**attr.value()).to_owned()));
            },
            _ => {}
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }
        self.update_sequentially_focusable_status();
    }
}

impl PartialEq for HTMLElementTypeId {
    #[inline]
    #[allow(unsafe_code)]
    fn eq(&self, other: &HTMLElementTypeId) -> bool {
        match (*self, *other) {
            (HTMLElementTypeId::HTMLMediaElement(this_type),
             HTMLElementTypeId::HTMLMediaElement(other_type)) => {
                this_type == other_type
            }
            (HTMLElementTypeId::HTMLTableCellElement(this_type),
             HTMLElementTypeId::HTMLTableCellElement(other_type)) => {
                this_type == other_type
            }
            (_, _) => {
                unsafe {
                    intrinsics::discriminant_value(self) == intrinsics::discriminant_value(other)
                }
            }
        }
    }
}
