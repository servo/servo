/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLImageElementBinding;
use dom::bindings::codegen::InheritTypes::{NodeCast, HTMLImageElementDerived};
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast};
use dom::bindings::error::ErrorResult;
use dom::bindings::js::JS;
use dom::bindings::trace::Untraceable;
use dom::document::Document;
use dom::element::{Element, HTMLImageElementTypeId};
use dom::element::AttributeHandlers;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId, NodeHelpers, window_from_node};
use dom::virtualmethods::VirtualMethods;
use servo_util::geometry::to_px;
use servo_net::image_cache_task;
use servo_util::url::parse_url;
use servo_util::str::DOMString;
use url::Url;

#[deriving(Encodable)]
pub struct HTMLImageElement {
    pub htmlelement: HTMLElement,
    image: Untraceable<Option<Url>>,
}

impl HTMLImageElementDerived for EventTarget {
    fn is_htmlimageelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLImageElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLImageElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLImageElement {
        HTMLImageElement {
            htmlelement: HTMLElement::new_inherited(HTMLImageElementTypeId, localName, document),
            image: Untraceable::new(None),
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLImageElement> {
        let element = HTMLImageElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLImageElementBinding::Wrap)
    }
}

impl HTMLImageElement {
    pub fn image<'a>(&'a self) -> &'a Option<Url> {
        &*self.image
    }

    /// Makes the local `image` member match the status of the `src` attribute and starts
    /// prefetching the image. This method must be called after `src` is changed.
    fn update_image(&mut self, value: Option<DOMString>, url: Option<Url>) {
        let elem = &mut self.htmlelement.element;
        let document = elem.node.owner_doc();
        let window = document.get().window.get();
        let image_cache = &window.image_cache_task;
        match value {
            None => {
                *self.image = None;
            }
            Some(src) => {
                let img_url = parse_url(src, url);
                *self.image = Some(img_url.clone());

                // inform the image cache to load this, but don't store a
                // handle.
                //
                // TODO (Issue #84): don't prefetch if we are within a
                // <noscript> tag.
                image_cache.send(image_cache_task::Prefetch(img_url));
            }
        }
    }

    pub fn Alt(&self, abstract_self: &JS<HTMLImageElement>) -> DOMString {
        let element: JS<Element> = ElementCast::from(abstract_self);
        element.get_string_attribute("alt")
    }

    pub fn SetAlt(&mut self, abstract_self: &JS<HTMLImageElement>, alt: DOMString) {
        let mut element: JS<Element> = ElementCast::from(abstract_self);
        element.set_string_attribute("alt", alt)
    }

    pub fn Src(&self, abstract_self: &JS<HTMLImageElement>) -> DOMString {
        let element: JS<Element> = ElementCast::from(abstract_self);
        element.get_string_attribute("src")
    }

    pub fn SetSrc(&mut self, abstract_self: &mut JS<HTMLImageElement>, src: DOMString) {
        let mut element: JS<Element> = ElementCast::from(abstract_self);
        element.set_url_attribute("src", src)
    }

    pub fn CrossOrigin(&self) -> DOMString {
        ~""
    }

    pub fn SetCrossOrigin(&mut self, _cross_origin: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn UseMap(&self, abstract_self: &JS<HTMLImageElement>) -> DOMString {
        let element: JS<Element> = ElementCast::from(abstract_self);
        element.get_string_attribute("useMap")
    }

    pub fn SetUseMap(&mut self, abstract_self: &mut JS<HTMLImageElement>, use_map: DOMString) {
        let mut element: JS<Element> = ElementCast::from(abstract_self);
        element.set_string_attribute("useMap", use_map)
    }

    pub fn IsMap(&self, abstract_self: &JS<HTMLImageElement>) -> bool {
        let element: JS<Element> = ElementCast::from(abstract_self);
        from_str::<bool>(element.get_string_attribute("hspace")).unwrap()
    }

    pub fn SetIsMap(&self, abstract_self: &mut JS<HTMLImageElement>, is_map: bool) {
        let mut element: JS<Element> = ElementCast::from(abstract_self);
        element.set_string_attribute("isMap", is_map.to_str())
    }

    pub fn Width(&self, abstract_self: &JS<HTMLImageElement>) -> u32 {
        let node: JS<Node> = NodeCast::from(abstract_self);
        let rect = node.get_bounding_content_box();
        to_px(rect.size.width) as u32
    }

    pub fn SetWidth(&mut self, abstract_self: &JS<HTMLImageElement>, width: u32) {
        let mut elem: JS<Element> = ElementCast::from(abstract_self);
        elem.set_uint_attribute("width", width)
    }

    pub fn Height(&self, abstract_self: &JS<HTMLImageElement>) -> u32 {
        let node: JS<Node> = NodeCast::from(abstract_self);
        let rect = node.get_bounding_content_box();
        to_px(rect.size.height) as u32
    }

    pub fn SetHeight(&mut self, abstract_self: &JS<HTMLImageElement>, height: u32) {
        let mut elem: JS<Element> = ElementCast::from(abstract_self);
        elem.set_uint_attribute("height", height)
    }

    pub fn NaturalWidth(&self) -> u32 {
        0
    }

    pub fn NaturalHeight(&self) -> u32 {
        0
    }

    pub fn Complete(&self) -> bool {
        false
    }

    pub fn Name(&self, abstract_self: &JS<HTMLImageElement>) -> DOMString {
        let element: JS<Element> = ElementCast::from(abstract_self);
        element.get_string_attribute("name")
    }

    pub fn SetName(&mut self, abstract_self: &mut JS<HTMLImageElement>, name: DOMString) {
        let mut element: JS<Element> = ElementCast::from(abstract_self);
        element.set_string_attribute("name", name)
    }

    pub fn Align(&self, abstract_self: &JS<HTMLImageElement>) -> DOMString {
        let element: JS<Element> = ElementCast::from(abstract_self);
        element.get_string_attribute("longdesc")
    }

    pub fn SetAlign(&mut self, abstract_self: &mut JS<HTMLImageElement>, align: DOMString) {
        let mut element: JS<Element> = ElementCast::from(abstract_self);
        element.set_string_attribute("align", align)
    }

    pub fn Hspace(&self, abstract_self: &JS<HTMLImageElement>) -> u32 {
        let element: JS<Element> = ElementCast::from(abstract_self);
        from_str::<u32>(element.get_string_attribute("hspace")).unwrap()
    }

    pub fn SetHspace(&mut self, abstract_self: &mut JS<HTMLImageElement>, hspace: u32) {
        let mut element: JS<Element> = ElementCast::from(abstract_self);
        element.set_uint_attribute("hspace", hspace)
    }

    pub fn Vspace(&self, abstract_self: &JS<HTMLImageElement>) -> u32 {
        let element: JS<Element> = ElementCast::from(abstract_self);
        from_str::<u32>(element.get_string_attribute("vspace")).unwrap()
    }

    pub fn SetVspace(&mut self, abstract_self: &mut JS<HTMLImageElement>, vspace: u32) {
        let mut element: JS<Element> = ElementCast::from(abstract_self);
        element.set_uint_attribute("vspace", vspace)
    }

    pub fn LongDesc(&self, abstract_self: &JS<HTMLImageElement>) -> DOMString {
        let element: JS<Element> = ElementCast::from(abstract_self);
        element.get_string_attribute("longdesc")
    }

    pub fn SetLongDesc(&mut self, abstract_self: &mut JS<HTMLImageElement>, longdesc: DOMString) {
        let mut element: JS<Element> = ElementCast::from(abstract_self);
        element.set_string_attribute("longdesc", longdesc)
    }

    pub fn Border(&self, abstract_self: &JS<HTMLImageElement>) -> DOMString {
        let element: JS<Element> = ElementCast::from(abstract_self);
        element.get_string_attribute("border")
    }

    pub fn SetBorder(&mut self, abstract_self: &mut JS<HTMLImageElement>, border: DOMString) {
        let mut element: JS<Element> = ElementCast::from(abstract_self);
        element.set_string_attribute("border", border)
    }
}

impl VirtualMethods for JS<HTMLImageElement> {
    fn super_type(&self) -> Option<~VirtualMethods:> {
        let htmlelement: JS<HTMLElement> = HTMLElementCast::from(self);
        Some(~htmlelement as ~VirtualMethods:)
    }

    fn after_set_attr(&mut self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref mut s) => s.after_set_attr(name.clone(), value.clone()),
            _ => (),
        }

        if "src" == name {
            let window = window_from_node(self);
            let url = Some(window.get().get_url());
            self.get_mut().update_image(Some(value), url);
        }
    }

    fn before_remove_attr(&mut self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref mut s) => s.before_remove_attr(name.clone(), value.clone()),
            _ => (),
        }

        if "src" == name {
            self.get_mut().update_image(None, None);
        }
    }
}
