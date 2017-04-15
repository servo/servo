/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::{Au, AU_PER_PX};
use document_loader::{LoadType, LoadBlocker};
use dom::activation::Activatable;
use dom::attr::Attr;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DOMRectBinding::DOMRectBinding::DOMRectMethods;
use dom::bindings::codegen::Bindings::ElementBinding::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::HTMLImageElementBinding;
use dom::bindings::codegen::Bindings::HTMLImageElementBinding::HTMLImageElementMethods;
use dom::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{LayoutJS, MutNullableJS, Root};
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::DomObject;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{AttributeMutation, Element, RawLayoutElementHelpers};
use dom::element::{reflect_cross_origin_attribute, set_cross_origin_attribute};
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::htmlareaelement::HTMLAreaElement;
use dom::htmlelement::HTMLElement;
use dom::htmlformelement::{FormControl, HTMLFormElement};
use dom::htmlmapelement::HTMLMapElement;
use dom::mouseevent::MouseEvent;
use dom::node::{Node, NodeDamage, document_from_node, window_from_node};
use dom::values::UNSIGNED_LONG_MAX;
use dom::virtualmethods::VirtualMethods;
use dom::window::Window;
use dom_struct::dom_struct;
use euclid::point::Point2D;
use html5ever_atoms::LocalName;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use net_traits::{FetchResponseListener, FetchMetadata, NetworkError, FetchResponseMsg};
use net_traits::image::base::{Image, ImageMetadata};
use net_traits::image_cache::{CanRequestImages, ImageCache, ImageOrMetadataAvailable};
use net_traits::image_cache::{ImageResponder, ImageResponse, ImageState, PendingImageId};
use net_traits::image_cache::UsePlaceholder;
use net_traits::request::{RequestInit, Type as RequestType};
use network_listener::{NetworkListener, PreInvoke};
use num_traits::ToPrimitive;
use script_thread::{Runnable, ScriptThread};
use servo_url::ServoUrl;
use std::cell::{Cell, RefMut};
use std::default::Default;
use std::i32;
use std::sync::{Arc, Mutex};
use style::attr::{AttrValue, LengthOrPercentageOrAuto};
use task_source::TaskSource;

#[derive(Clone, Copy, JSTraceable, HeapSizeOf)]
#[allow(dead_code)]
enum State {
    Unavailable,
    PartiallyAvailable,
    CompletelyAvailable,
    Broken,
}
#[derive(JSTraceable, HeapSizeOf)]
#[must_root]
struct ImageRequest {
    state: State,
    parsed_url: Option<ServoUrl>,
    source_url: Option<DOMString>,
    blocker: Option<LoadBlocker>,
    #[ignore_heap_size_of = "Arc"]
    image: Option<Arc<Image>>,
    metadata: Option<ImageMetadata>,
}
#[dom_struct]
pub struct HTMLImageElement {
    htmlelement: HTMLElement,
    current_request: DOMRefCell<ImageRequest>,
    pending_request: DOMRefCell<ImageRequest>,
    form_owner: MutNullableJS<HTMLFormElement>,
    generation: Cell<u32>,
}

impl HTMLImageElement {
    pub fn get_url(&self) -> Option<ServoUrl> {
        self.current_request.borrow().parsed_url.clone()
    }
}

struct ImageResponseHandlerRunnable {
    element: Trusted<HTMLImageElement>,
    url: ServoUrl,
    image: ImageResponse,
    generation: u32,
}

impl ImageResponseHandlerRunnable {
    fn new(element: Trusted<HTMLImageElement>, url: ServoUrl, image: ImageResponse, generation: u32)
           -> ImageResponseHandlerRunnable {
        ImageResponseHandlerRunnable {
            element: element,
            url: url,
            image: image,
            generation: generation,
        }
    }
}

impl Runnable for ImageResponseHandlerRunnable {
    fn name(&self) -> &'static str { "ImageResponseHandlerRunnable" }

    fn handler(self: Box<Self>) {
        let element = self.element.root();
        // Ignore any image response for a previous request that has been discarded.
        if element.generation.get() == self.generation {
            element.process_image_response(&self.image, &self.url);
        }
    }
}

/// The context required for asynchronously loading an external image.
struct ImageContext {
    /// Reference to the script thread image cache.
    image_cache: Arc<ImageCache>,
    /// Indicates whether the request failed, and why
    status: Result<(), NetworkError>,
    /// The cache ID for this request.
    id: PendingImageId,
}

impl FetchResponseListener for ImageContext {
    fn process_request_body(&mut self) {}
    fn process_request_eof(&mut self) {}

    fn process_response(&mut self, metadata: Result<FetchMetadata, NetworkError>) {
        self.image_cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessResponse(metadata.clone()));

        let metadata = metadata.ok().map(|meta| {
            match meta {
                FetchMetadata::Unfiltered(m) => m,
                FetchMetadata::Filtered { unsafe_, .. } => unsafe_
            }
        });

        let status_code = metadata.as_ref().and_then(|m| {
            m.status.as_ref().map(|&(code, _)| code)
        }).unwrap_or(0);

        self.status = match status_code {
            0 => Err(NetworkError::Internal("No http status code received".to_owned())),
            200...299 => Ok(()), // HTTP ok status codes
            _ => Err(NetworkError::Internal(format!("HTTP error code {}", status_code)))
        };
    }

    fn process_response_chunk(&mut self, payload: Vec<u8>) {
        if self.status.is_ok() {
            self.image_cache.notify_pending_response(
                self.id,
                FetchResponseMsg::ProcessResponseChunk(payload));
        }
    }

    fn process_response_eof(&mut self, response: Result<(), NetworkError>) {
        self.image_cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessResponseEOF(response));
    }
}

impl PreInvoke for ImageContext {}

impl HTMLImageElement {
    /// Update the current image with a valid URL.
    fn update_image_with_url(&self, img_url: &ServoUrl) {
        fn add_cache_listener_for_element(image_cache: Arc<ImageCache>,
                                          id: PendingImageId,
                                          elem: &HTMLImageElement,
                                          url: ServoUrl) {
            let trusted_node = Trusted::new(elem);
            let (responder_sender, responder_receiver) = ipc::channel().unwrap();

            let window = window_from_node(elem);
            let task_source = window.networking_task_source();
            let wrapper = window.get_runnable_wrapper();
            let generation = elem.generation.get();
            ROUTER.add_route(responder_receiver.to_opaque(), box move |message| {
                debug!("Got image {:?}", message);
                // Return the image via a message to the script thread, which marks
                // the element as dirty and triggers a reflow.
                let runnable = ImageResponseHandlerRunnable::new(
                    trusted_node.clone(), url.clone(), message.to().unwrap(), generation);
                let _ = task_source.queue_with_wrapper(box runnable, &wrapper);
            });

            image_cache.add_listener(id, ImageResponder::new(responder_sender, id));
        }

        let window = window_from_node(self);
        let image_cache = window.image_cache();
        let response =
            image_cache.find_image_or_metadata(img_url.clone().into(),
                                               UsePlaceholder::Yes,
                                               CanRequestImages::Yes);
        match response {
            Ok(ImageOrMetadataAvailable::ImageAvailable(image)) => {
                self.process_image_response(&ImageResponse::Loaded(image), img_url);
            }

            Ok(ImageOrMetadataAvailable::MetadataAvailable(m)) => {
                self.process_image_response(&ImageResponse::MetadataLoaded(m), img_url);
            }

            Err(ImageState::Pending(id)) => {
                add_cache_listener_for_element(image_cache.clone(), id, self, img_url.clone());
            }

            Err(ImageState::LoadError) => {
                self.process_image_response(&ImageResponse::None, img_url);
            }

            Err(ImageState::NotRequested(id)) => {
                add_cache_listener_for_element(image_cache, id, self, img_url.clone());
                self.request_image(img_url, id);
            }
        }
    }

    fn request_image(&self, img_url: &ServoUrl, id: PendingImageId) {
        let document = document_from_node(self);
        let window = window_from_node(self);

        let context = Arc::new(Mutex::new(ImageContext {
            image_cache: window.image_cache(),
            status: Ok(()),
            id: id,
        }));

        let (action_sender, action_receiver) = ipc::channel().unwrap();
        let listener = NetworkListener {
            context: context,
            task_source: window.networking_task_source(),
            wrapper: Some(window.get_runnable_wrapper()),
        };
        ROUTER.add_route(action_receiver.to_opaque(), box move |message| {
            listener.notify_fetch(message.to().unwrap());
        });

        let request = RequestInit {
            url: img_url.clone(),
            origin: document.url().clone(),
            type_: RequestType::Image,
            pipeline_id: Some(document.global().pipeline_id()),
            .. RequestInit::default()
        };

        // This is a background load because the load blocker already fulfills the
        // purpose of delaying the document's load event.
        document.loader().fetch_async_background(request, action_sender);
    }

    fn request_for_url(&self, url: &ServoUrl) -> RefMut<ImageRequest> {
        let current_request = self.current_request.borrow_mut();
        let pending_request = self.pending_request.borrow_mut();
        let request = match (current_request.parsed_url.clone(),
                             pending_request.parsed_url.clone()) {
            (Some(request_url), None) => {
                assert!(request_url == *url);
                current_request
            },
            (None, Some(request_url)) => {
                assert!(request_url == *url);
                pending_request
            },
            (_, _) => unreachable!("There should be only one active request")
        };
        request
    }

    /// Step 14 of https://html.spec.whatwg.org/multipage/#update-the-image-data
    fn process_image_response(&self, image: &ImageResponse, url: &ServoUrl) {
        let (trigger_image_load, trigger_image_error) = match *image {
            ImageResponse::Loaded(ref image) | ImageResponse::PlaceholderLoaded(ref image) => {
                let mut image_request = self.request_for_url(url);
                image_request.state = State::CompletelyAvailable;
                image_request.image = Some(image.clone());
                image_request.metadata = Some(ImageMetadata { height: image.height, width: image.width });
                LoadBlocker::terminate(&mut image_request.blocker);
                (true, false)
            }
            ImageResponse::MetadataLoaded(ref meta) => {
                let mut image_request = self.request_for_url(url);
                image_request.state = State::PartiallyAvailable;
                image_request.image = None;
                image_request.metadata = Some(meta.clone());
                (false, false)
            }
            ImageResponse::None => {
                let mut image_request = self.request_for_url(url);
                image_request.state = State::Broken;
                image_request.image = None;
                image_request.metadata = None;
                LoadBlocker::terminate(&mut image_request.blocker);
                (false, true)
            }
        };

        // Mark the node dirty
        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);

        // Fire image.onload and loadend
        if trigger_image_load {
            self.upcast::<EventTarget>().fire_event(atom!("load"));
            self.upcast::<EventTarget>().fire_event(atom!("loadend"));
        }

        // Fire image.onerror
        if trigger_image_error {
            self.upcast::<EventTarget>().fire_event(atom!("error"));
            self.upcast::<EventTarget>().fire_event(atom!("loadend"));
        }

        // Trigger reflow
        let window = window_from_node(self);
        window.add_pending_reflow();
    }

    /// https://html.spec.whatwg.org/multipage/#abort-the-image-request
    fn abort_requests(&self, state: State) {
        let mut current_request = self.current_request.borrow_mut();
        LoadBlocker::terminate(&mut current_request.blocker);
        current_request.state = state;
        current_request.image = None;
        current_request.metadata = None;
        let mut pending_request = self.pending_request.borrow_mut();
        LoadBlocker::terminate(&mut pending_request.blocker);
        pending_request.state = state;
        pending_request.image = None;
        pending_request.metadata = None;
    }

    fn fire_progress_event(&self) {
        struct FireProgressEventTask {
            img: Trusted<HTMLImageElement>,
        }
        impl Runnable for FireProgressEventTask {
            fn handler(self: Box<Self>) {
                let img = self.img.root();
                img.upcast::<EventTarget>().fire_event(atom!("progress"));
            }
        }
        let task = box FireProgressEventTask {
            img: Trusted::new(self),
        };
        let document = document_from_node(self);
        let window = document.window();
        let task_source = window.dom_manipulation_task_source();
        let _ = task_source.queue(task, window.upcast());
    }

    /// https://html.spec.whatwg.org/multipage/#update-the-source-set
    fn update_source_set(&self) -> Vec<DOMString> {
        let elem = self.upcast::<Element>();
        // TODO: follow the algorithm
        vec![elem.get_string_attribute(&local_name!("src"))]
    }

    /// https://html.spec.whatwg.org/multipage/#select-an-image-source
    fn select_image_source(&self) -> Option<DOMString> {
        // TODO: select an image source from source set
        let source_set = self.update_source_set();
        if let Some(src) = source_set.first() {
            return Some(src.clone())
        }
        return None
    }

    /// Step 9.2 of https://html.spec.whatwg.org/multipage/#update-the-image-data
    fn set_current_request_url_to_none(&self) {
        struct SetUrlToNoneTask {
            img: Trusted<HTMLImageElement>,
        }
        impl Runnable for SetUrlToNoneTask {
            fn handler(self: Box<Self>) {
                // Step 9.2
                let img = self.img.root();
                img.current_request.borrow_mut().source_url = None;
                img.current_request.borrow_mut().parsed_url = None;
            }
        }

        let task = box SetUrlToNoneTask {
            img: Trusted::new(self),
        };
        let document = document_from_node(self);
        let window = document.window();
        let task_source = window.dom_manipulation_task_source();
        let _ = task_source.queue(task, window.upcast());
    }

    fn set_current_request_url_to_selected_fire_error_loadend(&self, src: DOMString) {
        struct SetUrlToSelectedTask {
            img: Trusted<HTMLImageElement>,
            src: String,
        }
        impl Runnable for SetUrlToSelectedTask {
            fn handler(self: Box<Self>) {
                // https://html.spec.whatwg.org/multipage/#update-the-image-data
                // Step 11, substep 5
                let img = self.img.root();
                img.current_request.borrow_mut().source_url = Some(self.src.into());
                img.upcast::<EventTarget>().fire_event(atom!("error"));
                img.upcast::<EventTarget>().fire_event(atom!("loadend"));
            }
        }
        let runnable = box SetUrlToSelectedTask {
            img: Trusted::new(self),
            src: src.into(),
        };
        let document = document_from_node(self);
        let window = document.window();
        let task = window.dom_manipulation_task_source();
        let _ = task.queue(runnable, window.upcast());
    }

    /// Step 12 of html.spec.whatwg.org/multipage/#update-the-image-data
    fn fetch_image(&self, url: &ServoUrl, src: &DOMString) {
        let mut current_request = self.current_request.borrow_mut();
        let mut pending_request = self.pending_request.borrow_mut();
        if let Some(pending_url) = pending_request.parsed_url.clone() {
            // Step 12.1
            if pending_url == *url {
                return
            }
            // Step 12.2, 12.3 abort pending request
            pending_request.image = None;
            pending_request.parsed_url = None;
            LoadBlocker::terminate(&mut pending_request.blocker);
        }
        // step 12.4, create a new "image_request"
        let mut image_request = match (current_request.parsed_url.clone(), current_request.state) {
            (Some(parsed_url), State::PartiallyAvailable) => {
                // Step 12.2
                if parsed_url == *url {
                    // TODO: queue a task to restart animation, if restart-animation is set
                    return
                }
                pending_request
            },
            (_, State::Broken) | (_, State::Unavailable) => {
                // Step 12.5
                LoadBlocker::terminate(&mut current_request.blocker);
                current_request
            },
            (_, _) => {
                // step 12.6
                pending_request
            },
        };
        image_request.parsed_url = Some(url.clone());
        image_request.source_url = Some(src.clone());
        image_request.image = None;
        image_request.metadata = None;
        let document = document_from_node(self);
        image_request.blocker = Some(LoadBlocker::new(&*document, LoadType::Image(url.clone())));
        // Step 12.10 associate this instance of algorithm with "image_request"
        // we do this via the url.
        self.update_image_with_url(&url);
    }

    /// Step 8-12 of html.spec.whatwg.org/multipage/#update-the-image-data
    fn update_the_image_data_sync_steps(&self) {
        let document = document_from_node(self);
        // Step 8
        let image_source = self.select_image_source();
        // Step 9
        if image_source.is_none() {
            self.abort_requests(State::Broken);
            self.set_current_request_url_to_none();
            let elem = self.upcast::<Element>();
            if elem.has_attribute(&local_name!("src")) {
                self.upcast::<EventTarget>().fire_event(atom!("error"));
            }
            return
        }
        // Step 10
        self.fire_progress_event();
        // Step 11
        let elem = self.upcast::<Element>();
        let src = elem.get_string_attribute(&local_name!("src"));
        let base_url = document.base_url();
        let parsed_url = base_url.join(&src);
        match parsed_url {
            Ok(url) => {
                 // Step 12
                self.fetch_image(&url, &src);
            },
            Err(_) => {
                // Step 11.1-11.5
                self.abort_requests(State::Broken);
                self.set_current_request_url_to_selected_fire_error_loadend(src);
            }
        }
    }

    fn has_parent_image(&self) -> bool {
        let node = self.upcast::<Node>();
        if let Some(ref parent) = node.GetParentNode() {
            if parent.is::<HTMLImageElement>() {
                return true
            }
        }
        return false
    }

    /// https://html.spec.whatwg.org/multipage/#update-the-image-data
    fn update_the_image_data(&self) {
        let document = document_from_node(self);
        let window = document.window();
        let elem = self.upcast::<Element>();
        let src = elem.get_string_attribute(&local_name!("src"));
        let base_url = document.base_url();
        if !document.is_active() {
            // Step 1 (if the document is inactive)
            // queue micro task to come back to this algorithm.
            // NOTE: no task source for micro task on Window...
        }
        // Step 2 abort if user-agent does not supports images
        // NOTE: how to check this?
        // self.abort_requests(State::Unavailable)

        // step 3, 4
        // TODO: take srcset into account
        if !src.is_empty() && !self.has_parent_image() {
            // TODO: take pixel density into account
            if let Ok(img_url) = base_url.join(&src) {
                // step 5, check the list of available images
                let image_cache = window.image_cache();
                let response = image_cache.find_image_or_metadata(img_url.clone().into(),
                                                                  UsePlaceholder::No,
                                                                  CanRequestImages::No);
                if let Ok(ImageOrMetadataAvailable::ImageAvailable(image)) = response {
                    // Step 5.3
                    let metadata = ImageMetadata { height: image.height, width: image.width };
                    let mut current_request = self.current_request.borrow_mut();
                    current_request.image = Some(image.clone());
                    current_request.metadata = Some(metadata);
                    current_request.state = State::CompletelyAvailable;
                    current_request.parsed_url = Some(img_url);
                    current_request.source_url = Some(src);
                    self.upcast::<EventTarget>().fire_event(atom!("load"));
                    return
                }
            }
        }
        // step 6, await a stable state.
        struct StableStateUpdateImageDataTask {
            elem: Trusted<HTMLImageElement>,
            generation: u32,
        }
        impl Runnable for StableStateUpdateImageDataTask {
            fn handler(self: Box<StableStateUpdateImageDataTask>) {
                let elem = self.elem.root();
                // Step 7, stop here if other instances of this algorithm have been scheduled
                if elem.generation.get() == self.generation {
                    elem.update_the_image_data_sync_steps();
                }
            }
        }
        self.generation.set(self.generation.get() + 1);
        let task = StableStateUpdateImageDataTask {
            elem: Trusted::new(self),
            generation: self.generation.get(),
        };
        ScriptThread::await_stable_state(task);
    }

    fn new_inherited(local_name: LocalName, prefix: Option<DOMString>, document: &Document) -> HTMLImageElement {
        HTMLImageElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            current_request: DOMRefCell::new(ImageRequest {
                state: State::Unavailable,
                parsed_url: None,
                source_url: None,
                image: None,
                metadata: None,
                blocker: None,
            }),
            pending_request: DOMRefCell::new(ImageRequest {
                state: State::Unavailable,
                parsed_url: None,
                source_url: None,
                image: None,
                metadata: None,
                blocker: None,
            }),
            form_owner: Default::default(),
            generation: Default::default(),
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

    pub fn Image(window: &Window,
                 width: Option<u32>,
                 height: Option<u32>) -> Fallible<Root<HTMLImageElement>> {
        let document = window.Document();
        let image = HTMLImageElement::new(local_name!("img"), None, &document);
        if let Some(w) = width {
            image.SetWidth(w);
        }
        if let Some(h) = height {
            image.SetHeight(h);
        }

        Ok(image)
    }
    pub fn areas(&self) -> Option<Vec<Root<HTMLAreaElement>>> {
        let elem = self.upcast::<Element>();
        let usemap_attr = match elem.get_attribute(&ns!(), &local_name!("usemap")) {
            Some(attr) => attr,
            None => return None,
        };

        let value = usemap_attr.value();
        let (first, last) = value.split_at(1);

        if first != "#" || last.len() == 0 {
            return None
        }

        let map = self.upcast::<Node>()
                      .following_siblings()
                      .filter_map(Root::downcast::<HTMLMapElement>)
                      .find(|n| n.upcast::<Element>().get_string_attribute(&LocalName::from("name")) == last);

        let elements: Vec<Root<HTMLAreaElement>> = map.unwrap().upcast::<Node>()
                      .children()
                      .filter_map(Root::downcast::<HTMLAreaElement>)
                      .collect();
        Some(elements)
    }
}

pub trait LayoutHTMLImageElementHelpers {
    #[allow(unsafe_code)]
    unsafe fn image(&self) -> Option<Arc<Image>>;

    #[allow(unsafe_code)]
    unsafe fn image_url(&self) -> Option<ServoUrl>;

    fn get_width(&self) -> LengthOrPercentageOrAuto;
    fn get_height(&self) -> LengthOrPercentageOrAuto;
}

impl LayoutHTMLImageElementHelpers for LayoutJS<HTMLImageElement> {
    #[allow(unsafe_code)]
    unsafe fn image(&self) -> Option<Arc<Image>> {
        (*self.unsafe_get()).current_request.borrow_for_layout().image.clone()
    }

    #[allow(unsafe_code)]
    unsafe fn image_url(&self) -> Option<ServoUrl> {
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
    fn GetCrossOrigin(&self) -> Option<DOMString> {
        reflect_cross_origin_attribute(self.upcast::<Element>())
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-crossOrigin
    fn SetCrossOrigin(&self, value: Option<DOMString>) {
        set_cross_origin_attribute(self.upcast::<Element>(), value);
    }

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
        match node.bounding_content_box() {
            Some(rect) => rect.size.width.to_px() as u32,
            None => self.NaturalWidth(),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-width
    fn SetWidth(&self, value: u32) {
        image_dimension_setter(self.upcast(), local_name!("width"), value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-height
    fn Height(&self) -> u32 {
        let node = self.upcast::<Node>();
        match node.bounding_content_box() {
            Some(rect) => rect.size.height.to_px() as u32,
            None => self.NaturalHeight(),
        }
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
        let elem = self.upcast::<Element>();
        // TODO: take srcset into account
        if !elem.has_attribute(&local_name!("src")) {
            return true
        }
        let src = elem.get_string_attribute(&local_name!("src"));
        if src.is_empty() {
            return true
        }
        let request = self.current_request.borrow();
        let request_state = request.state;
        match request_state {
            State::CompletelyAvailable | State::Broken => return true,
            State::PartiallyAvailable | State::Unavailable => {
                // NOTE: this is an attempt to comply with:
                // "(return true if)The final task that is queued by the networking task source,
                //  once the resource has been fetched, has been queued"
                return request.parsed_url.is_some()
            },
        }
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

    fn adopting_steps(&self, old_doc: &Document) {
        self.super_type().unwrap().adopting_steps(old_doc);
        self.update_the_image_data();
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &local_name!("src") => self.update_the_image_data(),
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

    fn handle_event(&self, event: &Event) {
       if event.type_() == atom!("click") {
           let area_elements = self.areas();
           let elements = if let Some(x) = area_elements {
               x
           } else {
               return
           };

           // Fetch click coordinates
           let mouse_event = if let Some(x) = event.downcast::<MouseEvent>() {
               x
           } else {
               return;
           };

           let point = Point2D::new(mouse_event.ClientX().to_f32().unwrap(),
                                    mouse_event.ClientY().to_f32().unwrap());

           // Walk HTMLAreaElements
           for element in elements {
               let shape = element.get_shape_from_coords();
               let p = Point2D::new(self.upcast::<Element>().GetBoundingClientRect().X() as f32,
                                    self.upcast::<Element>().GetBoundingClientRect().Y() as f32);

               let shp = if let Some(x) = shape {
                   x.absolute_coords(p)
               } else {
                   return
               };
               if shp.hit_test(point) {
                   element.activation_behavior(event, self.upcast());
                   return
               }
           }
       }
    }
}

impl FormControl for HTMLImageElement {
    fn form_owner(&self) -> Option<Root<HTMLFormElement>> {
        self.form_owner.get()
    }

    fn set_form_owner(&self, form: Option<&HTMLFormElement>) {
        self.form_owner.set(form);
    }

    fn to_element<'a>(&'a self) -> &'a Element {
        self.upcast::<Element>()
    }

    fn is_listed(&self) -> bool {
        false
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
