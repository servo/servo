/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
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
use dom::element::{AttributeMutation, Element, RawLayoutElementHelpers};
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeDamage, document_from_node, window_from_node};
use dom::values::UNSIGNED_LONG_MAX;
use dom::virtualmethods::VirtualMethods;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use net_traits::image::base::{Image, ImageMetadata};
use net_traits::image_cache_thread::{ImageResponder, ImageResponse};
use script_thread::ScriptThreadEventCategory::UpdateReplacedElement;
use script_thread::{CommonScriptMsg, Runnable, ScriptChan};
use std::sync::Arc;
use string_cache::Atom;
use url::Url;
use util::str::{DOMString, LengthOrPercentageOrAuto};

#[dom_struct]
pub struct HTMLImageElement {
    htmlelement: HTMLElement,
    url: DOMRefCell<Option<Url>>,
    image: DOMRefCell<Option<Arc<Image>>>,
    metadata: DOMRefCell<Option<ImageMetadata>>,
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
        let (image, metadata, trigger_image_load) = match self.image {
            ImageResponse::Loaded(image) | ImageResponse::PlaceholderLoaded(image) => {
                (Some(image.clone()), Some(ImageMetadata { height: image.height, width: image.width } ), true)
            }
            ImageResponse::MetadataLoaded(meta) => {
                (None, Some(meta), false)
            }
            ImageResponse::None => (None, None, true)
        };
        *element_ref.image.borrow_mut() = image;
        *element_ref.metadata.borrow_mut() = metadata;

        // Mark the node dirty
        let document = document_from_node(&*element);
        document.content_changed(element.upcast(), NodeDamage::OtherNodeDamage);

        // Fire image.onload
        if trigger_image_load {
            element.upcast::<EventTarget>().fire_simple_event("load");
        }

        // Trigger reflow
        let window = window_from_node(document.r());
        window.add_pending_reflow();
    }
}

impl HTMLImageElement {
    /// Makes the local `image` member match the status of the `src` attribute and starts
    /// prefetching the image. This method must be called after `src` is changed.
    fn update_image(&self, value: Option<(DOMString, Url)>) {
        let document = document_from_node(self);
        let window = document.window();
        let image_cache = window.image_cache_thread();
        match value {
            None => {
                *self.url.borrow_mut() = None;
                *self.image.borrow_mut() = None;
            }
            Some((src, base_url)) => {
                let img_url = base_url.join(&src);
                // FIXME: handle URL parse errors more gracefully.
                let img_url = img_url.unwrap();
                *self.url.borrow_mut() = Some(img_url.clone());

                let trusted_node = Trusted::new(self, window.networking_task_source());
                let (responder_sender, responder_receiver) = ipc::channel().unwrap();
                let script_chan = window.networking_task_source();
                let wrapper = window.get_runnable_wrapper();
                ROUTER.add_route(responder_receiver.to_opaque(), box move |message| {
                    // Return the image via a message to the script thread, which marks the element
                    // as dirty and triggers a reflow.
                    let image_response = message.to().unwrap();
                    let runnable = ImageResponseHandlerRunnable::new(
                        trusted_node.clone(), image_response);
                    let runnable = wrapper.wrap_runnable(runnable);
                    let _ = script_chan.send(CommonScriptMsg::RunnableMsg(
                        UpdateReplacedElement, runnable));
                });

                image_cache.request_image_and_metadata(img_url,
                                          window.image_cache_chan(),
                                          Some(ImageResponder::new(responder_sender)));
            }
        }
    }

    fn new_inherited(localName: Atom, prefix: Option<DOMString>, document: &Document) -> HTMLImageElement {
        HTMLImageElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document),
            url: DOMRefCell::new(None),
            image: DOMRefCell::new(None),
            metadata: DOMRefCell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLImageElement> {
        let element = HTMLImageElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLImageElementBinding::Wrap)
    }

    pub fn Image(global: GlobalRef,
                 width: Option<u32>,
                 height: Option<u32>) -> Fallible<Root<HTMLImageElement>> {
        let document = global.as_window().Document();
        let image = HTMLImageElement::new(atom!("img"), None, document.r());
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

    fn get_width(&self) -> LengthOrPercentageOrAuto;
    fn get_height(&self) -> LengthOrPercentageOrAuto;
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

    #[allow(unsafe_code)]
    fn get_width(&self) -> LengthOrPercentageOrAuto {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &atom!("width"))
                .map(AttrValue::as_dimension)
                .cloned()
                .unwrap_or(LengthOrPercentageOrAuto::Auto)
        }
    }

    #[allow(unsafe_code)]
    fn get_height(&self) -> LengthOrPercentageOrAuto {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &atom!("height"))
                .map(AttrValue::as_dimension)
                .cloned()
                .unwrap_or(LengthOrPercentageOrAuto::Auto)
        }
    }
}

impl HTMLImageElementMethods for HTMLImageElement {
    // https://html.spec.whatwg.org/multipage/#dom-img-alt
    make_getter!(Alt, "alt");
    // https://html.spec.whatwg.org/multipage/#dom-img-alt
    make_setter!(SetAlt, "alt");

    // https://html.spec.whatwg.org/multipage/#dom-img-src
    make_url_getter!(Src, "src");
    // https://html.spec.whatwg.org/multipage/#dom-img-src
    make_setter!(SetSrc, "src");

    // https://html.spec.whatwg.org/multipage/#dom-img-usemap
    make_getter!(UseMap, "usemap");
    // https://html.spec.whatwg.org/multipage/#dom-img-usemap
    make_setter!(SetUseMap, "usemap");

    // https://html.spec.whatwg.org/multipage/#dom-img-ismap
    make_bool_getter!(IsMap, "ismap");
    // https://html.spec.whatwg.org/multipage/#dom-img-ismap
    make_bool_setter!(SetIsMap, "ismap");

    // https://html.spec.whatwg.org/multipage/#dom-img-width
    fn Width(&self) -> u32 {
        let node = self.upcast::<Node>();
        let rect = node.get_bounding_content_box();
        rect.size.width.to_px() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-width
    fn SetWidth(&self, value: u32) {
        image_dimension_setter(self.upcast(), atom!("width"), value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-height
    fn Height(&self) -> u32 {
        let node = self.upcast::<Node>();
        let rect = node.get_bounding_content_box();
        rect.size.height.to_px() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-height
    fn SetHeight(&self, value: u32) {
        image_dimension_setter(self.upcast(), atom!("height"), value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-naturalwidth
    fn NaturalWidth(&self) -> u32 {
        let metadata = self.metadata.borrow();

        match *metadata {
            Some(ref metadata) => metadata.width,
            None => 0,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-naturalheight
    fn NaturalHeight(&self) -> u32 {
        let metadata = self.metadata.borrow();

        match *metadata {
            Some(ref metadata) => metadata.height,
            None => 0,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-complete
    fn Complete(&self) -> bool {
        let image = self.image.borrow();
        image.is_some()
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-img-name
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-img-align
    make_getter!(Align, "align");

    // https://html.spec.whatwg.org/multipage/#dom-img-align
    make_setter!(SetAlign, "align");

    // https://html.spec.whatwg.org/multipage/#dom-img-hspace
    make_uint_getter!(Hspace, "hspace");

    // https://html.spec.whatwg.org/multipage/#dom-img-hspace
    make_uint_setter!(SetHspace, "hspace");

    // https://html.spec.whatwg.org/multipage/#dom-img-vspace
    make_uint_getter!(Vspace, "vspace");

    // https://html.spec.whatwg.org/multipage/#dom-img-vspace
    make_uint_setter!(SetVspace, "vspace");

    // https://html.spec.whatwg.org/multipage/#dom-img-longdesc
    make_getter!(LongDesc, "longdesc");

    // https://html.spec.whatwg.org/multipage/#dom-img-longdesc
    make_setter!(SetLongDesc, "longdesc");

    // https://html.spec.whatwg.org/multipage/#dom-img-border
    make_getter!(Border, "border");

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
            &atom!("width") | &atom!("height") => AttrValue::from_dimension(value),
            &atom!("hspace") | &atom!("vspace") => AttrValue::from_u32(value, 0),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}

fn image_dimension_setter(element: &Element, attr: Atom, value: u32) {
    // This setter is a bit weird: the IDL type is unsigned long, but it's parsed as
    // a dimension for rendering.
    let value = if value > UNSIGNED_LONG_MAX {
        0
    } else {
        value
    };
    let dim = LengthOrPercentageOrAuto::Length(Au::from_px(value as i32));
    let value = AttrValue::Dimension(DOMString::from(value.to_string()), dim);
    element.set_attribute(&attr, value);
}
