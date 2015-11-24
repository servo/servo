/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::attr::AttrValue;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::HTMLImageElementBinding;
use dom::bindings::codegen::Bindings::HTMLImageElementBinding::HTMLImageElementMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{LayoutJS, Root};
use dom::bindings::refcounted::Trusted;
use dom::document::Document;
use dom::element::AttributeMutation;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeDamage, document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use net_traits::image::base::Image;
use net_traits::image_cache_task::{ImageResponder, ImageResponse};
use script_task::ScriptTaskEventCategory::UpdateReplacedElement;
use script_task::{CommonScriptMsg, Runnable, ScriptChan};
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
        let document = document_from_node(&*element);
        document.content_changed(element.upcast(), NodeDamage::OtherNodeDamage);

        // Fire image.onload
        let window = window_from_node(document.r());
        let event = Event::new(GlobalRef::Window(window.r()),
                               DOMString::from("load"),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable);
        event.fire(element.upcast());

        // Trigger reflow
        window.add_pending_reflow();
    }
}

impl HTMLImageElement {
    /// Makes the local `image` member match the status of the `src` attribute and starts
    /// prefetching the image. This method must be called after `src` is changed.
    fn update_image(&self, value: Option<(DOMString, Url)>) {
        let document = document_from_node(self);
        let window = document.window();
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
                let wrapper = window.get_runnable_wrapper();
                ROUTER.add_route(responder_receiver.to_opaque(), box move |message| {
                    // Return the image via a message to the script task, which marks the element
                    // as dirty and triggers a reflow.
                    let image_response = message.to().unwrap();
                    let runnable = ImageResponseHandlerRunnable::new(
                        trusted_node.clone(), image_response);
                    let runnable = wrapper.wrap_runnable(runnable);
                    script_chan.send(CommonScriptMsg::RunnableMsg(
                        UpdateReplacedElement, runnable)).unwrap();
                });

                image_cache.request_image(img_url,
                                          window.image_cache_chan(),
                                          Some(ImageResponder::new(responder_sender)));
            }
        }
    }

    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document) -> HTMLImageElement {
        HTMLImageElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document),
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
        let image = HTMLImageElement::new(DOMString::from("img"), None, document.r());
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
    make_getter!(Alt);
    // https://html.spec.whatwg.org/multipage/#dom-img-alt
    make_setter!(SetAlt, "alt");

    // https://html.spec.whatwg.org/multipage/#dom-img-src
    make_url_getter!(Src);
    // https://html.spec.whatwg.org/multipage/#dom-img-src
    make_setter!(SetSrc, "src");

    // https://html.spec.whatwg.org/multipage/#dom-img-usemap
    make_getter!(UseMap);
    // https://html.spec.whatwg.org/multipage/#dom-img-usemap
    make_setter!(SetUseMap, "usemap");

    // https://html.spec.whatwg.org/multipage/#dom-img-ismap
    make_bool_getter!(IsMap);
    // https://html.spec.whatwg.org/multipage/#dom-img-ismap
    make_bool_setter!(SetIsMap, "ismap");

    // https://html.spec.whatwg.org/multipage/#dom-img-width
    fn Width(&self) -> u32 {
        let node = self.upcast::<Node>();
        let rect = node.get_bounding_content_box();
        rect.size.width.to_px() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-width
    make_uint_setter!(SetWidth, "width");

    // https://html.spec.whatwg.org/multipage/#dom-img-height
    fn Height(&self) -> u32 {
        let node = self.upcast::<Node>();
        let rect = node.get_bounding_content_box();
        rect.size.height.to_px() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-height
    make_uint_setter!(SetHeight, "height");

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
    make_getter!(Name);

    // https://html.spec.whatwg.org/multipage/#dom-img-name
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-img-align
    make_getter!(Align);

    // https://html.spec.whatwg.org/multipage/#dom-img-align
    make_setter!(SetAlign, "align");

    // https://html.spec.whatwg.org/multipage/#dom-img-hspace
    make_uint_getter!(Hspace);

    // https://html.spec.whatwg.org/multipage/#dom-img-hspace
    make_uint_setter!(SetHspace, "hspace");

    // https://html.spec.whatwg.org/multipage/#dom-img-vspace
    make_uint_getter!(Vspace);

    // https://html.spec.whatwg.org/multipage/#dom-img-vspace
    make_uint_setter!(SetVspace, "vspace");

    // https://html.spec.whatwg.org/multipage/#dom-img-longdesc
    make_getter!(LongDesc);

    // https://html.spec.whatwg.org/multipage/#dom-img-longdesc
    make_setter!(SetLongDesc, "longdesc");

    // https://html.spec.whatwg.org/multipage/#dom-img-border
    make_getter!(Border);

    // https://html.spec.whatwg.org/multipage/#dom-img-border
    make_setter!(SetBorder, "border");
}

impl VirtualMethods for HTMLImageElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &atom!("src") => {
                self.update_image(mutation.new_value(attr).map(|value| {
                    // FIXME(ajeffrey): convert directly from AttrValue to DOMString
                    (DOMString::from(&**value), window_from_node(self).get_url())
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
