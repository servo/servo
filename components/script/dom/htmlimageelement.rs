/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::AttrValue;
use dom::bindings::codegen::Bindings::HTMLImageElementBinding;
use dom::bindings::codegen::Bindings::HTMLImageElementBinding::HTMLImageElementMethods;
use dom::bindings::codegen::InheritTypes::{NodeCast, ElementCast, HTMLElementCast, HTMLImageElementDerived};
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::{Element, HTMLImageElementTypeId};
use dom::element::AttributeHandlers;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId, NodeHelpers, window_from_node};
use dom::virtualmethods::VirtualMethods;
use servo_net::image_cache_task;
use servo_util::geometry::to_px;
use servo_util::str::DOMString;
use string_cache::Atom;

use url::{Url, UrlParser};

use std::cell::RefCell;

#[jstraceable]
#[must_root]
pub struct HTMLImageElement {
    pub htmlelement: HTMLElement,
    image: RefCell<Option<Url>>,
}

impl HTMLImageElementDerived for EventTarget {
    fn is_htmlimageelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLImageElementTypeId))
    }
}

trait PrivateHTMLImageElementHelpers {
    fn update_image(self, value: Option<(DOMString, &Url)>);
}

impl<'a> PrivateHTMLImageElementHelpers for JSRef<'a, HTMLImageElement> {
    /// Makes the local `image` member match the status of the `src` attribute and starts
    /// prefetching the image. This method must be called after `src` is changed.
    fn update_image(self, value: Option<(DOMString, &Url)>) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        let document = node.owner_doc().root();
        let window = document.deref().window.root();
        let image_cache = &window.image_cache_task;
        match value {
            None => {
                *self.image.borrow_mut() = None;
            }
            Some((src, base_url)) => {
                let img_url = UrlParser::new().base_url(base_url).parse(src.as_slice());
                // FIXME: handle URL parse errors more gracefully.
                let img_url = img_url.unwrap();
                *self.image.borrow_mut() = Some(img_url.clone());

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
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLImageElement {
        HTMLImageElement {
            htmlelement: HTMLElement::new_inherited(HTMLImageElementTypeId, localName, prefix, document),
            image: RefCell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLImageElement> {
        let element = HTMLImageElement::new_inherited(localName, prefix, document);
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

impl<'a> HTMLImageElementMethods for JSRef<'a, HTMLImageElement> {
    make_getter!(Alt)

    make_setter!(SetAlt, "alt")

    make_url_getter!(Src)

    make_setter!(SetSrc, "src")

    make_getter!(UseMap)

    make_setter!(SetUseMap, "usemap")

    make_bool_getter!(IsMap)

    fn SetIsMap(self, is_map: bool) {
        let element: JSRef<Element> = ElementCast::from_ref(self);
        element.set_string_attribute("ismap", is_map.to_string())
    }

    fn Width(self) -> u32 {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        let rect = node.get_bounding_content_box();
        to_px(rect.size.width) as u32
    }

    fn SetWidth(self, width: u32) {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        elem.set_uint_attribute("width", width)
    }

    fn Height(self) -> u32 {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        let rect = node.get_bounding_content_box();
        to_px(rect.size.height) as u32
    }

    fn SetHeight(self, height: u32) {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        elem.set_uint_attribute("height", height)
    }

    make_getter!(Name)

    make_setter!(SetName, "name")

    make_getter!(Align)

    make_setter!(SetAlign, "align")

    make_uint_getter!(Hspace)

    make_uint_setter!(SetHspace, "hspace")

    make_uint_getter!(Vspace)

    make_uint_setter!(SetVspace, "vspace")

    make_getter!(LongDesc)

    make_setter!(SetLongDesc, "longdesc")

    make_getter!(Border)

    make_setter!(SetBorder, "border")
}

impl<'a> VirtualMethods for JSRef<'a, HTMLImageElement> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, name: &Atom, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(name, value.clone()),
            _ => (),
        }

        if "src" == name.as_slice() {
            let window = window_from_node(*self).root();
            let url = window.deref().get_url();
            self.update_image(Some((value, &url)));
        }
    }

    fn before_remove_attr(&self, name: &Atom, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(name, value.clone()),
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
