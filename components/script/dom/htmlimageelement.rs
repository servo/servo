/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::{Au, AU_PER_PX};
use dom::attr::Attr;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::HTMLImageElementBinding;
use dom::bindings::codegen::Bindings::HTMLImageElementBinding::HTMLImageElementMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{LayoutJS, Root};
use dom::bindings::refcounted::Trusted;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{AttributeMutation, Element, RawLayoutElementHelpers};
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeDamage, document_from_node, window_from_node};
use dom::values::UNSIGNED_LONG_MAX;
use dom::virtualmethods::VirtualMethods;
use html5ever_atoms::LocalName;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use net_traits::image::base::{Image, ImageMetadata};
use net_traits::image_cache_thread::{ImageResponder, ImageResponse};
use script_thread::Runnable;
use std::i32;
use std::sync::Arc;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};
use task_source::TaskSource;
use url::Url;

#[derive(JSTraceable, HeapSizeOf)]
#[allow(dead_code)]
enum State {
    Unavailable,
    PartiallyAvailable,
    CompletelyAvailable,
    Broken,
}
#[derive(JSTraceable, HeapSizeOf)]
struct ImageRequest {
    state: State,
    parsed_url: Option<Url>,
    source_url: Option<DOMString>,
    #[ignore_heap_size_of = "Arc"]
    image: Option<Arc<Image>>,
    metadata: Option<ImageMetadata>,
}
#[dom_struct]
pub struct HTMLImageElement {
    htmlelement: HTMLElement,
    current_request: DOMRefCell<ImageRequest>,
    pending_request: DOMRefCell<ImageRequest>,
}

impl HTMLImageElement {
    pub fn get_url(&self) -> Option<Url> {
        self.current_request.borrow().parsed_url.clone()
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
    fn name(&self) -> &'static str { "ImageResponseHandlerRunnable" }

    fn handler(self: Box<Self>) {
        // Update the image field
        let element = self.element.root();
        let (image, metadata, trigger_image_load, trigger_image_error) = match self.image {
            ImageResponse::Loaded(image) | ImageResponse::PlaceholderLoaded(image) => {
                (Some(image.clone()), Some(ImageMetadata { height: image.height, width: image.width } ), true, false)
            }
            ImageResponse::MetadataLoaded(meta) => {
                (None, Some(meta), false, false)
            }
            ImageResponse::None => (None, None, false, true)
        };
        element.current_request.borrow_mut().image = image;
        element.current_request.borrow_mut().metadata = metadata;

        // Mark the node dirty
        let document = document_from_node(&*element);
        element.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);

        // Fire image.onload
        if trigger_image_load {
            element.upcast::<EventTarget>().fire_event(atom!("load"));
        }

        // Fire image.onerror
        if trigger_image_error {
            element.upcast::<EventTarget>().fire_event(atom!("error"));
        }

        // Trigger reflow
        let window = window_from_node(&*document);
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
                self.current_request.borrow_mut().parsed_url = None;
                self.current_request.borrow_mut().source_url = None;
                self.current_request.borrow_mut().image = None;
            }
            Some((src, base_url)) => {
                let img_url = base_url.join(&src);
                if let Ok(img_url) = img_url {
                    self.current_request.borrow_mut().parsed_url = Some(img_url.clone());
                    self.current_request.borrow_mut().source_url = Some(src);

                    let trusted_node = Trusted::new(self);
                    let (responder_sender, responder_receiver) = ipc::channel().unwrap();
                    let task_source = window.networking_task_source();
                    let wrapper = window.get_runnable_wrapper();
                    ROUTER.add_route(responder_receiver.to_opaque(), box move |message| {
                        // Return the image via a message to the script thread, which marks the element
                        // as dirty and triggers a reflow.
                        let image_response = message.to().unwrap();
                        let runnable = box ImageResponseHandlerRunnable::new(
                            trusted_node.clone(), image_response);
                        let _ = task_source.queue_with_wrapper(runnable, &wrapper);
                    });

                    image_cache.request_image_and_metadata(img_url,
                                              window.image_cache_chan(),
                                              Some(ImageResponder::new(responder_sender)));
                } else {
                    // https://html.spec.whatwg.org/multipage/#update-the-image-data
                    // Step 11 (error substeps)
                    debug!("Failed to parse URL {} with base {}", src, base_url);
                    let mut req = self.current_request.borrow_mut();

                    // Substeps 1,2
                    req.image = None;
                    req.parsed_url = None;
                    req.state = State::Broken;
                    // todo: set pending request to null
                    // (pending requests aren't being used yet)


                    struct ImgParseErrorRunnable {
                        img: Trusted<HTMLImageElement>,
                        src: String,
                    }
                    impl Runnable for ImgParseErrorRunnable {
                        fn handler(self: Box<Self>) {
                            // https://html.spec.whatwg.org/multipage/#update-the-image-data
                            // Step 11, substep 5
                            let img = self.img.root();
                            img.current_request.borrow_mut().source_url = Some(self.src.into());
                            img.upcast::<EventTarget>().fire_event(atom!("error"));
                            img.upcast::<EventTarget>().fire_event(atom!("loadend"));
                        }
                    }

                    let runnable = box ImgParseErrorRunnable {
                        img: Trusted::new(self),
                        src: src.into(),
                    };
                    let task = window.dom_manipulation_task_source();
                    let _ = task.queue(runnable, window.upcast());
                }
            }
        }
    }
    fn new_inherited(local_name: LocalName, prefix: Option<DOMString>, document: &Document) -> HTMLImageElement {
        HTMLImageElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            current_request: DOMRefCell::new(ImageRequest {
                state: State::Unavailable,
                parsed_url: None,
                source_url: None,
                image: None,
                metadata: None
            }),
            pending_request: DOMRefCell::new(ImageRequest {
                state: State::Unavailable,
                parsed_url: None,
                source_url: None,
                image: None,
                metadata: None
            }),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLImageElement> {
        Node::reflect_node(box HTMLImageElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLImageElementBinding::Wrap)
    }

    pub fn Image(global: &GlobalScope,
                 width: Option<u32>,
                 height: Option<u32>) -> Fallible<Root<HTMLImageElement>> {
        let document = global.as_window().Document();
        let image = HTMLImageElement::new(local_name!("img"), None, &document);
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
        (*self.unsafe_get()).current_request.borrow_for_layout().image.clone()
    }

    #[allow(unsafe_code)]
    unsafe fn image_url(&self) -> Option<Url> {
        (*self.unsafe_get()).current_request.borrow_for_layout().parsed_url.clone()
    }

    #[allow(unsafe_code)]
    fn get_width(&self) -> LengthOrPercentageOrAuto {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &local_name!("width"))
                .map(AttrValue::as_dimension)
                .cloned()
                .unwrap_or(LengthOrPercentageOrAuto::Auto)
        }
    }

    #[allow(unsafe_code)]
    fn get_height(&self) -> LengthOrPercentageOrAuto {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &local_name!("height"))
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

    // https://html.spec.whatwg.org/multipage/#dom-img-crossOrigin
    make_enumerated_getter!(CrossOrigin, "crossorigin", "anonymous", "use-credentials");
    // https://html.spec.whatwg.org/multipage/#dom-img-crossOrigin
    make_setter!(SetCrossOrigin, "crossorigin");

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
        let rect = node.bounding_content_box();
        rect.size.width.to_px() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-width
    fn SetWidth(&self, value: u32) {
        image_dimension_setter(self.upcast(), local_name!("width"), value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-height
    fn Height(&self) -> u32 {
        let node = self.upcast::<Node>();
        let rect = node.bounding_content_box();
        rect.size.height.to_px() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-height
    fn SetHeight(&self, value: u32) {
        image_dimension_setter(self.upcast(), local_name!("height"), value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-naturalwidth
    fn NaturalWidth(&self) -> u32 {
        let ref metadata = self.current_request.borrow().metadata;

        match *metadata {
            Some(ref metadata) => metadata.width,
            None => 0,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-naturalheight
    fn NaturalHeight(&self) -> u32 {
        let ref metadata = self.current_request.borrow().metadata;

        match *metadata {
            Some(ref metadata) => metadata.height,
            None => 0,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-complete
    fn Complete(&self) -> bool {
        let ref image = self.current_request.borrow().image;
        image.is_some()
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-currentsrc
    fn CurrentSrc(&self) -> DOMString {
        let ref url = self.current_request.borrow().source_url;
        match *url {
            Some(ref url) => url.clone(),
            None => DOMString::from(""),
        }
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
            &local_name!("src") => {
                self.update_image(mutation.new_value(attr).map(|value| {
                    // FIXME(ajeffrey): convert directly from AttrValue to DOMString
                    (DOMString::from(&**value), document_from_node(self).base_url())
                }));
            },
            _ => {},
        }
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("name") => AttrValue::from_atomic(value.into()),
            &local_name!("width") | &local_name!("height") => AttrValue::from_dimension(value.into()),
            &local_name!("hspace") | &local_name!("vspace") => AttrValue::from_u32(value.into(), 0),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }
}

fn image_dimension_setter(element: &Element, attr: LocalName, value: u32) {
    // This setter is a bit weird: the IDL type is unsigned long, but it's parsed as
    // a dimension for rendering.
    let value = if value > UNSIGNED_LONG_MAX {
        0
    } else {
        value
    };

    // FIXME: There are probably quite a few more cases of this. This is the
    // only overflow that was hitting on automation, but we should consider what
    // to do in the general case case.
    //
    // See <https://github.com/servo/app_units/issues/22>
    let pixel_value = if value > (i32::MAX / AU_PER_PX) as u32 {
        0
    } else {
        value
    };

    let dim = LengthOrPercentageOrAuto::Length(Au::from_px(pixel_value as i32));
    let value = AttrValue::Dimension(value.to_string(), dim);
    element.set_attribute(&attr, value);
}
