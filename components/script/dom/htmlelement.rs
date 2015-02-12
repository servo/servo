/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::attr::AttrHelpers;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::HTMLElementBinding;
use dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLFrameSetElementDerived};
use dom::bindings::codegen::InheritTypes::{EventTargetCast, HTMLInputElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLElementDerived, HTMLBodyElementDerived};
use dom::bindings::js::{JSRef, Temporary, MutNullableJS};
use dom::bindings::error::ErrorResult;
use dom::bindings::error::Error::Syntax;
use dom::bindings::utils::Reflectable;
use dom::cssstyledeclaration::{CSSStyleDeclaration, CSSModificationAccess};
use dom::document::Document;
use dom::domstringmap::DOMStringMap;
use dom::element::{Element, ElementTypeId, ActivationElementHelpers, AttributeHandlers};
use dom::eventtarget::{EventTarget, EventTargetHelpers, EventTargetTypeId};
use dom::htmlinputelement::HTMLInputElement;
use dom::htmlmediaelement::HTMLMediaElementTypeId;
use dom::htmltablecellelement::HTMLTableCellElementTypeId;
use dom::node::{Node, NodeTypeId, window_from_node};
use dom::virtualmethods::VirtualMethods;

use util::str::DOMString;

use string_cache::Atom;

use std::borrow::ToOwned;
use std::default::Default;

#[dom_struct]
pub struct HTMLElement {
    element: Element,
    style_decl: MutNullableJS<CSSStyleDeclaration>,
    dataset: MutNullableJS<DOMStringMap>,
}

impl HTMLElementDerived for EventTarget {
    fn is_htmlelement(&self) -> bool {
        match *self.type_id() {
            EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(_))) => true,
            _ => false
        }
    }
}

impl HTMLElement {
    pub fn new_inherited(type_id: HTMLElementTypeId, tag_name: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLElement {
        HTMLElement {
            element: Element::new_inherited(ElementTypeId::HTMLElement(type_id), tag_name, ns!(HTML), prefix, document),
            style_decl: Default::default(),
            dataset: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLElement> {
        let element = HTMLElement::new_inherited(HTMLElementTypeId::HTMLElement, localName, prefix, document);
        Node::reflect_node(box element, document, HTMLElementBinding::Wrap)
    }
}

trait PrivateHTMLElementHelpers {
    fn is_body_or_frameset(self) -> bool;
}

impl<'a> PrivateHTMLElementHelpers for JSRef<'a, HTMLElement> {
    fn is_body_or_frameset(self) -> bool {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        eventtarget.is_htmlbodyelement() || eventtarget.is_htmlframesetelement()
    }
}

impl<'a> HTMLElementMethods for JSRef<'a, HTMLElement> {
    fn Style(self) -> Temporary<CSSStyleDeclaration> {
        self.style_decl.or_init(|| {
            let global = window_from_node(self).root();
            CSSStyleDeclaration::new(global.r(), self, CSSModificationAccess::ReadWrite)
        })
    }

    make_getter!(Title);
    make_setter!(SetTitle, "title");

    make_getter!(Lang);
    make_setter!(SetLang, "lang");

    // http://html.spec.whatwg.org/multipage/#dom-hidden
    make_bool_getter!(Hidden);
    make_bool_setter!(SetHidden, "hidden");

    global_event_handlers!(NoOnload);

    // https://html.spec.whatwg.org/multipage/dom.html#dom-dataset
    fn Dataset(self) -> Temporary<DOMStringMap> {
        self.dataset.or_init(|| DOMStringMap::new(self))
    }

    fn GetOnload(self) -> Option<EventHandlerNonNull> {
        if self.is_body_or_frameset() {
            let win = window_from_node(self).root();
            win.r().GetOnload()
        } else {
            let target: JSRef<EventTarget> = EventTargetCast::from_ref(self);
            target.get_event_handler_common("load")
        }
    }

    fn SetOnload(self, listener: Option<EventHandlerNonNull>) {
        if self.is_body_or_frameset() {
            let win = window_from_node(self).root();
            win.r().SetOnload(listener)
        } else {
            let target: JSRef<EventTarget> = EventTargetCast::from_ref(self);
            target.set_event_handler_common("load", listener)
        }
    }

    // https://html.spec.whatwg.org/multipage/interaction.html#dom-click
    fn Click(self) {
        let maybe_input: Option<JSRef<HTMLInputElement>> = HTMLInputElementCast::to_ref(self);
        match maybe_input {
            Some(i) if i.Disabled() => return,
            _ => ()
        }
        let element: JSRef<Element> = ElementCast::from_ref(self);
        // https://www.w3.org/Bugs/Public/show_bug.cgi?id=27430 ?
        element.as_maybe_activatable().map(|a| a.synthetic_click_activation(false, false, false, false));
    }
}

// https://html.spec.whatwg.org/#attr-data-*
pub trait HTMLElementCustomAttributeHelpers {
    fn set_custom_attr(self, name: DOMString, value: DOMString) -> ErrorResult;
    fn get_custom_attr(self, name: DOMString) -> Option<DOMString>;
    fn delete_custom_attr(self, name: DOMString);
}

fn to_snake_case(name: DOMString) -> DOMString {
    let mut attr_name = "data-".to_owned();
    for ch in name.as_slice().chars() {
        if ch.is_uppercase() {
            attr_name.push('\x2d');
            attr_name.push(ch.to_lowercase());
        } else {
            attr_name.push(ch);
        }
    }
    attr_name
}

impl<'a> HTMLElementCustomAttributeHelpers for JSRef<'a, HTMLElement> {
    fn set_custom_attr(self, name: DOMString, value: DOMString) -> ErrorResult {
        if name.as_slice().chars()
               .skip_while(|&ch| ch != '\u{2d}')
               .nth(1).map_or(false, |ch| ch as u8 - b'a' < 26) {
            return Err(Syntax);
        }
        let element: JSRef<Element> = ElementCast::from_ref(self);
        element.set_custom_attribute(to_snake_case(name), value)
    }

    fn get_custom_attr(self, name: DOMString) -> Option<DOMString> {
        let element: JSRef<Element> = ElementCast::from_ref(self);
        element.get_attribute(ns!(""), &Atom::from_slice(to_snake_case(name).as_slice())).map(|attr| {
            let attr = attr.root();
            attr.r().value().as_slice().to_owned()
        })
    }

    fn delete_custom_attr(self, name: DOMString) {
        let element: JSRef<Element> = ElementCast::from_ref(self);
        element.remove_attribute(ns!(""), to_snake_case(name).as_slice())
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLElement> {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let element: &JSRef<Element> = ElementCast::from_borrowed_ref(self);
        Some(element as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(attr),
            _ => ()
        }

        let name = attr.local_name().as_slice();
        if name.starts_with("on") {
            let window = window_from_node(*self).root();
            let (cx, url, reflector) = (window.r().get_cx(),
                                        window.r().get_url(),
                                        window.r().reflector().get_jsobject());
            let evtarget: JSRef<EventTarget> = EventTargetCast::from_ref(*self);
            evtarget.set_event_handler_uncompiled(cx, url, reflector,
                                                  &name[2..],
                                                  attr.value().as_slice().to_owned());
        }
    }
}

#[derive(Copy, PartialEq, Debug)]
#[jstraceable]
pub enum HTMLElementTypeId {
    HTMLElement,

    HTMLAnchorElement,
    HTMLAppletElement,
    HTMLAreaElement,
    HTMLBaseElement,
    HTMLBRElement,
    HTMLBodyElement,
    HTMLButtonElement,
    HTMLCanvasElement,
    HTMLDataElement,
    HTMLDataListElement,
    HTMLDirectoryElement,
    HTMLDListElement,
    HTMLDivElement,
    HTMLEmbedElement,
    HTMLFieldSetElement,
    HTMLFontElement,
    HTMLFormElement,
    HTMLFrameElement,
    HTMLFrameSetElement,
    HTMLHRElement,
    HTMLHeadElement,
    HTMLHeadingElement,
    HTMLHtmlElement,
    HTMLIFrameElement,
    HTMLImageElement,
    HTMLInputElement,
    HTMLLabelElement,
    HTMLLegendElement,
    HTMLLinkElement,
    HTMLLIElement,
    HTMLMapElement,
    HTMLMediaElement(HTMLMediaElementTypeId),
    HTMLMetaElement,
    HTMLMeterElement,
    HTMLModElement,
    HTMLObjectElement,
    HTMLOListElement,
    HTMLOptGroupElement,
    HTMLOptionElement,
    HTMLOutputElement,
    HTMLParagraphElement,
    HTMLParamElement,
    HTMLPreElement,
    HTMLProgressElement,
    HTMLQuoteElement,
    HTMLScriptElement,
    HTMLSelectElement,
    HTMLSourceElement,
    HTMLSpanElement,
    HTMLStyleElement,
    HTMLTableElement,
    HTMLTableCaptionElement,
    HTMLTableCellElement(HTMLTableCellElementTypeId),
    HTMLTableColElement,
    HTMLTableRowElement,
    HTMLTableSectionElement,
    HTMLTemplateElement,
    HTMLTextAreaElement,
    HTMLTimeElement,
    HTMLTitleElement,
    HTMLTrackElement,
    HTMLUListElement,
    HTMLUnknownElement,
}

