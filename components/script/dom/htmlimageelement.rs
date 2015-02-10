/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::attr::{AttrHelpers, AttrValue};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::HTMLImageElementBinding;
use dom::bindings::codegen::Bindings::HTMLImageElementBinding::HTMLImageElementMethods;
use dom::bindings::codegen::InheritTypes::{NodeCast, ElementCast, HTMLElementCast, HTMLImageElementDerived};
use dom::bindings::js::{JSRef, LayoutJS, Temporary};
use dom::document::{Document, DocumentHelpers};
use dom::element::Element;
use dom::element::AttributeHandlers;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId, NodeHelpers, NodeDamage, window_from_node};
use dom::virtualmethods::VirtualMethods;
use net::image_cache_task;
use util::geometry::to_px;
use util::str::DOMString;
use string_cache::Atom;

use url::{Url, UrlParser};

use std::borrow::ToOwned;

#[dom_struct]
pub struct HTMLImageElement {
    htmlelement: HTMLElement,
    image: DOMRefCell<Option<Url>>,
}

impl HTMLImageElementDerived for EventTarget {
    fn is_htmlimageelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLImageElement)))
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
        let window = document.r().window().root();
        let window = window.r();
        let image_cache = window.image_cache_task();
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
                image_cache.send(image_cache_task::Msg::Prefetch(img_url));
            }
        }
    }
}

impl HTMLImageElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLImageElement {
        HTMLImageElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLImageElement, localName, prefix, document),
            image: DOMRefCell::new(None),
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

impl LayoutHTMLImageElementHelpers for LayoutJS<HTMLImageElement> {
    unsafe fn image(&self) -> Option<Url> {
        (*self.unsafe_get()).image.borrow_for_layout().clone()
    }
}

impl<'a> HTMLImageElementMethods for JSRef<'a, HTMLImageElement> {
    make_getter!(Alt);

    make_setter!(SetAlt, "alt");

    make_url_getter!(Src);

    make_setter!(SetSrc, "src");

    make_getter!(UseMap);

    make_setter!(SetUseMap, "usemap");

    make_bool_getter!(IsMap);

    fn SetIsMap(self, is_map: bool) {
        let element: JSRef<Element> = ElementCast::from_ref(self);
        element.set_string_attribute(&atom!("ismap"), is_map.to_string())
    }

    fn Width(self) -> u32 {
        // FIXME(pcwalton): This is a really nasty thing to do, but the interaction between the
        // image cache task, the reflow messages that it sends to us via layout, and the image
        // holders seem to just plain be racy, and this works around it by ensuring that we
        // recreate the flow (picking up image changes on the way). The image cache task needs a
        // rewrite to modern Rust.
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.dirty(NodeDamage::OtherNodeDamage);

        let rect = node.get_bounding_content_box();
        to_px(rect.size.width) as u32
    }

    fn SetWidth(self, width: u32) {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        elem.set_uint_attribute(&atom!("width"), width)
    }

    fn Height(self) -> u32 {
        // FIXME(pcwalton): This is a really nasty thing to do, but the interaction between the
        // image cache task, the reflow messages that it sends to us via layout, and the image
        // holders seem to just plain be racy, and this works around it by ensuring that we
        // recreate the flow (picking up image changes on the way). The image cache task needs a
        // rewrite to modern Rust.
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.dirty(NodeDamage::OtherNodeDamage);

        let rect = node.get_bounding_content_box();
        to_px(rect.size.height) as u32
    }

    fn SetHeight(self, height: u32) {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        elem.set_uint_attribute(&atom!("height"), height)
    }

    make_getter!(Name);

    make_setter!(SetName, "name");

    make_getter!(Align);

    make_setter!(SetAlign, "align");

    make_uint_getter!(Hspace);

    make_uint_setter!(SetHspace, "hspace");

    make_uint_getter!(Vspace);

    make_uint_setter!(SetVspace, "vspace");

    make_getter!(LongDesc);

    make_setter!(SetLongDesc, "longdesc");

    make_getter!(Border);

    make_setter!(SetBorder, "border");
}

impl<'a> VirtualMethods for JSRef<'a, HTMLImageElement> {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(attr),
            _ => ()
        }

        match attr.local_name() {
            &atom!("src") => {
                let window = window_from_node(*self).root();
                let url = window.r().get_url();
                self.update_image(Some((attr.value().as_slice().to_owned(), &url)));
            },
            _ => ()
        }
    }

    fn before_remove_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(attr),
            _ => ()
        }

        match attr.local_name() {
            &atom!("src") => self.update_image(None),
            _ => ()
        }
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("width") | &atom!("height") |
            &atom!("hspace") | &atom!("vspace") => AttrValue::from_u32(value, 0),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}

