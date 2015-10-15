/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::attr::AttrValue;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::HTMLElementBinding;
use dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, ElementTypeId};
use dom::bindings::codegen::InheritTypes::{EventTargetCast, HTMLBodyElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLElementTypeId, HTMLFrameSetElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLHtmlElementDerived, HTMLInputElementCast};
use dom::bindings::codegen::InheritTypes::{NodeCast, NodeTypeId};
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::utils::Reflectable;
use dom::cssstyledeclaration::{CSSModificationAccess, CSSStyleDeclaration};
use dom::document::Document;
use dom::domstringmap::DOMStringMap;
use dom::element::{AttributeMutation, Element, EventState};
use dom::htmlinputelement::HTMLInputElement;
use dom::node::{Node, SEQUENTIALLY_FOCUSABLE};
use dom::node::{document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use msg::constellation_msg::FocusType;
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
        HTMLElement::new_inherited_with_state(EventState::empty(), tag_name, prefix, document)
    }

    pub fn new_inherited_with_state(state: EventState, tag_name: DOMString,
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
        let eventtarget = EventTargetCast::from_ref(self);
        eventtarget.is_htmlbodyelement() || eventtarget.is_htmlframesetelement()
    }

    fn update_sequentially_focusable_status(&self) {
        let element = ElementCast::from_ref(self);
        let node = NodeCast::from_ref(self);
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
            CSSStyleDeclaration::new(global.r(), ElementCast::from_ref(self), None, CSSModificationAccess::ReadWrite)
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
            let win = window_from_node(self);
            win.r().GetOnload()
        } else {
            let target = EventTargetCast::from_ref(self);
            target.get_event_handler_common("load")
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onload
    fn SetOnload(&self, listener: Option<Rc<EventHandlerNonNull>>) {
        if self.is_body_or_frameset() {
            let win = window_from_node(self);
            win.r().SetOnload(listener)
        } else {
            let target = EventTargetCast::from_ref(self);
            target.set_event_handler_common("load", listener)
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-click
    fn Click(&self) {
        let maybe_input: Option<&HTMLInputElement> = HTMLInputElementCast::to_ref(self);
        if let Some(i) = maybe_input {
            if i.Disabled() {
                return;
            }
        }
        let element = ElementCast::from_ref(self);
        // https://www.w3.org/Bugs/Public/show_bug.cgi?id=27430 ?
        element.as_maybe_activatable().map(|a| a.synthetic_click_activation(false, false, false, false));
    }

    // https://html.spec.whatwg.org/multipage/#dom-focus
    fn Focus(&self) {
        // TODO: Mark the element as locked for focus and run the focusing steps.
        // https://html.spec.whatwg.org/multipage/#focusing-steps
        let element = ElementCast::from_ref(self);
        let document = document_from_node(self);
        let document = document.r();
        document.begin_focus_transaction();
        document.request_focus(element);
        document.commit_focus_transaction(FocusType::Element);
    }

    // https://html.spec.whatwg.org/multipage/#dom-blur
    fn Blur(&self) {
        // TODO: Run the unfocusing steps.
        let el = ElementCast::from_ref(self);
        if !el.get_focus_state() {
            return;
        }
        // https://html.spec.whatwg.org/multipage/#unfocusing-steps
        let document = document_from_node(self);
        document.r().begin_focus_transaction();
        // If `request_focus` is not called, focus will be set to None.
        document.r().commit_focus_transaction(FocusType::Element);
    }

    // https://drafts.csswg.org/cssom-view/#extensions-to-the-htmlelement-interface
    fn GetOffsetParent(&self) -> Option<Root<Element>> {
        if self.is_htmlbodyelement() || self.is_htmlhtmlelement() {
            return None;
        }

        let node = NodeCast::from_ref(self);
        let window = window_from_node(self);
        let (element, _) = window.offset_parent_query(node.to_trusted_node_address());

        element
    }

    // https://drafts.csswg.org/cssom-view/#extensions-to-the-htmlelement-interface
    fn OffsetTop(&self) -> i32 {
        if self.is_htmlbodyelement() {
            return 0;
        }

        let node = NodeCast::from_ref(self);
        let window = window_from_node(self);
        let (_, rect) = window.offset_parent_query(node.to_trusted_node_address());

        rect.origin.y.to_nearest_px()
    }

    // https://drafts.csswg.org/cssom-view/#extensions-to-the-htmlelement-interface
    fn OffsetLeft(&self) -> i32 {
        if self.is_htmlbodyelement() {
            return 0;
        }

        let node = NodeCast::from_ref(self);
        let window = window_from_node(self);
        let (_, rect) = window.offset_parent_query(node.to_trusted_node_address());

        rect.origin.x.to_nearest_px()
    }

    // https://drafts.csswg.org/cssom-view/#extensions-to-the-htmlelement-interface
    fn OffsetWidth(&self) -> i32 {
        let node = NodeCast::from_ref(self);
        let window = window_from_node(self);
        let (_, rect) = window.offset_parent_query(node.to_trusted_node_address());

        rect.size.width.to_nearest_px()
    }

    // https://drafts.csswg.org/cssom-view/#extensions-to-the-htmlelement-interface
    fn OffsetHeight(&self) -> i32 {
        let node = NodeCast::from_ref(self);
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
    attr_name
}

impl HTMLElement {
    pub fn set_custom_attr(&self, name: DOMString, value: DOMString) -> ErrorResult {
        if name.chars()
               .skip_while(|&ch| ch != '\u{2d}')
               .nth(1).map_or(false, |ch| ch >= 'a' && ch <= 'z') {
            return Err(Error::Syntax);
        }
        let element = ElementCast::from_ref(self);
        element.set_custom_attribute(to_snake_case(name), value)
    }

    pub fn get_custom_attr(&self, local_name: DOMString) -> Option<DOMString> {
        let element = ElementCast::from_ref(self);
        let local_name = Atom::from_slice(&to_snake_case(local_name));
        element.get_attribute(&ns!(""), &local_name).map(|attr| {
            (**attr.r().value()).to_owned()
        })
    }

    pub fn delete_custom_attr(&self, local_name: DOMString) {
        let element = ElementCast::from_ref(self);
        let local_name = Atom::from_slice(&to_snake_case(local_name));
        element.remove_attribute(&ns!(""), &local_name);
    }
}

impl VirtualMethods for HTMLElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let element: &Element = ElementCast::from_ref(self);
        Some(element as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match (attr.local_name(), mutation) {
            (name, AttributeMutation::Set(_)) if name.starts_with("on") => {
                let window = window_from_node(self);
                let (cx, url, reflector) = (window.r().get_cx(),
                                            window.r().get_url(),
                                            window.r().reflector().get_jsobject());
                let evtarget = EventTargetCast::from_ref(self);
                evtarget.set_event_handler_uncompiled(cx, url, reflector,
                                                      &name[2..],
                                                      (**attr.value()).to_owned());
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
