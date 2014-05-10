/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLImageElementBinding;
use dom::bindings::codegen::InheritTypes::{NodeCast, ElementCast, HTMLElementCast, HTMLImageElementDerived};
use dom::bindings::error::ErrorResult;
use dom::bindings::js::{JS, JSRef, Temporary};
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
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLImageElementTypeId))
    }
}

trait PrivateHTMLImageElementHelpers {
    fn update_image(&mut self, value: Option<DOMString>, url: Option<Url>);
}

impl<'a> PrivateHTMLImageElementHelpers for JSRef<'a, HTMLImageElement> {
    /// Makes the local `image` member match the status of the `src` attribute and starts
    /// prefetching the image. This method must be called after `src` is changed.
    fn update_image(&mut self, value: Option<DOMString>, url: Option<Url>) {
        let self_alias = self.clone();
        let node_alias: &JSRef<Node> = NodeCast::from_ref(&self_alias);
        let document = node_alias.owner_doc().root();
        let window = document.deref().window.root();
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
}

impl HTMLImageElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLImageElement {
        HTMLImageElement {
            htmlelement: HTMLElement::new_inherited(HTMLImageElementTypeId, localName, document),
            image: Untraceable::new(None),
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLImageElement> {
        let element = HTMLImageElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLImageElementBinding::Wrap)
    }
}

pub trait LayoutHTMLImageElementHelpers {
    unsafe fn image<'a>(&'a self) -> &'a Option<Url>;
}

impl LayoutHTMLImageElementHelpers for JS<HTMLImageElement> {
    unsafe fn image<'a>(&'a self) -> &'a Option<Url> {
        &*(*self.unsafe_get()).image
    }
}

pub trait HTMLImageElementMethods {
    fn Alt(&self) -> DOMString;
    fn SetAlt(&mut self, alt: DOMString);
    fn Src(&self) -> DOMString;
    fn SetSrc(&mut self, src: DOMString);
    fn CrossOrigin(&self) -> DOMString;
    fn SetCrossOrigin(&mut self, _cross_origin: DOMString) -> ErrorResult;
    fn UseMap(&self) -> DOMString;
    fn SetUseMap(&mut self, use_map: DOMString);
    fn IsMap(&self) -> bool;
    fn SetIsMap(&mut self, is_map: bool);
    fn Width(&self) -> u32;
    fn SetWidth(&mut self, width: u32);
    fn Height(&self) -> u32;
    fn SetHeight(&mut self, height: u32);
    fn NaturalWidth(&self) -> u32;
    fn NaturalHeight(&self) -> u32;
    fn Complete(&self) -> bool;
    fn Name(&self) -> DOMString;
    fn SetName(&mut self, name: DOMString);
    fn Align(&self) -> DOMString;
    fn SetAlign(&mut self, align: DOMString);
    fn Hspace(&self) -> u32;
    fn SetHspace(&mut self, hspace: u32);
    fn Vspace(&self) -> u32;
    fn SetVspace(&mut self, vspace: u32);
    fn LongDesc(&self) -> DOMString;
    fn SetLongDesc(&mut self, longdesc: DOMString);
    fn Border(&self) -> DOMString;
    fn SetBorder(&mut self, border: DOMString);
}

impl<'a> HTMLImageElementMethods for JSRef<'a, HTMLImageElement> {
    fn Alt(&self) -> DOMString {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_string_attribute("alt")
    }

    fn SetAlt(&mut self, alt: DOMString) {
        let element: &mut JSRef<Element> = ElementCast::from_mut_ref(self);
        element.set_string_attribute("alt", alt)
    }

    fn Src(&self) -> DOMString {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_string_attribute("src")
    }

    fn SetSrc(&mut self, src: DOMString) {
        let element: &mut JSRef<Element> = ElementCast::from_mut_ref(self);
        element.set_url_attribute("src", src)
    }

    fn CrossOrigin(&self) -> DOMString {
        "".to_owned()
    }

    fn SetCrossOrigin(&mut self, _cross_origin: DOMString) -> ErrorResult {
        Ok(())
    }

    fn UseMap(&self) -> DOMString {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_string_attribute("useMap")
    }

    fn SetUseMap(&mut self, use_map: DOMString) {
        let element: &mut JSRef<Element> = ElementCast::from_mut_ref(self);
        element.set_string_attribute("useMap", use_map)
    }

    fn IsMap(&self) -> bool {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        from_str::<bool>(element.get_string_attribute("hspace")).unwrap()
    }

    fn SetIsMap(&mut self, is_map: bool) {
        let element: &mut JSRef<Element> = ElementCast::from_mut_ref(self);
        element.set_string_attribute("isMap", is_map.to_str())
    }

    fn Width(&self) -> u32 {
        let node: &JSRef<Node> = NodeCast::from_ref(self);
        let rect = node.get_bounding_content_box();
        to_px(rect.size.width) as u32
    }

    fn SetWidth(&mut self, width: u32) {
        let elem: &mut JSRef<Element> = ElementCast::from_mut_ref(self);
        elem.set_uint_attribute("width", width)
    }

    fn Height(&self) -> u32 {
        let node: &JSRef<Node> = NodeCast::from_ref(self);
        let rect = node.get_bounding_content_box();
        to_px(rect.size.height) as u32
    }

    fn SetHeight(&mut self, height: u32) {
        let elem: &mut JSRef<Element> = ElementCast::from_mut_ref(self);
        elem.set_uint_attribute("height", height)
    }

    fn NaturalWidth(&self) -> u32 {
        0
    }

    fn NaturalHeight(&self) -> u32 {
        0
    }

    fn Complete(&self) -> bool {
        false
    }

    fn Name(&self) -> DOMString {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_string_attribute("name")
    }

    fn SetName(&mut self, name: DOMString) {
        let element: &mut JSRef<Element> = ElementCast::from_mut_ref(self);
        element.set_string_attribute("name", name)
    }

    fn Align(&self) -> DOMString {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_string_attribute("align")
    }

    fn SetAlign(&mut self, align: DOMString) {
        let element: &mut JSRef<Element> = ElementCast::from_mut_ref(self);
        element.set_string_attribute("align", align)
    }

    fn Hspace(&self) -> u32 {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        from_str::<u32>(element.get_string_attribute("hspace")).unwrap()
    }

    fn SetHspace(&mut self, hspace: u32) {
        let element: &mut JSRef<Element> = ElementCast::from_mut_ref(self);
        element.set_uint_attribute("hspace", hspace)
    }

    fn Vspace(&self) -> u32 {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        from_str::<u32>(element.get_string_attribute("vspace")).unwrap()
    }

    fn SetVspace(&mut self, vspace: u32) {
        let element: &mut JSRef<Element> = ElementCast::from_mut_ref(self);
        element.set_uint_attribute("vspace", vspace)
    }

    fn LongDesc(&self) -> DOMString {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_string_attribute("longdesc")
    }

    fn SetLongDesc(&mut self, longdesc: DOMString) {
        let element: &mut JSRef<Element> = ElementCast::from_mut_ref(self);
        element.set_string_attribute("longdesc", longdesc)
    }

    fn Border(&self) -> DOMString {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_string_attribute("border")
    }

    fn SetBorder(&mut self, border: DOMString) {
        let element: &mut JSRef<Element> = ElementCast::from_mut_ref(self);
        element.set_string_attribute("border", border)
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLImageElement> {
    fn super_type<'a>(&'a mut self) -> Option<&'a mut VirtualMethods:> {
        let htmlelement: &mut JSRef<HTMLElement> = HTMLElementCast::from_mut_ref(self);
        Some(htmlelement as &mut VirtualMethods:)
    }

    fn after_set_attr(&mut self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref mut s) => s.after_set_attr(name.clone(), value.clone()),
            _ => (),
        }

        if "src" == name {
            let window = window_from_node(self).root();
            let url = Some(window.deref().get_url());
            self.update_image(Some(value), url);
        }
    }

    fn before_remove_attr(&mut self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref mut s) => s.before_remove_attr(name.clone(), value.clone()),
            _ => (),
        }

        if "src" == name {
            self.update_image(None, None);
        }
    }
}
