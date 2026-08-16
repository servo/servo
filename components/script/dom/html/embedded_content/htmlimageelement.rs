/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::mem;
use std::rc::Rc;
use std::sync::Arc;

use dom_struct::dom_struct;
use euclid::default::Point2D;
use html5ever::{LocalName, Prefix, QualName, local_name, ns};
use js::context::{JSContext, NoGC};
use js::rust::HandleObject;
use mime::{self, Mime};
use net_traits::image_cache::{
    Image, ImageCache, ImageCacheResult, ImageLoadListener, ImageOrMetadataAvailable,
    ImageResponse, PendingImageId,
};
use net_traits::request::{CorsSettings, Destination, Initiator, RequestId};
use net_traits::{
    FetchMetadata, FetchResponseMsg, NetworkError, ReferrerPolicy, ResourceFetchTiming,
};
use num_traits::ToPrimitive;
use pixels::{CorsStatus, ImageMetadata, Snapshot};
use script_bindings::cell::DomRefCell;
use servo_url::ServoUrl;
use servo_url::origin::MutableOrigin;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};

use crate::dom::activation::Activatable;
use crate::dom::bindings::codegen::Bindings::DOMRectBinding::DOMRect_Binding::DOMRectMethods;
use crate::dom::bindings::codegen::Bindings::ElementBinding::Element_Binding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLImageElementBinding::HTMLImageElementMethods;
use crate::dom::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot, LayoutDom, MutNullableDom, ToLayoutOptional};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::csp::{GlobalCspReporting, Violation};
use crate::dom::document::Document;
use crate::dom::element::attributes::storage::AttrRef;
use crate::dom::element::{
    AttributeMutation, CustomElementCreationMode, Element, ElementCreator,
    cors_setting_for_element, referrer_policy_for_element, reflect_cross_origin_attribute,
    reflect_referrer_policy_attribute, set_cross_origin_attribute,
};
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlareaelement::HTMLAreaElement;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlformelement::{FormControl, HTMLFormElement};
use crate::dom::html::htmlmapelement::HTMLMapElement;
use crate::dom::html::htmlpictureelement::HTMLPictureElement;
use crate::dom::iterators::ShadowIncluding;
use crate::dom::mouseevent::MouseEvent;
use crate::dom::node::virtualmethods::VirtualMethods;
use crate::dom::node::{BindContext, MoveContext, Node, NodeDamage, NodeTraits, UnbindContext};
use crate::dom::performance::performanceresourcetiming::InitiatorType;
use crate::dom::promise::Promise;
use crate::dom::srcset::SourceSet;
use crate::dom::window::Window;
use crate::event_loop::document_loader::{LoadBlocker, LoadType};
use crate::event_loop::script_thread::ScriptThread;
use crate::fetch::fetch::{RequestWithGlobalScope, create_a_potential_cors_request};
use crate::fetch::network_listener::{self, FetchResponseListener, ResourceTimingListener};
use crate::microtask::MicrotaskRunnable;
use crate::realms::enter_auto_realm;

/// <https://html.spec.whatwg.org/multipage/#img-req-state>
#[derive(Clone, Copy, JSTraceable, MallocSizeOf)]
enum State {
    Unavailable,
    PartiallyAvailable,
    CompletelyAvailable,
    Broken,
}

#[derive(Clone, Copy, JSTraceable, MallocSizeOf)]
enum ImageRequestPhase {
    Pending,
    Current,
}

/// <https://html.spec.whatwg.org/multipage/#image-request>
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct ImageRequest {
    state: State,
    #[no_trace]
    parsed_url: Option<ServoUrl>,
    source_url: Option<USVString>,
    blocker: DomRefCell<Option<LoadBlocker>>,
    #[no_trace]
    image: Option<Image>,
    #[no_trace]
    metadata: Option<ImageMetadata>,
    #[no_trace]
    final_url: Option<ServoUrl>,
    current_pixel_density: Option<f64>,
}

#[dom_struct]
pub(crate) struct HTMLImageElement {
    htmlelement: HTMLElement,
    image_request: Cell<ImageRequestPhase>,
    current_request: DomRefCell<ImageRequest>,
    pending_request: DomRefCell<ImageRequest>,
    form_owner: MutNullableDom<HTMLFormElement>,
    generation: Cell<u32>,
    source_set: DomRefCell<SourceSet>,
    /// <https://html.spec.whatwg.org/multipage/#concept-img-dimension-attribute-source>
    /// Always non-null after construction.
    dimension_attribute_source: MutNullableDom<Element>,
    /// <https://html.spec.whatwg.org/multipage/#last-selected-source>
    last_selected_source: DomRefCell<Option<USVString>>,
    #[conditional_malloc_size_of]
    image_decode_promises: DomRefCell<Vec<Rc<Promise>>>,
    /// Line number this element was created on
    line_number: u64,
}

impl HTMLImageElement {
    // https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument
    pub(crate) fn is_usable(&self) -> Fallible<bool> {
        // If image has an intrinsic width or intrinsic height (or both) equal to zero, then return bad.
        if let Some(image) = &self.current_request.borrow().image {
            let intrinsic_size = image.metadata();
            if intrinsic_size.width == 0 || intrinsic_size.height == 0 {
                return Ok(false);
            }
        }

        match self.current_request.borrow().state {
            // If image's current request's state is broken, then throw an "InvalidStateError" DOMException.
            State::Broken => Err(Error::InvalidState(Some(
                "Image element's state is broken".into(),
            ))),
            State::CompletelyAvailable => Ok(true),
            // If image is not fully decodable, then return bad.
            State::PartiallyAvailable | State::Unavailable => Ok(false),
        }
    }

    pub(crate) fn image_data(&self) -> Option<Image> {
        self.current_request.borrow().image.clone()
    }

    /// Gets the copy of the raster image data.
    pub(crate) fn get_raster_image_data(&self) -> Option<Snapshot> {
        let Some(raster_image) = self.image_data()?.as_raster_image() else {
            warn!("Vector image is not supported as raster image source");
            return None;
        };
        Some(raster_image.as_snapshot())
    }
}

/// The context required for asynchronously loading an external image.
struct ImageContext {
    /// Reference to the script thread image cache.
    image_cache: Arc<dyn ImageCache>,
    /// Indicates whether the request failed, and why
    status: Result<(), NetworkError>,
    /// The cache ID for this request.
    id: PendingImageId,
    /// Used to mark abort
    aborted: bool,
    /// The document associated with this request
    doc: Trusted<Document>,
    url: ServoUrl,
    element: Trusted<HTMLImageElement>,
}

impl FetchResponseListener for ImageContext {
    fn should_invoke(&self) -> bool {
        !self.aborted
    }

    fn process_request_body(&mut self, _: RequestId) {}

    fn process_response(
        &mut self,
        _: &mut js::context::JSContext,
        request_id: RequestId,
        metadata: Result<FetchMetadata, NetworkError>,
    ) {
        debug!("got {:?} for {:?}", metadata.as_ref().map(|_| ()), self.url);
        self.image_cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessResponse(request_id, metadata.clone()),
        );

        let metadata = metadata.ok().map(|meta| match meta {
            FetchMetadata::Unfiltered(m) => m,
            FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
        });

        // Step 14.5 of https://html.spec.whatwg.org/multipage/#img-environment-changes
        if let Some(metadata) = metadata.as_ref() &&
            let Some(ref content_type) = metadata.content_type
        {
            let mime: Mime = content_type.clone().into_inner().into();
            if mime.type_() == mime::MULTIPART && mime.subtype().as_str() == "x-mixed-replace" {
                self.aborted = true;
            }
        }

        // The HTTP status code is ignored here. Ok NetworkError is treated
        // as real error
        self.status = match metadata.as_ref().map(|m| m.status.clone()) {
            None => Err(NetworkError::ResourceLoadError(
                "No http status code received".to_owned(),
            )),
            Some(_) => Ok(()),
        };
    }

    fn process_response_chunk(
        &mut self,
        _: &mut js::context::JSContext,
        request_id: RequestId,
        payload: Vec<u8>,
    ) {
        if self.status.is_ok() {
            self.image_cache.notify_pending_response(
                self.id,
                FetchResponseMsg::ProcessResponseChunk(request_id, payload.into()),
            );
        }
    }

    fn process_response_eof(
        self,
        cx: &mut js::context::JSContext,
        request_id: RequestId,
        response: Result<(), NetworkError>,
        timing: ResourceFetchTiming,
    ) {
        self.image_cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessResponseEOF(request_id, response.clone(), timing.clone()),
        );
        network_listener::submit_timing(cx, &self, &response, &timing);
    }

    fn process_csp_violations(
        &mut self,
        cx: &mut js::context::JSContext,
        _request_id: RequestId,
        violations: Vec<Violation>,
    ) {
        let global = &self.resource_timing_global();
        let elem = self.element.root();
        let source_position = elem
            .upcast::<Element>()
            .compute_source_position(elem.line_number as u32);
        global.report_csp_violations(cx, violations, None, Some(source_position));
    }

    fn process_content_length(&mut self, request_id: RequestId, size: usize) {
        self.image_cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessContentLength(request_id, size),
        );
    }
}

impl ResourceTimingListener for ImageContext {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        (
            InitiatorType::LocalName("img".to_string()),
            self.url.clone(),
        )
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.doc.root().global()
    }
}

#[expect(non_snake_case)]
impl HTMLImageElement {
    /// Update the current image with a valid URL.
    fn fetch_image(&self, img_url: &ServoUrl, cx: &mut js::context::JSContext) {
        let window = self.owner_window();

        let cache_result = window.image_cache().get_cached_image_status(
            img_url.clone(),
            window.origin().immutable().clone(),
            cors_setting_for_element(self.upcast()),
        );

        match cache_result {
            ImageCacheResult::Available(ImageOrMetadataAvailable::ImageAvailable {
                image,
                url,
            }) => self.process_image_response(ImageResponse::Loaded(image, url), cx),
            ImageCacheResult::Available(ImageOrMetadataAvailable::MetadataAvailable(
                metadata,
                id,
            )) => {
                self.process_image_response(ImageResponse::MetadataLoaded(metadata), cx);
                self.register_image_cache_callback(id, ChangeType::Element);
            },
            ImageCacheResult::Pending(id) => {
                self.register_image_cache_callback(id, ChangeType::Element);
            },
            ImageCacheResult::ReadyForRequest(id) => {
                self.fetch_request(img_url, id);
                self.register_image_cache_callback(id, ChangeType::Element);
            },
            ImageCacheResult::FailedToLoadOrDecode => {
                self.process_image_response(ImageResponse::FailedToLoadOrDecode, cx)
            },
        };
    }

    fn register_image_cache_callback(&self, id: PendingImageId, change_type: ChangeType) {
        let trusted_node = Trusted::new(self);
        let generation = self.generation_id();
        let window = self.owner_window();
        let callback = window.register_image_cache_listener(id, move |response, _| {
            let trusted_node = trusted_node.clone();
            let window = trusted_node.root().owner_window();
            let callback_type = change_type.clone();

            window
                .as_global_scope()
                .task_manager()
                .networking_task_source()
                .queue(task!(process_image_response: move |cx| {
                let element = trusted_node.root();

                // Ignore any image response for a previous request that has been discarded.
                if generation != element.generation_id() {
                    return;
                }

                match callback_type {
                    ChangeType::Element => {
                        element.process_image_response(response.response, cx);
                    }
                    ChangeType::Environment { selected_source, selected_pixel_density } => {
                        element.process_image_response_for_environment_change(
                            response.response, selected_source, generation, selected_pixel_density, cx
                        );
                    }
                }
            }));
        });

        window.image_cache().add_listener(ImageLoadListener::new(
            callback,
            window.pipeline_id(),
            id,
        ));
    }

    fn fetch_request(&self, img_url: &ServoUrl, id: PendingImageId) {
        let document = self.owner_document();
        let window = self.owner_window();

        let context = ImageContext {
            image_cache: window.image_cache(),
            status: Ok(()),
            id,
            aborted: false,
            doc: Trusted::new(&document),
            element: Trusted::new(self),
            url: img_url.clone(),
        };

        // https://html.spec.whatwg.org/multipage/#update-the-image-data steps 17-20
        // This function is also used to prefetch an image in `script::dom::servoparser::prefetch`.
        let global = document.global();
        let mut request = create_a_potential_cors_request(
            Some(window.webview_id()),
            img_url.clone(),
            Destination::Image,
            cors_setting_for_element(self.upcast()),
            None,
            global.get_referrer(),
        )
        .with_global_scope(&global)
        .referrer_policy(referrer_policy_for_element(self.upcast()));

        if self.uses_srcset_or_picture() {
            request = request.initiator(Initiator::ImageSet);
        }

        // This is a background load because the load blocker already fulfills the
        // purpose of delaying the document's load event.
        document.fetch_background(request, context);
    }

    // Steps common to when an image has been loaded.
    fn handle_loaded_image(&self, image: Image, url: ServoUrl, cx: &mut js::context::JSContext) {
        self.current_request.borrow_mut().metadata = Some(image.metadata());
        self.current_request.borrow_mut().final_url = Some(url);
        self.current_request.borrow_mut().image = Some(image);
        self.current_request.borrow_mut().state = State::CompletelyAvailable;
        LoadBlocker::terminate(&self.current_request.borrow().blocker, cx);
        // Mark the node dirty
        self.upcast::<Node>().dirty(cx.no_gc(), NodeDamage::Other);
        self.resolve_image_decode_promises();
    }

    /// <https://html.spec.whatwg.org/multipage/#update-the-image-data>
    fn process_image_response(&self, image: ImageResponse, cx: &mut js::context::JSContext) {
        // Step 27. As soon as possible, jump to the first applicable entry from the following list:

        // TODO => "If the resource type is multipart/x-mixed-replace"

        // => "If the resource type and data corresponds to a supported image format ...""
        let (trigger_image_load, trigger_image_error) = match (image, self.image_request.get()) {
            (ImageResponse::Loaded(image, url), ImageRequestPhase::Current) => {
                self.handle_loaded_image(image, url, cx);
                (true, false)
            },
            (ImageResponse::Loaded(image, url), ImageRequestPhase::Pending) => {
                self.abort_request(State::Unavailable, ImageRequestPhase::Pending, cx);
                self.image_request.set(ImageRequestPhase::Current);
                self.handle_loaded_image(image, url, cx);
                (true, false)
            },
            (ImageResponse::MetadataLoaded(meta), ImageRequestPhase::Current) => {
                // Otherwise, if the user agent is able to determine image request's image's width
                // and height, and image request is the current request, prepare image request for
                // presentation given the img element and set image request's state to partially
                // available.
                self.current_request.borrow_mut().state = State::PartiallyAvailable;
                self.current_request.borrow_mut().metadata = Some(meta);
                (false, false)
            },
            (ImageResponse::MetadataLoaded(_), ImageRequestPhase::Pending) => {
                // If the user agent is able to determine image request's image's width and height,
                // and image request is the pending request, set image request's state to partially
                // available.
                self.pending_request.borrow_mut().state = State::PartiallyAvailable;
                (false, false)
            },
            (ImageResponse::FailedToLoadOrDecode, ImageRequestPhase::Current) => {
                // Otherwise, if the user agent is able to determine that image request's image is
                // corrupted in some fatal way such that the image dimensions cannot be obtained,
                // and image request is the current request:

                // Step 1. Abort the image request for image request.
                self.abort_request(State::Broken, ImageRequestPhase::Current, cx);

                self.load_broken_image_icon(cx.no_gc());

                // Step 2. If maybe omit events is not set or previousURL is not equal to urlString,
                // then fire an event named error at the img element.
                // TODO: Add missing `maybe omit events` flag and previousURL.
                (false, true)
            },
            (ImageResponse::FailedToLoadOrDecode, ImageRequestPhase::Pending) => {
                // Otherwise, if the user agent is able to determine that image request's image is
                // corrupted in some fatal way such that the image dimensions cannot be obtained,
                // and image request is the pending request:

                // Step 1. Abort the image request for the current request and the pending request.
                self.abort_request(State::Broken, ImageRequestPhase::Current, cx);
                self.abort_request(State::Unavailable, ImageRequestPhase::Pending, cx);

                // Step 2. Upgrade the pending request to the current request.
                mem::swap(
                    &mut *self.current_request.borrow_mut(),
                    &mut *self.pending_request.borrow_mut(),
                );
                self.image_request.set(ImageRequestPhase::Current);

                // Step 3. Set the current request's state to broken.
                self.current_request.borrow_mut().state = State::Broken;

                self.load_broken_image_icon(cx.no_gc());

                // Step 4. Fire an event named error at the img element.
                (false, true)
            },
        };

        // Fire image.onload and loadend
        if trigger_image_load {
            // TODO: https://html.spec.whatwg.org/multipage/#fire-a-progress-event-or-event
            self.upcast::<EventTarget>().fire_event(cx, atom!("load"));
            self.upcast::<EventTarget>()
                .fire_event(cx, atom!("loadend"));
        }

        // Fire image.onerror
        if trigger_image_error {
            self.upcast::<EventTarget>().fire_event(cx, atom!("error"));
            self.upcast::<EventTarget>()
                .fire_event(cx, atom!("loadend"));
        }
    }

    /// The response part of
    /// <https://html.spec.whatwg.org/multipage/#reacting-to-environment-changes>.
    fn process_image_response_for_environment_change(
        &self,
        image: ImageResponse,
        selected_source: USVString,
        generation: u32,
        selected_pixel_density: f64,
        cx: &mut js::context::JSContext,
    ) {
        match image {
            ImageResponse::Loaded(image, url) => {
                self.pending_request.borrow_mut().metadata = Some(image.metadata());
                self.pending_request.borrow_mut().final_url = Some(url);
                self.pending_request.borrow_mut().image = Some(image);
                self.finish_reacting_to_environment_change(
                    selected_source,
                    generation,
                    selected_pixel_density,
                );
            },
            ImageResponse::FailedToLoadOrDecode => {
                // > Step 15.6: If response's unsafe response is a network error or if the
                // > image format is unsupported (as determined by applying the image
                // > sniffing rules, again as mentioned earlier), or if the user agent is
                // > able to determine that image request's image is corrupted in some fatal
                // > way such that the image dimensions cannot be obtained, or if the
                // > resource type is multipart/x-mixed-replace, then set the pending
                // > request to null and abort these steps.
                self.abort_request(State::Unavailable, ImageRequestPhase::Pending, cx);
            },
            ImageResponse::MetadataLoaded(meta) => {
                self.pending_request.borrow_mut().metadata = Some(meta);
            },
        };
    }

    /// <https://html.spec.whatwg.org/multipage/#abort-the-image-request>
    fn abort_request(
        &self,
        state: State,
        request: ImageRequestPhase,
        cx: &mut js::context::JSContext,
    ) {
        let request = match request {
            ImageRequestPhase::Current => &self.current_request,
            ImageRequestPhase::Pending => &self.pending_request,
        };
        LoadBlocker::terminate(&request.borrow().blocker, cx);
        let mut request = request.safe_borrow_mut(cx);
        request.state = state;
        request.image = None;
        request.metadata = None;
        request.current_pixel_density = None;

        if matches!(state, State::Broken) {
            self.reject_image_decode_promises();
        } else if matches!(state, State::CompletelyAvailable) {
            self.resolve_image_decode_promises();
        }
    }

    fn init_image_request(
        &self,
        request: &DomRefCell<ImageRequest>,
        url: &ServoUrl,
        src: &USVString,
        cx: &mut js::context::JSContext,
    ) {
        {
            let mut request = request.borrow_mut();
            request.parsed_url = Some(url.clone());
            request.source_url = Some(src.clone());
            request.image = None;
            request.metadata = None;
        }
        let document = self.owner_document();
        LoadBlocker::terminate(&request.borrow().blocker, cx);
        *request.borrow_mut().blocker.borrow_mut() =
            Some(LoadBlocker::new(&document, LoadType::Image(url.clone())));
    }

    /// <https://html.spec.whatwg.org/multipage/#update-the-image-data>
    fn prepare_image_request(
        &self,
        selected_source: &USVString,
        selected_pixel_density: f64,
        image_url: &ServoUrl,
        cx: &mut js::context::JSContext,
    ) {
        match self.image_request.get() {
            ImageRequestPhase::Pending => {
                // Step 14. If the pending request is not null and urlString is the same as the
                // pending request's current URL, then return.
                if self
                    .pending_request
                    .borrow()
                    .parsed_url
                    .as_ref()
                    .is_some_and(|parsed_url| *parsed_url == *image_url)
                {
                    return;
                }
            },
            ImageRequestPhase::Current => {
                // Step 16. Abort the image request for the pending request.
                self.abort_request(State::Unavailable, ImageRequestPhase::Pending, cx);

                // Step 17. Set image request to a new image request whose current URL is urlString.
                let (current_request_url, current_request_state) = {
                    let current_request = self.current_request.borrow();
                    (current_request.parsed_url.clone(), current_request.state)
                };

                match (current_request_url, current_request_state) {
                    (Some(parsed_url), State::PartiallyAvailable) => {
                        // Step 15. If urlString is the same as the current request's current URL
                        // and the current request's state is partially available, then abort the
                        // image request for the pending request, queue an element task on the DOM
                        // manipulation task source given the img element to restart the animation
                        // if restart animation is set, and return.
                        if parsed_url == *image_url {
                            // TODO: queue a task to restart animation, if restart-animation is set
                            return;
                        }

                        // Step 18. If the current request's state is unavailable or broken, then
                        // set the current request to image request. Otherwise, set the pending
                        // request to image request.
                        self.image_request.set(ImageRequestPhase::Pending);
                        self.init_image_request(
                            &self.pending_request,
                            image_url,
                            selected_source,
                            cx,
                        );
                        self.pending_request.borrow_mut().current_pixel_density =
                            Some(selected_pixel_density);
                    },
                    (_, State::Broken) | (_, State::Unavailable) => {
                        // Step 18. If the current request's state is unavailable or broken, then
                        // set the current request to image request. Otherwise, set the pending
                        // request to image request.
                        self.init_image_request(
                            &self.current_request,
                            image_url,
                            selected_source,
                            cx,
                        );
                        self.current_request.borrow_mut().current_pixel_density =
                            Some(selected_pixel_density);
                        self.reject_image_decode_promises();
                    },
                    (_, _) => {
                        // Step 18. If the current request's state is unavailable or broken, then
                        // set the current request to image request. Otherwise, set the pending
                        // request to image request.
                        self.image_request.set(ImageRequestPhase::Pending);
                        self.init_image_request(
                            &self.pending_request,
                            image_url,
                            selected_source,
                            cx,
                        );
                        self.pending_request.borrow_mut().current_pixel_density =
                            Some(selected_pixel_density);
                    },
                }
            },
        }

        self.fetch_image(image_url, cx);
    }

    /// <https://html.spec.whatwg.org/multipage/#update-the-image-data>
    fn update_the_image_data_sync_steps(&self, cx: &mut js::context::JSContext) {
        // Step 10. Let selected source and selected pixel density be the URL and pixel density that
        // results from selecting an image source, respectively.
        let Some((selected_source, selected_pixel_density)) = self
            .source_set
            .borrow_mut()
            .select_image_source(self.upcast::<Element>())
        else {
            // Step 11. If selected source is null, then:

            // Step 11.1. Set the current request's state to broken, abort the image request for the
            // current request and the pending request, and set the pending request to null.
            self.abort_request(State::Broken, ImageRequestPhase::Current, cx);
            self.abort_request(State::Unavailable, ImageRequestPhase::Pending, cx);
            self.image_request.set(ImageRequestPhase::Current);

            // Step 11.2. Queue an element task on the DOM manipulation task source given the img
            // element and the following steps:
            let this = Trusted::new(self);

            self.owner_global().task_manager().dom_manipulation_task_source().queue(
                task!(image_null_source_error: move |cx| {
                    let this = this.root();

                    // Step 11.2.1. Change the current request's current URL to the empty string.
                    {
                        let mut current_request =
                            this.current_request.borrow_mut();
                        current_request.source_url = None;
                        current_request.parsed_url = None;
                    }

                    // Step 11.2.2. If all of the following are true:
                    // the element has a src attribute or it uses srcset or picture; and
                    // maybe omit events is not set or previousURL is not the empty string,
                    // then fire an event named error at the img element.
                    // TODO: Add missing `maybe omit events` flag and previousURL.
                    let has_src_attribute = this.upcast::<Element>().has_attribute(&local_name!("src"));

                    if has_src_attribute || this.uses_srcset_or_picture() {
                        this.upcast::<EventTarget>().fire_event(cx, atom!("error"));
                    }
                }));

            // Step 11.2.3. Return.
            return;
        };

        // Step 12. Let urlString be the result of encoding-parsing-and-serializing a URL given
        // selected source, relative to the element's node document.
        let Ok(image_url) = self.owner_document().base_url().join(&selected_source) else {
            // Step 13. If urlString is failure, then:

            // Step 13.1. Abort the image request for the current request and the pending request.
            // Step 13.2. Set the current request's state to broken.
            self.abort_request(State::Broken, ImageRequestPhase::Current, cx);
            self.abort_request(State::Unavailable, ImageRequestPhase::Pending, cx);

            // Step 13.3. Set the pending request to null.
            self.image_request.set(ImageRequestPhase::Current);

            // Step 13.4. Queue an element task on the DOM manipulation task source given the img
            // element and the following steps:
            let this = Trusted::new(self);

            self.owner_global()
                .task_manager()
                .dom_manipulation_task_source()
                .queue(task!(image_selected_source_error: move |cx| {
                    let this = this.root();

                    // Step 13.4.1. Change the current request's current URL to selected source.
                    {
                        let mut current_request =
                            this.current_request.borrow_mut();
                        current_request.source_url = Some(selected_source);
                        current_request.parsed_url = None;
                    }

                    // Step 13.4.2. If maybe omit events is not set or previousURL is not equal to
                    // selected source, then fire an event named error at the img element.
                    // TODO: Add missing `maybe omit events` flag and previousURL.
                    this.upcast::<EventTarget>().fire_event(cx, atom!("error"));
                }));

            // Step 13.5. Return.
            return;
        };

        self.prepare_image_request(&selected_source, selected_pixel_density, &image_url, cx);
    }

    /// <https://html.spec.whatwg.org/multipage/#update-the-image-data>
    pub(crate) fn update_the_image_data(&self, cx: &mut js::context::JSContext) {
        // Cancel any outstanding tasks that were queued before.
        self.generation.set(self.generation.get() + 1);

        // Step 1. If the element's node document is not fully active, then:
        if !self.owner_document().is_active() {
            // TODO Step 1.1. Continue running this algorithm in parallel.
            // TODO Step 1.2. Wait until the element's node document is fully active.
            // TODO Step 1.3. If another instance of this algorithm for this img element was started after
            // this instance (even if it aborted and is no longer running), then return.
            // TODO Step 1.4. Queue a microtask to continue this algorithm.
        }

        // Step 2. If the user agent cannot support images, or its support for images has been
        // disabled, then abort the image request for the current request and the pending request,
        // set the current request's state to unavailable, set the pending request to null, and
        // return.
        // Nothing specific to be done here since the user agent supports image processing.

        // Always first set the current request to unavailable, ensuring img.complete is false.
        // <https://html.spec.whatwg.org/multipage/#when-to-obtain-images>
        self.current_request.borrow_mut().state = State::Unavailable;

        // TODO Step 3. Let previousURL be the current request's current URL.

        // Step 4. Let selected source be null and selected pixel density be undefined.
        let mut selected_source = None;
        let mut selected_pixel_density = None;

        // Step 5. If the element does not use srcset or picture and it has a src attribute
        // specified whose value is not the empty string, then set selected source to the value of
        // the element's src attribute and set selected pixel density to 1.0.
        let src = self
            .upcast::<Element>()
            .get_string_attribute(&local_name!("src"));

        if !self.uses_srcset_or_picture() && !src.is_empty() {
            selected_source = Some(USVString(String::from(src)));
            selected_pixel_density = Some(1_f64);
        };

        // Step 6. Set the element's last selected source to selected source.
        self.last_selected_source
            .borrow_mut()
            .clone_from(&selected_source);

        // Step 7. If selected source is not null, then:
        if let Some(selected_source) = selected_source {
            // Step 7.1. Let urlString be the result of encoding-parsing-and-serializing a URL given
            // selected source, relative to the element's node document.
            // Step 7.2. If urlString is failure, then abort this inner set of steps.
            if let Ok(image_url) = self.owner_document().base_url().join(&selected_source) {
                // Step 7.3. Let key be a tuple consisting of urlString, the img element's
                // crossorigin attribute's mode, and, if that mode is not No CORS, the node
                // document's origin.
                let window = self.owner_window();
                let response = window.image_cache().get_image(
                    image_url.clone(),
                    window.origin().immutable().clone(),
                    cors_setting_for_element(self.upcast()),
                );

                // Step 7.4. If the list of available images contains an entry for key, then:
                if let Some(image) = response {
                    // TODO Step 7.4.1. Set the ignore higher-layer caching flag for that entry.

                    // Step 7.4.2. Abort the image request for the current request and the pending
                    // request.
                    self.abort_request(State::CompletelyAvailable, ImageRequestPhase::Current, cx);
                    self.abort_request(State::Unavailable, ImageRequestPhase::Pending, cx);

                    // Step 7.4.3. Set the pending request to null.
                    self.image_request.set(ImageRequestPhase::Current);

                    // Step 7.4.4. Set the current request to a new image request whose image data
                    // is that of the entry and whose state is completely available.
                    let mut current_request = self.current_request.borrow_mut();
                    current_request.metadata = Some(image.metadata());
                    current_request.image = Some(image);
                    current_request.final_url = Some(image_url.clone());

                    // TODO Step 7.4.5. Prepare the current request for presentation given the img
                    // element.
                    self.upcast::<Node>().dirty(cx.no_gc(), NodeDamage::Other);

                    // Step 7.4.6. Set the current request's current pixel density to selected pixel
                    // density.
                    current_request.current_pixel_density = selected_pixel_density;

                    // Step 7.4.7. Queue an element task on the DOM manipulation task source given
                    // the img element and the following steps:
                    let this = Trusted::new(self);

                    self.owner_global()
                        .task_manager()
                        .dom_manipulation_task_source()
                        .queue(task!(image_load_event: move |cx| {
                            let this = this.root();

                            // TODO Step 7.4.7.1. If restart animation is set, then restart the
                            // animation.

                            // Step 7.4.7.2. Set the current request's current URL to urlString.
                            {
                                let mut current_request =
                                    this.current_request.borrow_mut();
                                current_request.source_url = Some(selected_source);
                                current_request.parsed_url = Some(image_url);
                            }

                            // Step 7.4.7.3. If maybe omit events is not set or previousURL is not
                            // equal to urlString, then fire an event named load at the img element.
                            // TODO: Add missing `maybe omit events` flag and previousURL.
                            this.upcast::<EventTarget>().fire_event(cx, atom!("load"));
                        }));

                    // Step 7.4.8. Abort the update the image data algorithm.
                    return;
                }
            }
        }

        // Step 8. Queue a microtask to perform the rest of this algorithm, allowing the task that
        // invoked this algorithm to continue.
        let task = ImageElementMicrotask::UpdateImageData {
            elem: Dom::from_ref(self),
            generation: self.generation.get(),
        };

        ScriptThread::await_stable_state(cx, Box::new(task));
    }

    /// <https://html.spec.whatwg.org/multipage/#img-environment-changes>
    pub(crate) fn react_to_environment_changes(&self, cx: &JSContext) {
        // Step 1. Await a stable state.
        let task = ImageElementMicrotask::EnvironmentChanges {
            elem: Dom::from_ref(self),
            generation: self.generation.get(),
        };

        ScriptThread::await_stable_state(cx, Box::new(task));
    }

    /// <https://html.spec.whatwg.org/multipage/#img-environment-changes>
    fn react_to_environment_changes_sync_steps(
        &self,
        generation: u32,
        cx: &mut js::context::JSContext,
    ) {
        let document = self.owner_document();
        let has_pending_request = matches!(self.image_request.get(), ImageRequestPhase::Pending);

        // Step 2. If the img element does not use srcset or picture, its node document is not fully
        // active, it has image data whose resource type is multipart/x-mixed-replace, or its
        // pending request is not null, then return.
        if !document.is_active() || !self.uses_srcset_or_picture() || has_pending_request {
            return;
        }

        // Step 3. Let selected source and selected pixel density be the URL and pixel density that
        // results from selecting an image source, respectively.
        let Some((selected_source, selected_pixel_density)) = self
            .source_set
            .borrow_mut()
            .select_image_source(self.upcast::<Element>())
        else {
            // Step 4. If selected source is null, then return.
            return;
        };

        // Step 5. If selected source and selected pixel density are the same as the element's last
        // selected source and current pixel density, then return.
        let mut same_selected_source = self
            .last_selected_source
            .borrow()
            .as_ref()
            .is_some_and(|source| *source == selected_source);

        // There are missing steps for the element's last selected source in specification so let's
        // check the current request's current URL as well.
        // <https://github.com/whatwg/html/issues/5060>
        same_selected_source = same_selected_source ||
            self.current_request
                .borrow()
                .source_url
                .as_ref()
                .is_some_and(|source| *source == selected_source);

        let same_selected_pixel_density = self
            .current_request
            .borrow()
            .current_pixel_density
            .is_some_and(|pixel_density| pixel_density == selected_pixel_density);

        if same_selected_source && same_selected_pixel_density {
            return;
        }

        // Step 6. Let urlString be the result of encoding-parsing-and-serializing a URL given
        // selected source, relative to the element's node document.
        // Step 7. If urlString is failure, then return.
        let Ok(image_url) = document.base_url().join(&selected_source) else {
            return;
        };

        // Step 13. Set the element's pending request to image request.
        self.image_request.set(ImageRequestPhase::Pending);
        self.init_image_request(&self.pending_request, &image_url, &selected_source, cx);

        // Step 15. If the list of available images contains an entry for key, then set image
        // request's image data to that of the entry. Continue to the next step.
        let window = self.owner_window();
        let cache_result = window.image_cache().get_cached_image_status(
            image_url.clone(),
            window.origin().immutable().clone(),
            cors_setting_for_element(self.upcast()),
        );

        let change_type = ChangeType::Environment {
            selected_source: selected_source.clone(),
            selected_pixel_density,
        };

        match cache_result {
            ImageCacheResult::Available(ImageOrMetadataAvailable::ImageAvailable { .. }) => {
                self.finish_reacting_to_environment_change(
                    selected_source,
                    generation,
                    selected_pixel_density,
                );
            },
            ImageCacheResult::Available(ImageOrMetadataAvailable::MetadataAvailable(m, id)) => {
                self.process_image_response_for_environment_change(
                    ImageResponse::MetadataLoaded(m),
                    selected_source,
                    generation,
                    selected_pixel_density,
                    cx,
                );
                self.register_image_cache_callback(id, change_type);
            },
            ImageCacheResult::FailedToLoadOrDecode => {
                self.process_image_response_for_environment_change(
                    ImageResponse::FailedToLoadOrDecode,
                    selected_source,
                    generation,
                    selected_pixel_density,
                    cx,
                );
            },
            ImageCacheResult::ReadyForRequest(id) => {
                self.fetch_request(&image_url, id);
                self.register_image_cache_callback(id, change_type);
            },
            ImageCacheResult::Pending(id) => {
                self.register_image_cache_callback(id, change_type);
            },
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-img-decode>
    fn react_to_decode_image_sync_steps(&self, cx: &mut JSContext, promise: Rc<Promise>) {
        // Step 2.2. If any of the following are true: this's node document is not fully active; or
        // this's current request's state is broken, then reject promise with an "EncodingError"
        // DOMException.
        if !self.owner_document().is_fully_active() ||
            matches!(self.current_request.borrow().state, State::Broken)
        {
            promise.reject_error(cx, Error::Encoding(Some("Image element's owner document is not fully active or image element's request is broken".into())));
        } else if matches!(
            self.current_request.borrow().state,
            State::CompletelyAvailable
        ) {
            // this doesn't follow the spec, but it's been discussed in <https://github.com/whatwg/html/issues/4217>
            promise.resolve_native(cx, &());
        } else if matches!(self.current_request.borrow().state, State::Unavailable) &&
            self.current_request.borrow().source_url.is_none()
        {
            // Note: Despite being not explicitly stated in the specification but if current
            // request's state is unavailable and current URL is empty string (<img> without "src"
            // and "srcset" attributes) then reject promise with an "EncodingError" DOMException.
            // <https://github.com/whatwg/html/issues/11769>
            promise.reject_error(
                cx,
                Error::Encoding(Some(
                    "Image element does not provide `src` or `srcset` attributes".into(),
                )),
            );
        } else {
            self.image_decode_promises.borrow_mut().push(promise);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-img-decode>
    fn resolve_image_decode_promises(&self) {
        if self.image_decode_promises.borrow().is_empty() {
            return;
        }

        // Step 2.3. If the decoding process completes successfully, then queue a global task on the
        // DOM manipulation task source with global to resolve promise with undefined.
        let trusted_image_decode_promises: Vec<TrustedPromise> = self
            .image_decode_promises
            .borrow()
            .iter()
            .map(|promise| TrustedPromise::new(promise.clone()))
            .collect();

        self.image_decode_promises.borrow_mut().clear();

        self.owner_global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(fulfill_image_decode_promises: move |cx| {
                for trusted_promise in trusted_image_decode_promises {
                    trusted_promise.root().resolve_native(cx, &());
                }
            }));
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-img-decode>
    fn reject_image_decode_promises(&self) {
        if self.image_decode_promises.borrow().is_empty() {
            return;
        }

        // Step 2.3. Queue a global task on the DOM manipulation task source with global to reject
        // promise with an "EncodingError" DOMException.
        let trusted_image_decode_promises: Vec<TrustedPromise> = self
            .image_decode_promises
            .borrow()
            .iter()
            .map(|promise| TrustedPromise::new(promise.clone()))
            .collect();

        self.image_decode_promises.borrow_mut().clear();

        self.owner_global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(reject_image_decode_promises: move |cx| {
                for trusted_promise in trusted_image_decode_promises {
                    trusted_promise.root().reject_error(cx, Error::Encoding(Some("Could not decode all image promises".into())));
                }
            }));
    }

    /// <https://html.spec.whatwg.org/multipage/#img-environment-changes>
    fn finish_reacting_to_environment_change(
        &self,
        selected_source: USVString,
        generation: u32,
        selected_pixel_density: f64,
    ) {
        // Step 16. Queue an element task on the DOM manipulation task source given the img element
        // and the following steps:
        let this = Trusted::new(self);

        self.owner_global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(image_load_event: move |cx| {
                let this = this.root();

                // Step 16.1. If the img element has experienced relevant mutations since this
                // algorithm started, then set the pending request to null and abort these steps.
                if this.generation.get() != generation {
                    this.abort_request(State::Unavailable, ImageRequestPhase::Pending, cx);
                    this.image_request.set(ImageRequestPhase::Current);
                    return;
                }

                // Step 16.2. Set the img element's last selected source to selected source and the
                // img element's current pixel density to selected pixel density.
                *this.last_selected_source.borrow_mut() = Some(selected_source);

                {
                    let mut pending_request = this.pending_request.borrow_mut();

                    // Step 16.3. Set the image request's state to completely available.
                    pending_request.state = State::CompletelyAvailable;

                    pending_request.current_pixel_density = Some(selected_pixel_density);

                    // Step 16.4. Add the image to the list of available images using the key key,
                    // with the ignore higher-layer caching flag set.
                    // Already a part of the list of available images due to Step 15.

                    // Step 16.5. Upgrade the pending request to the current request.
                    mem::swap(&mut *this.current_request.borrow_mut(), &mut *pending_request);
                }

                this.abort_request(State::Unavailable, ImageRequestPhase::Pending, cx);
                this.image_request.set(ImageRequestPhase::Current);

                // TODO Step 16.6. Prepare image request for presentation given the img element.
                this.upcast::<Node>().dirty(cx.no_gc(), NodeDamage::Other);

                // Step 16.7. Fire an event named load at the img element.
                this.upcast::<EventTarget>().fire_event(cx, atom!("load"));
            }));
    }

    /// <https://html.spec.whatwg.org/multipage/#use-srcset-or-picture>
    fn uses_srcset_or_picture(&self) -> bool {
        let element = self.upcast::<Element>();

        let has_srcset_attribute = element.has_attribute(&local_name!("srcset"));
        let has_parent_picture = element
            .upcast::<Node>()
            .GetParentElement()
            .is_some_and(|parent| parent.is::<HTMLPictureElement>());
        has_srcset_attribute || has_parent_picture
    }

    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        creator: ElementCreator,
    ) -> HTMLImageElement {
        HTMLImageElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            image_request: Cell::new(ImageRequestPhase::Current),
            current_request: DomRefCell::new(ImageRequest {
                state: State::Unavailable,
                parsed_url: None,
                source_url: None,
                image: None,
                metadata: None,
                blocker: DomRefCell::new(None),
                final_url: None,
                current_pixel_density: None,
            }),
            pending_request: DomRefCell::new(ImageRequest {
                state: State::Unavailable,
                parsed_url: None,
                source_url: None,
                image: None,
                metadata: None,
                blocker: DomRefCell::new(None),
                final_url: None,
                current_pixel_density: None,
            }),
            form_owner: Default::default(),
            generation: Default::default(),
            source_set: DomRefCell::new(SourceSet::new()),
            dimension_attribute_source: Default::default(),
            last_selected_source: DomRefCell::new(None),
            image_decode_promises: DomRefCell::new(vec![]),
            line_number: creator.return_line_number(),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        creator: ElementCreator,
    ) -> DomRoot<HTMLImageElement> {
        let image_element = Node::reflect_node_with_proto(
            cx,
            Box::new(HTMLImageElement::new_inherited(
                local_name, prefix, document, creator,
            )),
            document,
            proto,
        );
        image_element
            .dimension_attribute_source
            .set(Some(image_element.upcast()));
        image_element
    }

    pub(crate) fn areas(&self) -> Option<Vec<DomRoot<HTMLAreaElement>>> {
        let elem = self.upcast::<Element>();
        let value = elem.get_attribute_string_value(&local_name!("usemap"))?;

        if value.is_empty() || !value.is_char_boundary(1) {
            return None;
        }

        let (first, last) = value.split_at(1);

        if first != "#" || last.is_empty() {
            return None;
        }

        let useMapElements = self
            .owner_document()
            .upcast::<Node>()
            .traverse_preorder(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<HTMLMapElement>)
            .find(|n| {
                n.upcast::<Element>()
                    .get_name()
                    .is_some_and(|n| *n == *last)
            });

        useMapElements.map(|mapElem| mapElem.get_area_elements())
    }

    pub(crate) fn same_origin(&self, origin: &MutableOrigin) -> bool {
        if let Some(ref image) = self.current_request.borrow().image {
            return image.cors_status() == CorsStatus::Safe;
        }

        self.current_request
            .borrow()
            .final_url
            .as_ref()
            .is_some_and(|url| url.scheme() == "data" || url.origin().same_origin(origin))
    }

    fn generation_id(&self) -> u32 {
        self.generation.get()
    }

    fn load_broken_image_icon(&self, no_gc: &NoGC) {
        let window = self.owner_window();
        let Some(broken_image_icon) = window.image_cache().get_broken_image_icon() else {
            return;
        };

        self.current_request.borrow_mut().metadata = Some(broken_image_icon.metadata);
        self.current_request.borrow_mut().image = Some(Image::Raster(broken_image_icon));
        self.upcast::<Node>().dirty(no_gc, NodeDamage::Other);
    }

    /// Get the full URL of the current image of this `<img>` element, returning `None` if the URL
    /// could not be joined with the `Document` URL.
    pub(crate) fn full_image_url_for_user_interface(&self) -> Option<ServoUrl> {
        self.owner_document()
            .base_url()
            .join(&self.CurrentSrc())
            .ok()
    }

    pub(crate) fn set_dimension_attribute_source(&self, value: Option<&Element>) {
        self.dimension_attribute_source.set(value)
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) enum ImageElementMicrotask {
    UpdateImageData {
        elem: Dom<HTMLImageElement>,
        generation: u32,
    },
    EnvironmentChanges {
        elem: Dom<HTMLImageElement>,
        generation: u32,
    },
    Decode {
        elem: Dom<HTMLImageElement>,
        #[conditional_malloc_size_of]
        promise: Rc<Promise>,
    },
}

impl MicrotaskRunnable for ImageElementMicrotask {
    fn handler(&self, cx: &mut js::context::JSContext) {
        let mut realm = match self {
            &ImageElementMicrotask::UpdateImageData { ref elem, .. } |
            &ImageElementMicrotask::EnvironmentChanges { ref elem, .. } |
            &ImageElementMicrotask::Decode { ref elem, .. } => enter_auto_realm(cx, &**elem),
        };
        let cx = &mut realm;
        match *self {
            ImageElementMicrotask::UpdateImageData {
                ref elem,
                ref generation,
            } => {
                // <https://html.spec.whatwg.org/multipage/#update-the-image-data>
                // Step 9. If another instance of this algorithm for this img element was started
                // after this instance (even if it aborted and is no longer running), then return.
                if elem.generation.get() == *generation {
                    elem.update_the_image_data_sync_steps(cx);
                }
            },
            ImageElementMicrotask::EnvironmentChanges {
                ref elem,
                ref generation,
            } => {
                elem.react_to_environment_changes_sync_steps(*generation, cx);
            },
            ImageElementMicrotask::Decode {
                ref elem,
                ref promise,
            } => {
                elem.react_to_decode_image_sync_steps(cx, promise.clone());
            },
        }
    }
}

impl<'dom> LayoutDom<'dom, HTMLImageElement> {
    #[expect(unsafe_code)]
    fn current_request(self) -> &'dom ImageRequest {
        unsafe { self.unsafe_get().current_request.borrow_for_layout() }
    }

    #[expect(unsafe_code)]
    fn dimension_attribute_source(self) -> LayoutDom<'dom, Element> {
        unsafe {
            self.unsafe_get()
                .dimension_attribute_source
                .to_layout()
                .expect("dimension attribute source should be always non-null")
        }
    }

    pub(crate) fn image_url(self) -> Option<ServoUrl> {
        self.current_request().parsed_url.clone()
    }

    pub(crate) fn image_data(self) -> (Option<Image>, Option<ImageMetadata>) {
        let current_request = self.current_request();
        (current_request.image.clone(), current_request.metadata)
    }

    pub(crate) fn image_density(self) -> Option<f64> {
        self.current_request().current_pixel_density
    }

    pub(crate) fn showing_broken_image_icon(self) -> bool {
        matches!(self.current_request().state, State::Broken)
    }

    pub(crate) fn get_width(self) -> LengthOrPercentageOrAuto {
        self.dimension_attribute_source()
            .get_attr_for_layout(&ns!(), &local_name!("width"))
            .map(AttrValue::as_dimension)
            .cloned()
            .unwrap_or(LengthOrPercentageOrAuto::Auto)
    }

    pub(crate) fn get_height(self) -> LengthOrPercentageOrAuto {
        self.dimension_attribute_source()
            .get_attr_for_layout(&ns!(), &local_name!("height"))
            .map(AttrValue::as_dimension)
            .cloned()
            .unwrap_or(LengthOrPercentageOrAuto::Auto)
    }
}

impl HTMLImageElementMethods<crate::DomTypeHolder> for HTMLImageElement {
    /// <https://html.spec.whatwg.org/multipage/#dom-image>
    fn Image(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Fallible<DomRoot<HTMLImageElement>> {
        // Step 1. Let document be the current global object's associated Document.
        let document = window.Document();

        // Step 2. Let img be the result of creating an element given document, "img", and the HTML
        // namespace.
        let element = Element::create(
            cx,
            QualName::new(None, ns!(html), local_name!("img")),
            None,
            &document,
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Synchronous,
            proto,
        );

        let image = DomRoot::downcast::<HTMLImageElement>(element).unwrap();

        // Step 3. If width is given, then set an attribute value for img using "width" and width.
        if let Some(w) = width {
            image.SetWidth(cx, w);
        }

        // Step 4. If height is given, then set an attribute value for img using "height" and
        // height.
        if let Some(h) = height {
            image.SetHeight(cx, h);
        }

        // Step 5. Return img.
        Ok(image)
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-alt
    make_getter!(Alt, "alt");
    // https://html.spec.whatwg.org/multipage/#dom-img-alt
    make_setter!(SetAlt, "alt");

    // https://html.spec.whatwg.org/multipage/#dom-img-src
    make_url_getter!(Src, "src");

    // https://html.spec.whatwg.org/multipage/#dom-img-src
    make_url_setter!(SetSrc, "src");

    // https://html.spec.whatwg.org/multipage/#dom-img-srcset
    make_url_getter!(Srcset, "srcset");
    // https://html.spec.whatwg.org/multipage/#dom-img-src
    make_url_setter!(SetSrcset, "srcset");

    // <https://html.spec.whatwg.org/multipage/#dom-img-sizes>
    make_getter!(Sizes, "sizes");

    // <https://html.spec.whatwg.org/multipage/#dom-img-sizes>
    make_setter!(SetSizes, "sizes");

    /// <https://html.spec.whatwg.org/multipage/#dom-img-crossOrigin>
    fn GetCrossOrigin(&self) -> Option<DOMString> {
        reflect_cross_origin_attribute(self.upcast::<Element>())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-img-crossOrigin>
    fn SetCrossOrigin(&self, cx: &mut JSContext, value: Option<DOMString>) {
        set_cross_origin_attribute(cx, self.upcast::<Element>(), value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-usemap
    make_getter!(UseMap, "usemap");
    // https://html.spec.whatwg.org/multipage/#dom-img-usemap
    make_setter!(SetUseMap, "usemap");

    // https://html.spec.whatwg.org/multipage/#dom-img-ismap
    make_bool_getter!(IsMap, "ismap");
    // https://html.spec.whatwg.org/multipage/#dom-img-ismap
    make_bool_setter!(SetIsMap, "ismap");

    // <https://html.spec.whatwg.org/multipage/#dom-img-width>
    fn Width(&self) -> u32 {
        let node = self.upcast::<Node>();
        node.content_box()
            .map(|rect| rect.size.width.to_px() as u32)
            .unwrap_or_else(|| self.NaturalWidth())
    }

    // <https://html.spec.whatwg.org/multipage/#dom-img-width>
    make_dimension_uint_setter!(SetWidth, "width");

    // <https://html.spec.whatwg.org/multipage/#dom-img-height>
    fn Height(&self) -> u32 {
        let node = self.upcast::<Node>();
        node.content_box()
            .map(|rect| rect.size.height.to_px() as u32)
            .unwrap_or_else(|| self.NaturalHeight())
    }

    // <https://html.spec.whatwg.org/multipage/#dom-img-height>
    make_dimension_uint_setter!(SetHeight, "height");

    /// <https://html.spec.whatwg.org/multipage/#dom-img-naturalwidth>
    fn NaturalWidth(&self) -> u32 {
        let request = self.current_request.borrow();
        if matches!(request.state, State::Broken) {
            return 0;
        }

        let pixel_density = request.current_pixel_density.unwrap_or(1f64);
        match request.metadata {
            Some(ref metadata) => (metadata.width as f64 / pixel_density) as u32,
            None => 0,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-img-naturalheight>
    fn NaturalHeight(&self) -> u32 {
        let request = self.current_request.borrow();
        if matches!(request.state, State::Broken) {
            return 0;
        }

        let pixel_density = request.current_pixel_density.unwrap_or(1f64);
        match request.metadata {
            Some(ref metadata) => (metadata.height as f64 / pixel_density) as u32,
            None => 0,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-img-complete>
    fn Complete(&self) -> bool {
        let element = self.upcast::<Element>();

        // Step 1. If any of the following are true:
        // both the src attribute and the srcset attribute are omitted;
        let has_srcset_attribute = element.has_attribute(&local_name!("srcset"));
        if !element.has_attribute(&local_name!("src")) && !has_srcset_attribute {
            return true;
        }

        // the srcset attribute is omitted and the src attribute's value is the empty string;
        let src = element.get_string_attribute(&local_name!("src"));
        if !has_srcset_attribute && src.is_empty() {
            return true;
        }

        // the img element's current request's state is completely available and its pending request
        // is null; or the img element's current request's state is broken and its pending request
        // is null, then return true.
        if matches!(self.image_request.get(), ImageRequestPhase::Current) &&
            matches!(
                self.current_request.borrow().state,
                State::CompletelyAvailable | State::Broken
            )
        {
            return true;
        }

        // Step 2. Return false.
        false
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-img-currentsrc>
    fn CurrentSrc(&self) -> USVString {
        let current_request = self.current_request.borrow();
        let url = &current_request.parsed_url;
        match *url {
            Some(ref url) => USVString(url.clone().into_string()),
            None => {
                let unparsed_url = &current_request.source_url;
                match *unparsed_url {
                    Some(ref url) => url.clone(),
                    None => USVString("".to_owned()),
                }
            },
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-img-referrerpolicy>
    fn ReferrerPolicy(&self) -> DOMString {
        reflect_referrer_policy_attribute(self.upcast::<Element>())
    }

    // <https://html.spec.whatwg.org/multipage/#dom-img-referrerpolicy>
    make_setter!(SetReferrerPolicy, "referrerpolicy");

    /// <https://html.spec.whatwg.org/multipage/#dom-img-decode>
    fn Decode(&self, cx: &mut JSContext) -> Rc<Promise> {
        // Step 1. Let promise be a new promise.
        let promise = Promise::new(cx, &self.global());

        // Step 2. Queue a microtask to perform the following steps:
        let task = ImageElementMicrotask::Decode {
            elem: Dom::from_ref(self),
            promise: promise.clone(),
        };

        ScriptThread::await_stable_state(cx, Box::new(task));

        // Step 3. Return promise.
        promise
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
    make_url_getter!(LongDesc, "longdesc");

    // https://html.spec.whatwg.org/multipage/#dom-img-longdesc
    make_url_setter!(SetLongDesc, "longdesc");

    // https://html.spec.whatwg.org/multipage/#dom-img-border
    make_getter!(Border, "border");

    // https://html.spec.whatwg.org/multipage/#dom-img-border
    make_setter!(SetBorder, "border");
}

impl VirtualMethods for HTMLImageElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn adopting_steps(&self, cx: &mut JSContext, old_doc: &Document) {
        self.super_type().unwrap().adopting_steps(cx, old_doc);
        self.update_the_image_data(cx);
    }

    fn attribute_mutated(
        &self,
        cx: &mut js::context::JSContext,
        attr: AttrRef<'_>,
        mutation: AttributeMutation,
    ) {
        self.super_type()
            .unwrap()
            .attribute_mutated(cx, attr, mutation);
        match attr.local_name() {
            &local_name!("src") |
            &local_name!("srcset") |
            &local_name!("width") |
            &local_name!("sizes") => {
                // <https://html.spec.whatwg.org/multipage/#reacting-to-dom-mutations>
                // The element's src, srcset, width, or sizes attributes are set, changed, or
                // removed.
                self.update_the_image_data(cx);
            },
            &local_name!("crossorigin") => {
                // <https://html.spec.whatwg.org/multipage/#reacting-to-dom-mutations>
                // The element's crossorigin attribute's state is changed.
                let cross_origin_state_changed = match mutation {
                    AttributeMutation::Removed | AttributeMutation::Set(None, _) => true,
                    AttributeMutation::Set(Some(old_value), _) => {
                        let new_cors_setting =
                            CorsSettings::from_enumerated_attribute(&attr.value());
                        let old_cors_setting = CorsSettings::from_enumerated_attribute(old_value);

                        new_cors_setting != old_cors_setting
                    },
                };

                if cross_origin_state_changed {
                    self.update_the_image_data(cx);
                }
            },
            &local_name!("referrerpolicy") => {
                // <https://html.spec.whatwg.org/multipage/#reacting-to-dom-mutations>
                // The element's referrerpolicy attribute's state is changed.
                let referrer_policy_state_changed = match mutation {
                    AttributeMutation::Removed | AttributeMutation::Set(None, _) => {
                        ReferrerPolicy::from(&**attr.value()) != ReferrerPolicy::EmptyString
                    },
                    AttributeMutation::Set(Some(old_value), _) => {
                        ReferrerPolicy::from(&**attr.value()) != ReferrerPolicy::from(&**old_value)
                    },
                };

                if referrer_policy_state_changed {
                    self.update_the_image_data(cx);
                }
            },
            _ => {},
        }
    }

    fn attribute_affects_presentational_hints(&self, attr: AttrRef<'_>) -> bool {
        match attr.local_name() {
            &local_name!("width") | &local_name!("height") => true,
            _ => self
                .super_type()
                .unwrap()
                .attribute_affects_presentational_hints(attr),
        }
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("width") | &local_name!("height") => {
                AttrValue::from_dimension(value.into())
            },
            &local_name!("hspace") | &local_name!("vspace") => AttrValue::from_u32(value.into(), 0),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }

    fn handle_event(&self, cx: &mut js::context::JSContext, event: &Event) {
        if event.type_() != atom!("click") {
            return;
        }

        let Some(area_elements) = self.areas() else {
            return;
        };

        // Fetch click coordinates
        let Some(mouse_event) = event.downcast::<MouseEvent>() else {
            return;
        };

        let click_location = Point2D::new(
            mouse_event.ClientX().to_f32().unwrap(),
            mouse_event.ClientY().to_f32().unwrap(),
        );
        let bounding_rectangle = self.upcast::<Element>().GetBoundingClientRect(cx);
        let image_extents =
            Point2D::new(bounding_rectangle.X() as f32, bounding_rectangle.Y() as f32);

        // Walk HTMLAreaElements
        for area_element in area_elements {
            if !area_element.is_instance_activatable() {
                continue;
            }
            let activatable_area = match area_element.get_shape_from_coords() {
                Some(shape) => shape.absolute_coords(image_extents),
                None => return,
            };
            if activatable_area.hit_test(&click_location) {
                area_element.activation_behavior(cx, event, self.upcast());
                return;
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#the-img-element:html-element-insertion-steps>
    fn bind_to_tree(&self, cx: &mut JSContext, context: &BindContext) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(cx, context);
        }
        let document = self.owner_document();
        if context.tree_connected {
            document.register_responsive_image(self);
        }

        let parent = self.upcast::<Node>().GetParentNode().unwrap();

        // Step 1. If insertedNode's parent is a picture element, then, count this as a relevant
        // mutation for insertedNode.
        if parent.is::<HTMLPictureElement>() && *parent == *context.parent {
            self.update_the_image_data(cx);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#the-img-element:html-element-removing-steps>
    fn unbind_from_tree(&self, cx: &mut js::context::JSContext, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(cx, context);
        let document = self.owner_document();
        document.unregister_responsive_image(self);

        // Step 1. If oldParent is a picture element, then, count this as a relevant mutation for
        // removedNode.
        if context.parent.is::<HTMLPictureElement>() && !self.upcast::<Node>().has_parent() {
            self.update_the_image_data(cx);
        }
    }

    /// <https://html.spec.whatwg.org/multipage#the-img-element:html-element-moving-steps>
    fn moving_steps(&self, cx: &mut JSContext, context: &MoveContext) {
        if let Some(super_type) = self.super_type() {
            super_type.moving_steps(cx, context);
        }

        // Step 1. If oldParent is a picture element, then, count this as a relevant mutation for movedNode.
        if let Some(old_parent) = context.old_parent &&
            old_parent.is::<HTMLPictureElement>()
        {
            self.update_the_image_data(cx);
        }
    }
}

impl FormControl for HTMLImageElement {
    fn form_owner(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner.get()
    }

    fn set_form_owner(&self, _cx: &mut JSContext, form: Option<&HTMLFormElement>) {
        self.form_owner.set(form);
    }

    fn to_html_element(&self) -> &HTMLElement {
        self.upcast::<HTMLElement>()
    }
}

#[derive(Clone)]
enum ChangeType {
    Environment {
        selected_source: USVString,
        selected_pixel_density: f64,
    },
    Element,
}
