/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::activation::{ActivationSource, synthetic_click_activation};
use dom::attr::Attr;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::EventHandlerBinding::OnErrorEventHandlerNonNull;
use dom::bindings::codegen::Bindings::HTMLElementBinding;
use dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::inheritance::{ElementTypeId, HTMLElementTypeId, NodeTypeId};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableJS, Root, RootedReference};
use dom::bindings::str::DOMString;
use dom::cssstyledeclaration::{CSSModificationAccess, CSSStyleDeclaration, CSSStyleOwner};
use dom::document::{Document, FocusType};
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
use dom_struct::dom_struct;
use html5ever::LocalName;
use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::default::Default;
use std::rc::Rc;
use style::attr::AttrValue;
use style::element_state::*;

#[dom_struct]
pub struct HTMLElement {
    element: Element,
    style_decl: MutNullableJS<CSSStyleDeclaration>,
    dataset: MutNullableJS<DOMStringMap>,
}

impl HTMLElement {
    pub fn new_inherited(tag_name: LocalName, prefix: Option<DOMString>,
                         document: &Document) -> HTMLElement {
        HTMLElement::new_inherited_with_state(ElementState::empty(), tag_name, prefix, document)
    }

    pub fn new_inherited_with_state(state: ElementState, tag_name: LocalName,
                                    prefix: Option<DOMString>, document: &Document)
                                    -> HTMLElement {
        HTMLElement {
            element:
                Element::new_inherited_with_state(state, tag_name, ns!(html), prefix, document),
            style_decl: Default::default(),
            dataset: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName, prefix: Option<DOMString>, document: &Document) -> Root<HTMLElement> {
        Node::reflect_node(box HTMLElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLElementBinding::Wrap)
    }

    fn is_body_or_frameset(&self) -> bool {
        let eventtarget = self.upcast::<EventTarget>();
        eventtarget.is::<HTMLBodyElement>() || eventtarget.is::<HTMLFrameSetElement>()
    }

    fn update_sequentially_focusable_status(&self) {
        let element = self.upcast::<Element>();
        let node = self.upcast::<Node>();
        if element.has_attribute(&local_name!("tabindex")) {
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
                    if element.has_attribute(&local_name!("href")) {
                        node.set_flag(SEQUENTIALLY_FOCUSABLE, true);
                    }
                },
                _ => {
                    if let Some(attr) = element.get_attribute(&ns!(), &local_name!("draggable")) {
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
            CSSStyleDeclaration::new(&global,
                                     CSSStyleOwner::Element(JS::from_ref(self.upcast())),
                                     None,
                                     CSSModificationAccess::ReadWrite)
        })
    }

    // https://html.spec.whatwg.org/multipage/#attr-title
    make_getter!(Title, "title");
    // https://html.spec.whatwg.org/multipage/#attr-title
    make_setter!(SetTitle, "title");

    // https://html.spec.whatwg.org/multipage/#attr-lang
    make_getter!(Lang, "lang");
    // https://html.spec.whatwg.org/multipage/#attr-lang
    make_setter!(SetLang, "lang");

    // https://html.spec.whatwg.org/multipage/#dom-hidden
    make_bool_getter!(Hidden, "hidden");
    // https://html.spec.whatwg.org/multipage/#dom-hidden
    make_bool_setter!(SetHidden, "hidden");

    // https://html.spec.whatwg.org/multipage/#globaleventhandlers
    global_event_handlers!(NoOnload);

    // https://html.spec.whatwg.org/multipage/#documentandelementeventhandlers
    document_and_element_event_handlers!();

    // https://html.spec.whatwg.org/multipage/#dom-dataset
    fn Dataset(&self) -> Root<DOMStringMap> {
        self.dataset.or_init(|| DOMStringMap::new(self))
    }

    // https://html.spec.whatwg.org/multipage/#handler-onload
    fn GetOnload(&self) -> Option<Rc<EventHandlerNonNull>> {
        if self.is_body_or_frameset() {
            let document = document_from_node(self);
            if document.has_browsing_context() {
                document.window().GetOnload()
            } else {
                None
            }
        } else {
            self.upcast::<EventTarget>().get_event_handler_common("load")
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onload
    fn SetOnload(&self, listener: Option<Rc<EventHandlerNonNull>>) {
        if self.is_body_or_frameset() {
            let document = document_from_node(self);
            if document.has_browsing_context() {
                document.window().SetOnload(listener)
            }
        } else {
            self.upcast::<EventTarget>().set_event_handler_common("load", listener)
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onresize
    fn GetOnresize(&self) -> Option<Rc<EventHandlerNonNull>> {
        if self.is_body_or_frameset() {
            let document = document_from_node(self);
            if document.has_browsing_context() {
                document.window().GetOnload()
            } else {
                None
            }
        } else {
            self.upcast::<EventTarget>().get_event_handler_common("resize")
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onresize
    fn SetOnresize(&self, listener: Option<Rc<EventHandlerNonNull>>) {
        if self.is_body_or_frameset() {
            let document = document_from_node(self);
            if document.has_browsing_context() {
                document.window().SetOnresize(listener);
            }
        } else {
            self.upcast::<EventTarget>().set_event_handler_common("resize", listener)
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onblur
    fn GetOnblur(&self) -> Option<Rc<EventHandlerNonNull>> {
        if self.is_body_or_frameset() {
            let document = document_from_node(self);
            if document.has_browsing_context() {
                document.window().GetOnblur()
            } else {
                None
            }
        } else {
            self.upcast::<EventTarget>().get_event_handler_common("blur")
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onblur
    fn SetOnblur(&self, listener: Option<Rc<EventHandlerNonNull>>) {
        if self.is_body_or_frameset() {
            let document = document_from_node(self);
            if document.has_browsing_context() {
                document.window().SetOnblur(listener)
            }
        } else {
            self.upcast::<EventTarget>().set_event_handler_common("blur", listener)
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onfocus
    fn GetOnfocus(&self) -> Option<Rc<EventHandlerNonNull>> {
        if self.is_body_or_frameset() {
            let document = document_from_node(self);
            if document.has_browsing_context() {
                document.window().GetOnfocus()
            } else {
                None
            }
        } else {
            self.upcast::<EventTarget>().get_event_handler_common("focus")
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onfocus
    fn SetOnfocus(&self, listener: Option<Rc<EventHandlerNonNull>>) {
        if self.is_body_or_frameset() {
            let document = document_from_node(self);
            if document.has_browsing_context() {
                document.window().SetOnfocus(listener)
            }
        } else {
            self.upcast::<EventTarget>().set_event_handler_common("focus", listener)
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onscroll
    fn GetOnscroll(&self) -> Option<Rc<EventHandlerNonNull>> {
        if self.is_body_or_frameset() {
            let document = document_from_node(self);
            if document.has_browsing_context() {
                document.window().GetOnscroll()
            } else {
                None
            }
        } else {
            self.upcast::<EventTarget>().get_event_handler_common("scroll")
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-onscroll
    fn SetOnscroll(&self, listener: Option<Rc<EventHandlerNonNull>>) {
        if self.is_body_or_frameset() {
            let document = document_from_node(self);
            if document.has_browsing_context() {
                document.window().SetOnscroll(listener)
            }
        } else {
            self.upcast::<EventTarget>().set_event_handler_common("scroll", listener)
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-click
    fn Click(&self) {
        if !self.upcast::<Element>().disabled_state() {
            synthetic_click_activation(self.upcast::<Element>(),
                                       false,
                                       false,
                                       false,
                                       false,
                                       ActivationSource::FromClick)
        }
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
        if !self.upcast::<Element>().focus_state() {
            return;
        }
        // https://html.spec.whatwg.org/multipage/#unfocusing-steps
        let document = document_from_node(self);
        document.begin_focus_transaction();
        // If `request_focus` is not called, focus will be set to None.
        document.commit_focus_transaction(FocusType::Element);
    }

    // https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsetparent
    fn GetOffsetParent(&self) -> Option<Root<Element>> {
        if self.is::<HTMLBodyElement>() || self.is::<HTMLHtmlElement>() {
            return None;
        }

        let node = self.upcast::<Node>();
        let window = window_from_node(self);
        let (element, _) = window.offset_parent_query(node.to_trusted_node_address());

        element
    }

    // https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsettop
    fn OffsetTop(&self) -> i32 {
        if self.is::<HTMLBodyElement>() {
            return 0;
        }

        let node = self.upcast::<Node>();
        let window = window_from_node(self);
        let (_, rect) = window.offset_parent_query(node.to_trusted_node_address());

        rect.origin.y.to_nearest_px()
    }

    // https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsetleft
    fn OffsetLeft(&self) -> i32 {
        if self.is::<HTMLBodyElement>() {
            return 0;
        }

        let node = self.upcast::<Node>();
        let window = window_from_node(self);
        let (_, rect) = window.offset_parent_query(node.to_trusted_node_address());

        rect.origin.x.to_nearest_px()
    }

    // https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsetwidth
    fn OffsetWidth(&self) -> i32 {
        let node = self.upcast::<Node>();
        let window = window_from_node(self);
        let (_, rect) = window.offset_parent_query(node.to_trusted_node_address());

        rect.size.width.to_nearest_px()
    }

    // https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsetheight
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
    DOMString::from(attr_name)
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
    Some(DOMString::from(result))
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
        // FIXME(ajeffrey): Convert directly from DOMString to LocalName
        let local_name = LocalName::from(to_snake_case(local_name));
        self.upcast::<Element>().get_attribute(&ns!(), &local_name).map(|attr| {
            DOMString::from(&**attr.value()) // FIXME(ajeffrey): Convert directly from AttrValue to DOMString
        })
    }

    pub fn delete_custom_attr(&self, local_name: DOMString) {
        // FIXME(ajeffrey): Convert directly from DOMString to LocalName
        let local_name = LocalName::from(to_snake_case(local_name));
        self.upcast::<Element>().remove_attribute(&ns!(), &local_name);
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

    // https://html.spec.whatwg.org/multipage/#category-listed
    pub fn is_listed_element(&self) -> bool {
        // Servo does not implement HTMLKeygenElement
        // https://github.com/servo/servo/issues/2782
        if self.upcast::<Element>().local_name() == &local_name!("keygen") {
            return true;
        }

        match self.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(type_id)) =>
                match type_id {
                    HTMLElementTypeId::HTMLButtonElement |
                        HTMLElementTypeId::HTMLFieldSetElement |
                        HTMLElementTypeId::HTMLInputElement |
                        HTMLElementTypeId::HTMLObjectElement |
                        HTMLElementTypeId::HTMLOutputElement |
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
                .filter(|elem| !elem.upcast::<Element>().has_attribute(&local_name!("for")))
                .filter(|elem| elem.first_labelable_descendant().r() == Some(self))
                .map(Root::upcast::<Node>);

        let id = element.Id();
        let id = match &id as &str {
            "" => return NodeList::new_simple_list(&window, ancestors),
            id => id,
        };

        // Traverse entire tree for <label> elements with `for` attribute matching `id`
        let root_element = element.root_element();
        let root_node = root_element.upcast::<Node>();
        let children = root_node.traverse_preorder()
                                .filter_map(Root::downcast::<Element>)
                                .filter(|elem| elem.is::<HTMLLabelElement>())
                                .filter(|elem| elem.get_string_attribute(&local_name!("for")) == id)
                                .map(Root::upcast::<Node>);

        NodeList::new_simple_list(&window, children.chain(ancestors))
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
                let evtarget = self.upcast::<EventTarget>();
                let source_line = 1; //TODO(#9604) get current JS execution line
                evtarget.set_event_handler_uncompiled(window_from_node(self).get_url(),
                                                      source_line,
                                                      &name[2..],
                                                      // FIXME(ajeffrey): Convert directly from AttrValue to DOMString
                                                      DOMString::from(&**attr.value()));
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
