/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::AttrValue;
use dom::bindings::codegen::Bindings::HTMLImageElementBinding;
use dom::bindings::codegen::InheritTypes::{NodeCast, ElementCast, HTMLElementCast, HTMLImageElementDerived};
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::trace::Untraceable;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::{Element, HTMLImageElementTypeId};
use dom::element::AttributeHandlers;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId, NodeHelpers, window_from_node};
use dom::virtualmethods::VirtualMethods;
use servo_util::geometry::to_px;
use servo_net::image_cache_task;
use servo_util::str::DOMString;
use std::cell::RefCell;
use url::{Url, UrlParser};

#[deriving(Encodable)]
pub struct HTMLImageElement {
    pub htmlelement: HTMLElement,
    image: Untraceable<RefCell<Option<Url>>>,
}

impl HTMLImageElementDerived for EventTarget {
    fn is_htmlimageelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLImageElementTypeId))
    }
}

trait PrivateHTMLImageElementHelpers {
    fn update_image(&self, value: Option<(DOMString, &Url)>);
}

impl<'a> PrivateHTMLImageElementHelpers for JSRef<'a, HTMLImageElement> {
    /// Makes the local `image` member match the status of the `src` attribute and starts
    /// prefetching the image. This method must be called after `src` is changed.
    fn update_image(&self, value: Option<(DOMString, &Url)>) {
        let node: &JSRef<Node> = NodeCast::from_ref(self);
        let document = node.owner_doc().root();
        let window = document.deref().window.root();
        let image_cache = &window.image_cache_task;
        match value {
            None => {
                *self.image.deref().borrow_mut() = None;
            }
            Some((src, base_url)) => {
                let img_url = UrlParser::new().base_url(base_url).parse(src.as_slice());
                // FIXME: handle URL parse errors more gracefully.
                let img_url = img_url.unwrap();
                *self.image.deref().borrow_mut() = Some(img_url.clone());

                // inform the image cache to load this, but don't store a
                // handle.
                //
                // TODO (Issue #84): don't prefetch if we are within a
                // <noscript> tag.
                image_cache.send(image_cache_task::Prefetch(img_url));
            }
        }
    }
}

impl HTMLImageElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLImageElement {
        HTMLImageElement {
            htmlelement: HTMLElement::new_inherited(HTMLImageElementTypeId, localName, document),
            image: Untraceable::new(RefCell::new(None)),
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLImageElement> {
        let element = HTMLImageElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLImageElementBinding::Wrap)
    }
}

pub trait LayoutHTMLImageElementHelpers {
    unsafe fn image(&self) -> Option<Url>;
}

impl LayoutHTMLImageElementHelpers for JS<HTMLImageElement> {
    unsafe fn image(&self) -> Option<Url> {
        (*self.unsafe_get()).image.borrow().clone()
    }
}

pub trait HTMLImageElementMethods {
    fn Alt(&self) -> DOMString;
    fn SetAlt(&self, alt: DOMString);
    fn Src(&self) -> DOMString;
    fn SetSrc(&self, src: DOMString);
    fn UseMap(&self) -> DOMString;
    fn SetUseMap(&self, use_map: DOMString);
    fn IsMap(&self) -> bool;
    fn SetIsMap(&self, is_map: bool);
    fn Width(&self) -> u32;
    fn SetWidth(&self, width: u32);
    fn Height(&self) -> u32;
    fn SetHeight(&self, height: u32);
    fn Name(&self) -> DOMString;
    fn SetName(&self, name: DOMString);
    fn Align(&self) -> DOMString;
    fn SetAlign(&self, align: DOMString);
    fn Hspace(&self) -> u32;
    fn SetHspace(&self, hspace: u32);
    fn Vspace(&self) -> u32;
    fn SetVspace(&self, vspace: u32);
    fn LongDesc(&self) -> DOMString;
    fn SetLongDesc(&self, longdesc: DOMString);
    fn Border(&self) -> DOMString;
    fn SetBorder(&self, border: DOMString);
}

impl<'a> HTMLImageElementMethods for JSRef<'a, HTMLImageElement> {
    fn Alt(&self) -> DOMString {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_string_attribute("alt")
    }

    fn SetAlt(&self, alt: DOMString) {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.set_string_attribute("alt", alt)
    }

    fn Src(&self) -> DOMString {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_string_attribute("src")
    }

    fn SetSrc(&self, src: DOMString) {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.set_url_attribute("src", src)
    }

    fn UseMap(&self) -> DOMString {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_string_attribute("useMap")
    }

    fn SetUseMap(&self, use_map: DOMString) {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.set_string_attribute("useMap", use_map)
    }

    fn IsMap(&self) -> bool {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        from_str::<bool>(element.get_string_attribute("hspace").as_slice()).unwrap()
    }

    fn SetIsMap(&self, is_map: bool) {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.set_string_attribute("isMap", is_map.to_str())
    }

    fn Width(&self) -> u32 {
        let node: &JSRef<Node> = NodeCast::from_ref(self);
        let rect = node.get_bounding_content_box();
        to_px(rect.size.width) as u32
    }

    fn SetWidth(&self, width: u32) {
        let elem: &JSRef<Element> = ElementCast::from_ref(self);
        elem.set_uint_attribute("width", width)
    }

    fn Height(&self) -> u32 {
        let node: &JSRef<Node> = NodeCast::from_ref(self);
        let rect = node.get_bounding_content_box();
        to_px(rect.size.height) as u32
    }

    fn SetHeight(&self, height: u32) {
        let elem: &JSRef<Element> = ElementCast::from_ref(self);
        elem.set_uint_attribute("height", height)
    }

    fn Name(&self) -> DOMString {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_string_attribute("name")
    }

    fn SetName(&self, name: DOMString) {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.set_string_attribute("name", name)
    }

    fn Align(&self) -> DOMString {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_string_attribute("align")
    }

    fn SetAlign(&self, align: DOMString) {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.set_string_attribute("align", align)
    }

    fn Hspace(&self) -> u32 {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_uint_attribute("hspace")
    }

    fn SetHspace(&self, hspace: u32) {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.set_uint_attribute("hspace", hspace)
    }

    fn Vspace(&self) -> u32 {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_uint_attribute("vspace")
    }

    fn SetVspace(&self, vspace: u32) {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.set_uint_attribute("vspace", vspace)
    }

    fn LongDesc(&self) -> DOMString {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_string_attribute("longdesc")
    }

    fn SetLongDesc(&self, longdesc: DOMString) {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.set_string_attribute("longdesc", longdesc)
    }

    fn Border(&self) -> DOMString {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_string_attribute("border")
    }

    fn SetBorder(&self, border: DOMString) {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.set_string_attribute("border", border)
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLImageElement> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods+> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_ref(self);
        Some(htmlelement as &VirtualMethods+)
    }

    fn after_set_attr(&self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(name.clone(), value.clone()),
            _ => (),
        }

        if "src" == name.as_slice() {
            let window = window_from_node(self).root();
            let url = window.deref().get_url();
            self.update_image(Some((value, &url)));
        }
    }

    fn before_remove_attr(&self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(name.clone(), value.clone()),
            _ => (),
        }

        if "src" == name.as_slice() {
            self.update_image(None);
        }
    }

    fn parse_plain_attribute(&self, name: &str, value: DOMString) -> AttrValue {
        match name {
            "width" | "height" | "hspace" | "vspace" => AttrValue::from_u32(value, 0),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}

impl Reflectable for HTMLImageElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
