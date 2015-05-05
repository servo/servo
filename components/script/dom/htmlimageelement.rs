/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::attr::{AttrHelpers, AttrValue};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::HTMLImageElementBinding;
use dom::bindings::codegen::Bindings::HTMLImageElementBinding::HTMLImageElementMethods;
use dom::bindings::codegen::InheritTypes::{NodeCast, ElementCast, EventTargetCast, HTMLElementCast, HTMLImageElementDerived};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, LayoutJS, Rootable, Temporary};
use dom::bindings::refcounted::Trusted;
use dom::document::{Document, DocumentHelpers};
use dom::element::Element;
use dom::element::AttributeHandlers;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::event::{Event, EventBubbles, EventCancelable, EventHelpers};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{document_from_node, Node, NodeTypeId, NodeHelpers, NodeDamage, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom::window::WindowHelpers;
use util::str::DOMString;
use string_cache::Atom;

use net_traits::image::base::Image;
use net_traits::image_cache_task::ImageResponder;
use url::{Url, UrlParser};

use std::borrow::ToOwned;
use std::sync::Arc;

#[dom_struct]
pub struct HTMLImageElement {
    htmlelement: HTMLElement,
    url: DOMRefCell<Option<Url>>,
    image: DOMRefCell<Option<Arc<Image>>>,
}

impl HTMLImageElementDerived for EventTarget {
    fn is_htmlimageelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLImageElement)))
    }
}

pub trait HTMLImageElementHelpers {
    fn get_url(&self) -> Option<Url>;
}

impl<'a> HTMLImageElementHelpers for JSRef<'a, HTMLImageElement> {
    fn get_url(&self) -> Option<Url>{
        self.url.borrow().clone()
    }
}

trait PrivateHTMLImageElementHelpers {
    fn update_image(self, value: Option<(DOMString, &Url)>);
}

/// This is passed to the image cache when the src attribute
/// changes. It is returned via a message to the script task,
/// which marks the element as dirty and triggers a reflow.
struct Responder {
    element: Trusted<HTMLImageElement>,
}

impl Responder {
    fn new(element: Trusted<HTMLImageElement>) -> Responder {
        Responder {
            element: element
        }
    }
}

impl ImageResponder for Responder {
    fn respond(&self, image: Option<Arc<Image>>) {
        // Update the image field
        let element = self.element.to_temporary().root();
        let element_ref = element.r();
        *element_ref.image.borrow_mut() = image;

        // Mark the node dirty
        let node = NodeCast::from_ref(element.r());
        let document = document_from_node(node).root();
        document.r().content_changed(node, NodeDamage::OtherNodeDamage);

        // Fire image.onload
        let window = window_from_node(document.r()).root();
        let event = Event::new(GlobalRef::Window(window.r()),
                               "load".to_owned(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable).root();
        let event = event.r();
        let target: JSRef<EventTarget> = EventTargetCast::from_ref(node);
        event.fire(target);

        // Trigger reflow
        window.r().add_pending_reflow();
    }
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
                *self.url.borrow_mut() = None;
                *self.image.borrow_mut() = None;
            }
            Some((src, base_url)) => {
                let img_url = UrlParser::new().base_url(base_url).parse(&src);
                // FIXME: handle URL parse errors more gracefully.
                let img_url = img_url.unwrap();
                *self.url.borrow_mut() = Some(img_url.clone());

                let trusted_node = Trusted::new(window.get_cx(), self, window.script_chan());
                let responder = box Responder::new(trusted_node);
                image_cache.request_image(img_url, window.image_cache_chan(), Some(responder));
            }
        }
    }
}

impl HTMLImageElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLImageElement {
        HTMLImageElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLImageElement, localName, prefix, document),
            url: DOMRefCell::new(None),
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
    #[allow(unsafe_code)]
    unsafe fn image(&self) -> Option<Arc<Image>>;

    #[allow(unsafe_code)]
    unsafe fn image_url(&self) -> Option<Url>;
}

impl LayoutHTMLImageElementHelpers for LayoutJS<HTMLImageElement> {
    #[allow(unsafe_code)]
    unsafe fn image(&self) -> Option<Arc<Image>> {
        (*self.unsafe_get()).image.borrow_for_layout().clone()
    }

    #[allow(unsafe_code)]
    unsafe fn image_url(&self) -> Option<Url> {
        (*self.unsafe_get()).url.borrow_for_layout().clone()
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
        let node: JSRef<Node> = NodeCast::from_ref(self);
        let rect = node.get_bounding_content_box();
        rect.size.width.to_px() as u32
    }

    fn SetWidth(self, width: u32) {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        elem.set_uint_attribute(&atom!("width"), width)
    }

    fn Height(self) -> u32 {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        let rect = node.get_bounding_content_box();
        rect.size.height.to_px() as u32
    }

    fn SetHeight(self, height: u32) {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        elem.set_uint_attribute(&atom!("height"), height)
    }

    fn NaturalWidth(self) -> u32 {
        let image = self.image.borrow();

        match *image {
            Some(ref image) => image.width,
            None => 0,
        }
    }

    fn NaturalHeight(self) -> u32 {
        let image = self.image.borrow();

        match *image {
            Some(ref image) => image.height,
            None => 0,
        }
    }

    fn Complete(self) -> bool {
        let image = self.image.borrow();
        image.is_some()
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
        if let Some(ref s) = self.super_type() {
            s.after_set_attr(attr);
        }

        match attr.local_name() {
            &atom!("src") => {
                let window = window_from_node(*self).root();
                let url = window.r().get_url();
                self.update_image(Some(((**attr.value()).to_owned(), &url)));
            },
            _ => ()
        }
    }

    fn before_remove_attr(&self, attr: JSRef<Attr>) {
        if let Some(ref s) = self.super_type() {
            s.before_remove_attr(attr);
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

