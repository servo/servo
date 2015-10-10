/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::attr::AttrValue;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::HTMLImageElementBinding;
use dom::bindings::codegen::Bindings::HTMLImageElementBinding::HTMLImageElementMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, EventTargetCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLImageElementDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{LayoutJS, Root};
use dom::bindings::refcounted::Trusted;
use dom::document::Document;
use dom::element::{AttributeMutation, ElementTypeId};
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeDamage, NodeTypeId, document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use net_traits::image::base::Image;
use net_traits::image_cache_task::{ImageResponder, ImageResponse};
use script_task::ScriptTaskEventCategory::UpdateReplacedElement;
use script_task::{CommonScriptMsg, Runnable, ScriptChan};
use std::borrow::ToOwned;
use std::sync::Arc;
use string_cache::Atom;
use url::{Url, UrlParser};
use util::str::DOMString;

#[dom_struct]
pub struct HTMLImageElement {
    htmlelement: HTMLElement,
    url: DOMRefCell<Option<Url>>,
    image: DOMRefCell<Option<Arc<Image>>>,
}

impl HTMLImageElementDerived for EventTarget {
    fn is_htmlimageelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLImageElement)))
    }
}


impl HTMLImageElement {
    pub fn get_url(&self) -> Option<Url>{
        self.url.borrow().clone()
    }
}


struct ImageResponseHandlerRunnable {
    element: Trusted<HTMLImageElement>,
    image: ImageResponse,
}

impl ImageResponseHandlerRunnable {
    fn new(element: Trusted<HTMLImageElement>, image: ImageResponse)
           -> ImageResponseHandlerRunnable {
        ImageResponseHandlerRunnable {
            element: element,
            image: image,
        }
    }
}

impl Runnable for ImageResponseHandlerRunnable {
    fn handler(self: Box<Self>) {
        // Update the image field
        let element = self.element.root();
        let element_ref = element.r();
        *element_ref.image.borrow_mut() = match self.image {
            ImageResponse::Loaded(image) | ImageResponse::PlaceholderLoaded(image) => {
                Some(image)
            }
            ImageResponse::None => None,
        };

        // Mark the node dirty
        let node = NodeCast::from_ref(element.r());
        let document = document_from_node(node);
        document.r().content_changed(node, NodeDamage::OtherNodeDamage);

        // Fire image.onload
        let window = window_from_node(document.r());
        let event = Event::new(GlobalRef::Window(window.r()),
                               "load".to_owned(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable);
        let event = event.r();
        let target = EventTargetCast::from_ref(node);
        event.fire(target);

        // Trigger reflow
        window.r().add_pending_reflow();
    }
}

impl HTMLImageElement {
    /// Makes the local `image` member match the status of the `src` attribute and starts
    /// prefetching the image. This method must be called after `src` is changed.
    fn update_image(&self, value: Option<(DOMString, Url)>) {
        let node = NodeCast::from_ref(self);
        let document = node.owner_doc();
        let window = document.r().window();
        let window = window.r();
        let image_cache = window.image_cache_task();
        match value {
            None => {
                *self.url.borrow_mut() = None;
                *self.image.borrow_mut() = None;
            }
            Some((src, base_url)) => {
                let img_url = UrlParser::new().base_url(&base_url).parse(&src);
                // FIXME: handle URL parse errors more gracefully.
                let img_url = img_url.unwrap();
                *self.url.borrow_mut() = Some(img_url.clone());

                let trusted_node = Trusted::new(window.get_cx(), self, window.script_chan());
                let (responder_sender, responder_receiver) = ipc::channel().unwrap();
                let script_chan = window.script_chan();
                ROUTER.add_route(responder_receiver.to_opaque(), box move |message| {
                    // Return the image via a message to the script task, which marks the element
                    // as dirty and triggers a reflow.
                    let image_response = message.to().unwrap();
                    script_chan.send(CommonScriptMsg::RunnableMsg(UpdateReplacedElement,
                        box ImageResponseHandlerRunnable::new(
                            trusted_node.clone(), image_response))).unwrap();
                });

                image_cache.request_image(img_url,
                                          window.image_cache_chan(),
                                          Some(ImageResponder::new(responder_sender)));
            }
        }
    }

    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document) -> HTMLImageElement {
        HTMLImageElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLImageElement, localName, prefix, document),
            url: DOMRefCell::new(None),
            image: DOMRefCell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLImageElement> {
        let element = HTMLImageElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLImageElementBinding::Wrap)
    }

    pub fn Image(global: GlobalRef,
                 width: Option<u32>,
                 height: Option<u32>) -> Fallible<Root<HTMLImageElement>> {
        let document = global.as_window().Document();
        let image = HTMLImageElement::new("img".to_owned(), None, document.r());
        if let Some(w) = width {
            image.SetWidth(w);
        }
        if let Some(h) = height {
            image.SetHeight(h);
        }

        Ok(image)
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

impl HTMLImageElementMethods for HTMLImageElement {
    // https://html.spec.whatwg.org/multipage/#dom-img-alt
    make_getter_setter!(Alt, SetAlt);

    // https://html.spec.whatwg.org/multipage/#dom-img-src
    make_url_getter_setter!(Src, SetSrc);

    // https://html.spec.whatwg.org/multipage/#dom-img-usemap
    make_getter_setter!(UseMap, SetUseMap);

    // https://html.spec.whatwg.org/multipage/#dom-img-ismap
    make_bool_getter_setter!(IsMap, SetIsMap);

    // https://html.spec.whatwg.org/multipage/#dom-img-width
    fn Width(&self) -> u32 {
        let node = NodeCast::from_ref(self);
        let rect = node.get_bounding_content_box();
        rect.size.width.to_px() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-width
    fn SetWidth(&self, width: u32) {
        // FIXME: This needs to cap width at 2147483647.
        let elem = ElementCast::from_ref(self);
        elem.set_uint_attribute(&atom!("width"), width)
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-height
    fn Height(&self) -> u32 {
        let node = NodeCast::from_ref(self);
        let rect = node.get_bounding_content_box();
        rect.size.height.to_px() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-height
    fn SetHeight(&self, height: u32) {
        // FIXME: This needs to cap height at 2147483647.
        let elem = ElementCast::from_ref(self);
        elem.set_uint_attribute(&atom!("height"), height)
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-naturalwidth
    fn NaturalWidth(&self) -> u32 {
        let image = self.image.borrow();

        match *image {
            Some(ref image) => image.width,
            None => 0,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-naturalheight
    fn NaturalHeight(&self) -> u32 {
        let image = self.image.borrow();

        match *image {
            Some(ref image) => image.height,
            None => 0,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-complete
    fn Complete(&self) -> bool {
        let image = self.image.borrow();
        image.is_some()
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-name
    make_atomic_getter_setter!(Name, SetName);

    // https://html.spec.whatwg.org/multipage/#dom-img-align
    make_getter_setter!(Align, SetAlign);

    // https://html.spec.whatwg.org/multipage/#dom-img-hspace
    make_uint_getter_setter!(Hspace, SetHspace);

    // https://html.spec.whatwg.org/multipage/#dom-img-vspace
    make_uint_getter_setter!(Vspace, SetVspace);

    // https://html.spec.whatwg.org/multipage/#dom-img-longdesc
    make_getter_setter!(LongDesc, SetLongDesc);

    // https://html.spec.whatwg.org/multipage/#dom-img-border
    make_getter_setter!(Border, SetBorder);
}

impl VirtualMethods for HTMLImageElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &HTMLElement = HTMLElementCast::from_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &atom!(src) => {
                self.update_image(mutation.new_value(attr).map(|value| {
                    ((**value).to_owned(), window_from_node(self).get_url())
                }));
            },
            _ => {},
        }
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("name") => AttrValue::from_atomic(value),
            &atom!("width") | &atom!("height") |
            &atom!("hspace") | &atom!("vspace") => AttrValue::from_u32(value, 0),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}
