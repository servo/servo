/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::{Au, AU_PER_PX};
use crate::document_loader::{LoadBlocker, LoadType};
use crate::dom::activation::Activatable;
use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DOMRectBinding::DOMRectBinding::DOMRectMethods;
use crate::dom::bindings::codegen::Bindings::ElementBinding::ElementBinding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLImageElementBinding;
use crate::dom::bindings::codegen::Bindings::HTMLImageElementBinding::HTMLImageElementMethods;
use crate::dom::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::{DomRoot, LayoutDom, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{reflect_cross_origin_attribute, set_cross_origin_attribute};
use crate::dom::element::{AttributeMutation, Element, RawLayoutElementHelpers};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlareaelement::HTMLAreaElement;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlformelement::{FormControl, HTMLFormElement};
use crate::dom::htmlmapelement::HTMLMapElement;
use crate::dom::htmlpictureelement::HTMLPictureElement;
use crate::dom::htmlsourceelement::HTMLSourceElement;
use crate::dom::mouseevent::MouseEvent;
use crate::dom::node::{document_from_node, window_from_node, Node, NodeDamage, UnbindContext};
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::progressevent::ProgressEvent;
use crate::dom::values::UNSIGNED_LONG_MAX;
use crate::dom::virtualmethods::VirtualMethods;
use crate::dom::window::Window;
use crate::microtask::{Microtask, MicrotaskRunnable};
use crate::network_listener::{self, NetworkListener, PreInvoke, ResourceTimingListener};
use crate::script_thread::ScriptThread;
use crate::task_source::TaskSource;
use cssparser::{Parser, ParserInput};

use dom_struct::dom_struct;
use euclid::Point2D;
use html5ever::{LocalName, Prefix};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use mime::{self, Mime};
use net_traits::image::base::{Image, ImageMetadata};
use net_traits::image_cache::UsePlaceholder;
use net_traits::image_cache::{CanRequestImages, ImageCache, ImageOrMetadataAvailable};
use net_traits::image_cache::{ImageResponder, ImageResponse, ImageState, PendingImageId};
use net_traits::request::RequestInit;
use net_traits::{FetchMetadata, FetchResponseListener, FetchResponseMsg, NetworkError};
use net_traits::{ResourceFetchTiming, ResourceTimingType};
use num_traits::ToPrimitive;
use servo_url::origin::MutableOrigin;
use servo_url::ServoUrl;
use std::cell::{Cell, RefMut};
use std::char;
use std::collections::HashSet;
use std::default::Default;
use std::i32;
use std::mem;
use std::sync::{Arc, Mutex};
use style::attr::{
    parse_double, parse_length, parse_unsigned_integer, AttrValue, LengthOrPercentageOrAuto,
};
use style::context::QuirksMode;
use style::media_queries::MediaList;
use style::parser::ParserContext;
use style::str::is_ascii_digit;
use style::stylesheets::{CssRuleType, Origin};
use style::values::specified::length::{Length, NoCalcLength};
use style::values::specified::{source_size_list::SourceSizeList, AbsoluteLength};
use style_traits::ParsingMode;

enum ParseState {
    InDescriptor,
    InParens,
    AfterDescriptor,
}

pub struct SourceSet {
    image_sources: Vec<ImageSource>,
    source_size: SourceSizeList,
}

impl SourceSet {
    fn new() -> SourceSet {
        SourceSet {
            image_sources: Vec::new(),
            source_size: SourceSizeList::empty(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImageSource {
    pub url: String,
    pub descriptor: Descriptor,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Descriptor {
    pub wid: Option<u32>,
    pub den: Option<f64>,
}

#[derive(Clone, Copy, JSTraceable, MallocSizeOf)]
#[allow(dead_code)]
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
#[derive(JSTraceable, MallocSizeOf)]
#[must_root]
struct ImageRequest {
    state: State,
    parsed_url: Option<ServoUrl>,
    source_url: Option<DOMString>,
    blocker: Option<LoadBlocker>,
    #[ignore_malloc_size_of = "Arc"]
    image: Option<Arc<Image>>,
    metadata: Option<ImageMetadata>,
    final_url: Option<ServoUrl>,
    current_pixel_density: Option<f64>,
}
#[dom_struct]
pub struct HTMLImageElement {
    htmlelement: HTMLElement,
    image_request: Cell<ImageRequestPhase>,
    current_request: DomRefCell<ImageRequest>,
    pending_request: DomRefCell<ImageRequest>,
    form_owner: MutNullableDom<HTMLFormElement>,
    generation: Cell<u32>,
    #[ignore_malloc_size_of = "SourceSet"]
    source_set: DomRefCell<SourceSet>,
    last_selected_source: DomRefCell<Option<DOMString>>,
}

impl HTMLImageElement {
    pub fn get_url(&self) -> Option<ServoUrl> {
        self.current_request.borrow().parsed_url.clone()
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
    aborted: Cell<bool>,
    /// The document associated with this request
    doc: Trusted<Document>,
    /// timing data for this resource
    resource_timing: ResourceFetchTiming,
    url: ServoUrl,
}

impl FetchResponseListener for ImageContext {
    fn process_request_body(&mut self) {}
    fn process_request_eof(&mut self) {}

    fn process_response(&mut self, metadata: Result<FetchMetadata, NetworkError>) {
        self.image_cache
            .notify_pending_response(self.id, FetchResponseMsg::ProcessResponse(metadata.clone()));

        let metadata = metadata.ok().map(|meta| match meta {
            FetchMetadata::Unfiltered(m) => m,
            FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
        });

        // Step 14.5 of https://html.spec.whatwg.org/multipage/#img-environment-changes
        if let Some(metadata) = metadata.as_ref() {
            if let Some(ref content_type) = metadata.content_type {
                let mime: Mime = content_type.clone().into_inner().into();
                if mime.type_() == mime::MULTIPART && mime.subtype().as_str() == "x-mixed-replace" {
                    self.aborted.set(true);
                }
            }
        }

        let status_code = metadata
            .as_ref()
            .and_then(|m| m.status.as_ref().map(|&(code, _)| code))
            .unwrap_or(0);

        self.status = match status_code {
            0 => Err(NetworkError::Internal(
                "No http status code received".to_owned(),
            )),
            200...299 => Ok(()), // HTTP ok status codes
            _ => Err(NetworkError::Internal(format!(
                "HTTP error code {}",
                status_code
            ))),
        };
    }

    fn process_response_chunk(&mut self, payload: Vec<u8>) {
        if self.status.is_ok() {
            self.image_cache
                .notify_pending_response(self.id, FetchResponseMsg::ProcessResponseChunk(payload));
        }
    }

    fn process_response_eof(&mut self, response: Result<ResourceFetchTiming, NetworkError>) {
        self.image_cache
            .notify_pending_response(self.id, FetchResponseMsg::ProcessResponseEOF(response));
    }

    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming {
        &mut self.resource_timing
    }

    fn resource_timing(&self) -> &ResourceFetchTiming {
        &self.resource_timing
    }

    fn submit_resource_timing(&mut self) {
        network_listener::submit_timing(self)
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

impl PreInvoke for ImageContext {
    fn should_invoke(&self) -> bool {
        !self.aborted.get()
    }
}

impl HTMLImageElement {
    /// Update the current image with a valid URL.
    fn fetch_image(&self, img_url: &ServoUrl) {
        fn add_cache_listener_for_element(
            image_cache: Arc<dyn ImageCache>,
            id: PendingImageId,
            elem: &HTMLImageElement,
        ) {
            let trusted_node = Trusted::new(elem);
            let (responder_sender, responder_receiver) = ipc::channel().unwrap();

            let window = window_from_node(elem);
            let (task_source, canceller) = window
                .task_manager()
                .networking_task_source_with_canceller();
            let generation = elem.generation.get();
            ROUTER.add_route(
                responder_receiver.to_opaque(),
                Box::new(move |message| {
                    debug!("Got image {:?}", message);
                    // Return the image via a message to the script thread, which marks
                    // the element as dirty and triggers a reflow.
                    let element = trusted_node.clone();
                    let image = message.to().unwrap();
                    // FIXME(nox): Why are errors silenced here?
                    let _ = task_source.queue_with_canceller(
                        task!(process_image_response: move || {
                        let element = element.root();
                        // Ignore any image response for a previous request that has been discarded.
                        if generation == element.generation.get() {
                            element.process_image_response(image);
                        }
                    }),
                        &canceller,
                    );
                }),
            );

            image_cache.add_listener(id, ImageResponder::new(responder_sender, id));
        }

        let window = window_from_node(self);
        let image_cache = window.image_cache();
        let response = image_cache.find_image_or_metadata(
            img_url.clone().into(),
            UsePlaceholder::Yes,
            CanRequestImages::Yes,
        );
        match response {
            Ok(ImageOrMetadataAvailable::ImageAvailable(image, url)) => {
                self.process_image_response(ImageResponse::Loaded(image, url));
            },

            Ok(ImageOrMetadataAvailable::MetadataAvailable(m)) => {
                self.process_image_response(ImageResponse::MetadataLoaded(m));
            },

            Err(ImageState::Pending(id)) => {
                add_cache_listener_for_element(image_cache.clone(), id, self);
            },

            Err(ImageState::LoadError) => {
                self.process_image_response(ImageResponse::None);
            },

            Err(ImageState::NotRequested(id)) => {
                add_cache_listener_for_element(image_cache, id, self);
                self.fetch_request(img_url, id);
            },
        }
    }

    fn fetch_request(&self, img_url: &ServoUrl, id: PendingImageId) {
        let document = document_from_node(self);
        let window = window_from_node(self);

        let context = Arc::new(Mutex::new(ImageContext {
            image_cache: window.image_cache(),
            status: Ok(()),
            id: id,
            aborted: Cell::new(false),
            doc: Trusted::new(&document),
            resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
            url: img_url.clone(),
        }));

        let (action_sender, action_receiver) = ipc::channel().unwrap();
        let (task_source, canceller) = document
            .window()
            .task_manager()
            .networking_task_source_with_canceller();
        let listener = NetworkListener {
            context,
            task_source,
            canceller: Some(canceller),
        };
        ROUTER.add_route(
            action_receiver.to_opaque(),
            Box::new(move |message| {
                listener.notify_fetch(message.to().unwrap());
            }),
        );

        let request = RequestInit {
            url: img_url.clone(),
            origin: document.origin().immutable().clone(),
            pipeline_id: Some(document.global().pipeline_id()),
            ..RequestInit::default()
        };

        // This is a background load because the load blocker already fulfills the
        // purpose of delaying the document's load event.
        document
            .loader_mut()
            .fetch_async_background(request, action_sender);
    }

    // Steps common to when an image has been loaded.
    fn handle_loaded_image(&self, image: Arc<Image>, url: ServoUrl) {
        self.current_request.borrow_mut().metadata = Some(ImageMetadata {
            height: image.height,
            width: image.width,
        });
        self.current_request.borrow_mut().final_url = Some(url);
        self.current_request.borrow_mut().image = Some(image);
        self.current_request.borrow_mut().state = State::CompletelyAvailable;
        LoadBlocker::terminate(&mut self.current_request.borrow_mut().blocker);
        // Mark the node dirty
        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }

    /// Step 24 of https://html.spec.whatwg.org/multipage/#update-the-image-data
    fn process_image_response(&self, image: ImageResponse) {
        // TODO: Handle multipart/x-mixed-replace
        let (trigger_image_load, trigger_image_error) = match (image, self.image_request.get()) {
            (ImageResponse::Loaded(image, url), ImageRequestPhase::Current) => {
                self.handle_loaded_image(image, url);
                (true, false)
            },
            (ImageResponse::PlaceholderLoaded(image, url), ImageRequestPhase::Current) => {
                self.handle_loaded_image(image, url);
                (false, true)
            },
            (ImageResponse::Loaded(image, url), ImageRequestPhase::Pending) => {
                self.abort_request(State::Unavailable, ImageRequestPhase::Pending);
                self.image_request.set(ImageRequestPhase::Current);
                self.handle_loaded_image(image, url);
                (true, false)
            },
            (ImageResponse::PlaceholderLoaded(image, url), ImageRequestPhase::Pending) => {
                self.abort_request(State::Unavailable, ImageRequestPhase::Pending);
                self.image_request.set(ImageRequestPhase::Current);
                self.handle_loaded_image(image, url);
                (false, true)
            },
            (ImageResponse::MetadataLoaded(meta), ImageRequestPhase::Current) => {
                self.current_request.borrow_mut().state = State::PartiallyAvailable;
                self.current_request.borrow_mut().metadata = Some(meta);
                (false, false)
            },
            (ImageResponse::MetadataLoaded(_), ImageRequestPhase::Pending) => {
                self.pending_request.borrow_mut().state = State::PartiallyAvailable;
                (false, false)
            },
            (ImageResponse::None, ImageRequestPhase::Current) => {
                self.abort_request(State::Broken, ImageRequestPhase::Current);
                (false, true)
            },
            (ImageResponse::None, ImageRequestPhase::Pending) => {
                self.abort_request(State::Broken, ImageRequestPhase::Current);
                self.abort_request(State::Broken, ImageRequestPhase::Pending);
                self.image_request.set(ImageRequestPhase::Current);
                (false, true)
            },
        };

        // Fire image.onload and loadend
        if trigger_image_load {
            // TODO: https://html.spec.whatwg.org/multipage/#fire-a-progress-event-or-event
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

    fn process_image_response_for_environment_change(
        &self,
        image: ImageResponse,
        src: DOMString,
        generation: u32,
        selected_pixel_density: f64,
    ) {
        match image {
            ImageResponse::Loaded(image, url) | ImageResponse::PlaceholderLoaded(image, url) => {
                self.pending_request.borrow_mut().metadata = Some(ImageMetadata {
                    height: image.height,
                    width: image.width,
                });
                self.pending_request.borrow_mut().final_url = Some(url);
                self.pending_request.borrow_mut().image = Some(image);
                self.finish_reacting_to_environment_change(src, generation, selected_pixel_density);
            },
            ImageResponse::MetadataLoaded(meta) => {
                self.pending_request.borrow_mut().metadata = Some(meta);
            },
            ImageResponse::None => {
                self.abort_request(State::Unavailable, ImageRequestPhase::Pending);
            },
        };
    }

    /// <https://html.spec.whatwg.org/multipage/#abort-the-image-request>
    fn abort_request(&self, state: State, request: ImageRequestPhase) {
        let mut request = match request {
            ImageRequestPhase::Current => self.current_request.borrow_mut(),
            ImageRequestPhase::Pending => self.pending_request.borrow_mut(),
        };
        LoadBlocker::terminate(&mut request.blocker);
        request.state = state;
        request.image = None;
        request.metadata = None;
    }

    /// <https://html.spec.whatwg.org/multipage/#update-the-source-set>
    fn update_source_set(&self) {
        // Step 1
        *self.source_set.borrow_mut() = SourceSet::new();

        // Step 2
        let elem = self.upcast::<Element>();
        let parent = elem.upcast::<Node>().GetParentElement();
        let nodes;
        let elements = match parent.as_ref() {
            Some(p) => {
                if p.is::<HTMLPictureElement>() {
                    nodes = p.upcast::<Node>().children();
                    nodes
                        .filter_map(DomRoot::downcast::<Element>)
                        .map(|n| DomRoot::from_ref(&*n))
                        .collect()
                } else {
                    vec![DomRoot::from_ref(&*elem)]
                }
            },
            None => vec![DomRoot::from_ref(&*elem)],
        };

        // Step 3
        let width = match elem.get_attribute(&ns!(), &local_name!("width")) {
            Some(x) => match parse_length(&x.value()) {
                LengthOrPercentageOrAuto::Length(x) => {
                    let abs_length = AbsoluteLength::Px(x.to_f32_px());
                    Some(Length::NoCalc(NoCalcLength::Absolute(abs_length)))
                },
                _ => None,
            },
            None => None,
        };

        // Step 4
        for element in &elements {
            // Step 4.1
            if *element == DomRoot::from_ref(&*elem) {
                let mut source_set = SourceSet::new();
                // Step 4.1.1
                if let Some(x) = element.get_attribute(&ns!(), &local_name!("srcset")) {
                    source_set.image_sources = parse_a_srcset_attribute(&x.value());
                }

                // Step 4.1.2
                if let Some(x) = element.get_attribute(&ns!(), &local_name!("sizes")) {
                    source_set.source_size =
                        parse_a_sizes_attribute(DOMString::from_string(x.value().to_string()));
                }

                // Step 4.1.3
                let src_attribute = element.get_string_attribute(&local_name!("src"));
                let is_src_empty = src_attribute.is_empty();
                let no_density_source_of_1 = source_set
                    .image_sources
                    .iter()
                    .all(|source| source.descriptor.den != Some(1.));
                let no_width_descriptor = source_set
                    .image_sources
                    .iter()
                    .all(|source| source.descriptor.wid.is_none());
                if !is_src_empty && no_density_source_of_1 && no_width_descriptor {
                    source_set.image_sources.push(ImageSource {
                        url: src_attribute.to_string(),
                        descriptor: Descriptor {
                            wid: None,
                            den: None,
                        },
                    })
                }

                // Step 4.1.4
                self.normalise_source_densities(&mut source_set, width);

                // Step 4.1.5
                *self.source_set.borrow_mut() = source_set;

                // Step 4.1.6
                return;
            }
            // Step 4.2
            if !element.is::<HTMLSourceElement>() {
                continue;
            }

            // Step 4.3 - 4.4
            let mut source_set = SourceSet::new();
            match element.get_attribute(&ns!(), &local_name!("srcset")) {
                Some(x) => {
                    source_set.image_sources = parse_a_srcset_attribute(&x.value());
                },
                _ => continue,
            }

            // Step 4.5
            if source_set.image_sources.is_empty() {
                continue;
            }

            // Step 4.6
            if let Some(x) = element.get_attribute(&ns!(), &local_name!("media")) {
                if !self.matches_environment(x.value().to_string()) {
                    continue;
                }
            }

            // Step 4.7
            if let Some(x) = element.get_attribute(&ns!(), &local_name!("sizes")) {
                source_set.source_size =
                    parse_a_sizes_attribute(DOMString::from_string(x.value().to_string()));
            }

            // Step 4.8
            if let Some(x) = element.get_attribute(&ns!(), &local_name!("type")) {
                // TODO Handle unsupported mime type
                let mime = x.value().parse::<Mime>();
                match mime {
                    Ok(m) => match m.type_() {
                        mime::IMAGE => (),
                        _ => continue,
                    },
                    _ => continue,
                }
            }

            // Step 4.9
            self.normalise_source_densities(&mut source_set, width);

            // Step 4.10
            *self.source_set.borrow_mut() = source_set;
            return;
        }
    }

    fn evaluate_source_size_list(
        &self,
        source_size_list: &mut SourceSizeList,
        _width: Option<Length>,
    ) -> Au {
        let document = document_from_node(self);
        let device = document.device();
        if !device.is_some() {
            return Au(1);
        }
        let quirks_mode = document.quirks_mode();
        //FIXME https://github.com/whatwg/html/issues/3832
        source_size_list.evaluate(&device.unwrap(), quirks_mode)
    }

    /// https://html.spec.whatwg.org/multipage/#matches-the-environment
    fn matches_environment(&self, media_query: String) -> bool {
        let document = document_from_node(self);
        let device = match document.device() {
            Some(device) => device,
            None => return false,
        };
        let quirks_mode = document.quirks_mode();
        let document_url = &document.url();
        // FIXME(emilio): This should do the same that we do for other media
        // lists regarding the rule type and such, though it doesn't really
        // matter right now...
        //
        // Also, ParsingMode::all() is wrong, and should be DEFAULT.
        let context = ParserContext::new(
            Origin::Author,
            document_url,
            Some(CssRuleType::Style),
            ParsingMode::all(),
            quirks_mode,
            None,
            None,
        );
        let mut parserInput = ParserInput::new(&media_query);
        let mut parser = Parser::new(&mut parserInput);
        let media_list = MediaList::parse(&context, &mut parser);
        media_list.evaluate(&device, quirks_mode)
    }

    /// <https://html.spec.whatwg.org/multipage/#normalise-the-source-densities>
    fn normalise_source_densities(&self, source_set: &mut SourceSet, width: Option<Length>) {
        // Step 1
        let mut source_size = &mut source_set.source_size;

        // Find source_size_length for Step 2.2
        let source_size_length = self.evaluate_source_size_list(&mut source_size, width);

        // Step 2
        for imgsource in &mut source_set.image_sources {
            // Step 2.1
            if imgsource.descriptor.den.is_some() {
                continue;
            }
            // Step 2.2
            if imgsource.descriptor.wid.is_some() {
                let wid = imgsource.descriptor.wid.unwrap();
                imgsource.descriptor.den = Some(wid as f64 / source_size_length.to_f64_px());
            } else {
                //Step 2.3
                imgsource.descriptor.den = Some(1 as f64);
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#select-an-image-source>
    fn select_image_source(&self) -> Option<(DOMString, f64)> {
        // Step 1, 3
        self.update_source_set();
        let source_set = &*self.source_set.borrow_mut();
        let len = source_set.image_sources.len();

        // Step 2
        if len == 0 {
            return None;
        }

        // Step 4
        let mut repeat_indices = HashSet::new();
        for outer_index in 0..len {
            if repeat_indices.contains(&outer_index) {
                continue;
            }
            let imgsource = &source_set.image_sources[outer_index];
            let pixel_density = imgsource.descriptor.den.unwrap();
            for inner_index in (outer_index + 1)..len {
                let imgsource2 = &source_set.image_sources[inner_index];
                if pixel_density == imgsource2.descriptor.den.unwrap() {
                    repeat_indices.insert(inner_index);
                }
            }
        }

        let mut max = (0f64, 0);
        let img_sources = &mut vec![];
        for (index, image_source) in source_set.image_sources.iter().enumerate() {
            if repeat_indices.contains(&index) {
                continue;
            }
            let den = image_source.descriptor.den.unwrap();
            if max.0 < den {
                max = (den, img_sources.len());
            }
            img_sources.push(image_source);
        }

        // Step 5
        let mut best_candidate = max;
        let device = document_from_node(self).device();
        if let Some(device) = device {
            let device_den = device.device_pixel_ratio().get() as f64;
            for (index, image_source) in img_sources.iter().enumerate() {
                let current_den = image_source.descriptor.den.unwrap();
                if current_den < best_candidate.0 && current_den >= device_den {
                    best_candidate = (current_den, index);
                }
            }
        }
        let selected_source = img_sources.remove(best_candidate.1).clone();
        Some((
            DOMString::from_string(selected_source.url),
            selected_source.descriptor.den.unwrap() as f64,
        ))
    }

    fn init_image_request(
        &self,
        request: &mut RefMut<ImageRequest>,
        url: &ServoUrl,
        src: &DOMString,
    ) {
        request.parsed_url = Some(url.clone());
        request.source_url = Some(src.clone());
        request.image = None;
        request.metadata = None;
        let document = document_from_node(self);
        LoadBlocker::terminate(&mut request.blocker);
        request.blocker = Some(LoadBlocker::new(&*document, LoadType::Image(url.clone())));
    }

    /// Step 13-17 of html.spec.whatwg.org/multipage/#update-the-image-data
    fn prepare_image_request(&self, url: &ServoUrl, src: &DOMString, selected_pixel_density: f64) {
        match self.image_request.get() {
            ImageRequestPhase::Pending => {
                if let Some(pending_url) = self.pending_request.borrow().parsed_url.clone() {
                    // Step 13
                    if pending_url == *url {
                        return;
                    }
                }
            },
            ImageRequestPhase::Current => {
                let mut current_request = self.current_request.borrow_mut();
                let mut pending_request = self.pending_request.borrow_mut();
                // step 16, create a new "image_request"
                match (current_request.parsed_url.clone(), current_request.state) {
                    (Some(parsed_url), State::PartiallyAvailable) => {
                        // Step 14
                        if parsed_url == *url {
                            // Step 15 abort pending request
                            pending_request.image = None;
                            pending_request.parsed_url = None;
                            LoadBlocker::terminate(&mut pending_request.blocker);
                            // TODO: queue a task to restart animation, if restart-animation is set
                            return;
                        }
                        pending_request.current_pixel_density = Some(selected_pixel_density);
                        self.image_request.set(ImageRequestPhase::Pending);
                        self.init_image_request(&mut pending_request, &url, &src);
                    },
                    (_, State::Broken) | (_, State::Unavailable) => {
                        // Step 17
                        current_request.current_pixel_density = Some(selected_pixel_density);
                        self.init_image_request(&mut current_request, &url, &src);
                    },
                    (_, _) => {
                        // step 17
                        pending_request.current_pixel_density = Some(selected_pixel_density);
                        self.image_request.set(ImageRequestPhase::Pending);
                        self.init_image_request(&mut pending_request, &url, &src);
                    },
                }
            },
        }
        self.fetch_image(&url);
    }

    /// Step 8-12 of html.spec.whatwg.org/multipage/#update-the-image-data
    fn update_the_image_data_sync_steps(&self) {
        let document = document_from_node(self);
        let window = document.window();
        let task_source = window.task_manager().dom_manipulation_task_source();
        let this = Trusted::new(self);
        let (src, pixel_density) = match self.select_image_source() {
            // Step 8
            Some(data) => data,
            None => {
                // Step 9.
                // FIXME(nox): Why are errors silenced here?
                let _ = task_source.queue(
                    task!(image_null_source_error: move || {
                        let this = this.root();
                        {
                            let mut current_request =
                                this.current_request.borrow_mut();
                            current_request.source_url = None;
                            current_request.parsed_url = None;
                        }
                        let elem = this.upcast::<Element>();
                        let src_present = elem.has_attribute(&local_name!("src"));

                        if src_present || Self::uses_srcset_or_picture(elem) {
                            this.upcast::<EventTarget>().fire_event(atom!("error"));
                        }
                        // FIXME(nox): According to the spec, setting the current
                        // request to the broken state is done prior to queuing a
                        // task, why is this here?
                        this.abort_request(State::Broken, ImageRequestPhase::Current);
                        this.abort_request(State::Broken, ImageRequestPhase::Pending);
                    }),
                    window.upcast(),
                );
                return;
            },
        };
        // Step 10.
        let target = Trusted::new(self.upcast::<EventTarget>());
        // FIXME(nox): Why are errors silenced here?
        let _ = task_source.queue(
            task!(fire_progress_event: move || {
                let target = target.root();

                let event = ProgressEvent::new(
                    &target.global(),
                    atom!("loadstart"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::NotCancelable,
                    false,
                    0,
                    0,
                );
                event.upcast::<Event>().fire(&target);
            }),
            window.upcast(),
        );
        // Step 11
        let base_url = document.base_url();
        let parsed_url = base_url.join(&src);
        match parsed_url {
            Ok(url) => {
                // Step 13-17
                self.prepare_image_request(&url, &src, pixel_density);
            },
            Err(_) => {
                // Step 12.1-12.5.
                let src = String::from(src);
                // FIXME(nox): Why are errors silenced here?
                let _ = task_source.queue(
                    task!(image_selected_source_error: move || {
                        let this = this.root();
                        {
                            let mut current_request =
                                this.current_request.borrow_mut();
                            current_request.source_url = Some(src.into());
                        }
                        this.upcast::<EventTarget>().fire_event(atom!("error"));
                        this.upcast::<EventTarget>().fire_event(atom!("loadend"));

                        // FIXME(nox): According to the spec, setting the current
                        // request to the broken state is done prior to queuing a
                        // task, why is this here?
                        this.abort_request(State::Broken, ImageRequestPhase::Current);
                        this.abort_request(State::Broken, ImageRequestPhase::Pending);
                    }),
                    window.upcast(),
                );
            },
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#update-the-image-data>
    pub fn update_the_image_data(&self) {
        let document = document_from_node(self);
        let window = document.window();
        let elem = self.upcast::<Element>();
        let src = elem.get_string_attribute(&local_name!("src"));
        let base_url = document.base_url();

        // https://html.spec.whatwg.org/multipage/#reacting-to-dom-mutations
        // Always first set the current request to unavailable,
        // ensuring img.complete is false.
        {
            let mut current_request = self.current_request.borrow_mut();
            current_request.state = State::Unavailable;
        }

        if !document.is_active() {
            // Step 1 (if the document is inactive)
            // TODO: use GlobalScope::enqueue_microtask,
            // to queue micro task to come back to this algorithm
        }
        // Step 2 abort if user-agent does not supports images
        // NOTE: Servo only supports images, skipping this step

        // Step 3, 4
        let mut selected_source = None;
        let mut pixel_density = None;
        let src_set = elem.get_string_attribute(&local_name!("srcset"));
        let is_parent_picture = elem
            .upcast::<Node>()
            .GetParentElement()
            .map_or(false, |p| p.is::<HTMLPictureElement>());
        if src_set.is_empty() && !is_parent_picture && !src.is_empty() {
            selected_source = Some(src.clone());
            pixel_density = Some(1 as f64);
        };

        // Step 5
        *self.last_selected_source.borrow_mut() = selected_source.clone();

        // Step 6, check the list of available images
        if !selected_source
            .as_ref()
            .map_or(false, |source| source.is_empty())
        {
            if let Ok(img_url) = base_url.join(&src) {
                let image_cache = window.image_cache();
                let response = image_cache.find_image_or_metadata(
                    img_url.clone().into(),
                    UsePlaceholder::No,
                    CanRequestImages::No,
                );
                if let Ok(ImageOrMetadataAvailable::ImageAvailable(image, url)) = response {
                    // Cancel any outstanding tasks that were queued before the src was
                    // set on this element.
                    self.generation.set(self.generation.get() + 1);
                    // Step 6.3
                    let metadata = ImageMetadata {
                        height: image.height,
                        width: image.width,
                    };
                    // Step 6.3.2 abort requests
                    self.abort_request(State::CompletelyAvailable, ImageRequestPhase::Current);
                    self.abort_request(State::Unavailable, ImageRequestPhase::Pending);
                    let mut current_request = self.current_request.borrow_mut();
                    current_request.final_url = Some(url);
                    current_request.image = Some(image.clone());
                    current_request.metadata = Some(metadata);
                    // Step 6.3.6
                    current_request.current_pixel_density = pixel_density;
                    let this = Trusted::new(self);
                    let src = String::from(src);
                    let _ = window.task_manager().dom_manipulation_task_source().queue(
                        task!(image_load_event: move || {
                            let this = this.root();
                            {
                                let mut current_request =
                                    this.current_request.borrow_mut();
                                current_request.parsed_url = Some(img_url);
                                current_request.source_url = Some(src.into());
                            }
                            // TODO: restart animation, if set.
                            this.upcast::<EventTarget>().fire_event(atom!("load"));
                        }),
                        window.upcast(),
                    );
                    return;
                }
            }
        }
        // step 7, await a stable state.
        self.generation.set(self.generation.get() + 1);
        let task = ImageElementMicrotask::StableStateUpdateImageDataTask {
            elem: DomRoot::from_ref(self),
            generation: self.generation.get(),
        };
        ScriptThread::await_stable_state(Microtask::ImageElement(task));
    }

    /// <https://html.spec.whatwg.org/multipage/#img-environment-changes>
    pub fn react_to_environment_changes(&self) {
        // Step 1
        let task = ImageElementMicrotask::EnvironmentChangesTask {
            elem: DomRoot::from_ref(self),
            generation: self.generation.get(),
        };
        ScriptThread::await_stable_state(Microtask::ImageElement(task));
    }

    /// Step 2-12 of https://html.spec.whatwg.org/multipage/#img-environment-changes
    fn react_to_environment_changes_sync_steps(&self, generation: u32) {
        // TODO reduce duplicacy of this code
        fn add_cache_listener_for_element(
            image_cache: Arc<dyn ImageCache>,
            id: PendingImageId,
            elem: &HTMLImageElement,
            selected_source: String,
            selected_pixel_density: f64,
        ) {
            let trusted_node = Trusted::new(elem);
            let (responder_sender, responder_receiver) = ipc::channel().unwrap();

            let window = window_from_node(elem);
            let (task_source, canceller) = window
                .task_manager()
                .networking_task_source_with_canceller();
            let generation = elem.generation.get();
            ROUTER.add_route(responder_receiver.to_opaque(), Box::new(move |message| {
                debug!("Got image {:?}", message);
                // Return the image via a message to the script thread, which marks
                // the element as dirty and triggers a reflow.
                let element = trusted_node.clone();
                let image = message.to().unwrap();
                let selected_source_clone = selected_source.clone();
                let _ = task_source.queue_with_canceller(
                    task!(process_image_response_for_environment_change: move || {
                        let element = element.root();
                        // Ignore any image response for a previous request that has been discarded.
                        if generation == element.generation.get() {
                            element.process_image_response_for_environment_change(image,
                            DOMString::from_string(selected_source_clone), generation, selected_pixel_density);
                        }
                    }),
                    &canceller,
                );
            }));

            image_cache.add_listener(id, ImageResponder::new(responder_sender, id));
        }

        let elem = self.upcast::<Element>();
        let document = document_from_node(elem);
        let has_pending_request = match self.image_request.get() {
            ImageRequestPhase::Pending => true,
            _ => false,
        };

        // Step 2
        if !document.is_active() || !Self::uses_srcset_or_picture(elem) || has_pending_request {
            return;
        }

        // Steps 3-4
        let (selected_source, selected_pixel_density) = match self.select_image_source() {
            Some(selected) => selected,
            None => return,
        };

        // Step 5
        let same_source = match *self.last_selected_source.borrow() {
            Some(ref last_src) => *last_src == selected_source,
            _ => false,
        };

        let same_selected_pixel_density = match self.current_request.borrow().current_pixel_density
        {
            Some(den) => selected_pixel_density == den,
            _ => false,
        };

        if same_source && same_selected_pixel_density {
            return;
        }

        let base_url = document.base_url();
        // Step 6
        let img_url = match base_url.join(&selected_source) {
            Ok(url) => url,
            Err(_) => return,
        };

        // Step 12
        self.image_request.set(ImageRequestPhase::Pending);
        self.init_image_request(
            &mut self.pending_request.borrow_mut(),
            &img_url,
            &selected_source,
        );

        let window = window_from_node(self);
        let image_cache = window.image_cache();
        // Step 14
        let response = image_cache.find_image_or_metadata(
            img_url.clone().into(),
            UsePlaceholder::No,
            CanRequestImages::Yes,
        );
        match response {
            Ok(ImageOrMetadataAvailable::ImageAvailable(_image, _url)) => {
                // Step 15
                self.finish_reacting_to_environment_change(
                    selected_source,
                    generation,
                    selected_pixel_density,
                );
            },

            Ok(ImageOrMetadataAvailable::MetadataAvailable(m)) => {
                self.process_image_response_for_environment_change(
                    ImageResponse::MetadataLoaded(m),
                    selected_source,
                    generation,
                    selected_pixel_density,
                );
            },

            Err(ImageState::Pending(id)) => {
                add_cache_listener_for_element(
                    image_cache.clone(),
                    id,
                    self,
                    selected_source.to_string(),
                    selected_pixel_density,
                );
            },

            Err(ImageState::LoadError) => {
                self.process_image_response_for_environment_change(
                    ImageResponse::None,
                    selected_source,
                    generation,
                    selected_pixel_density,
                );
            },

            Err(ImageState::NotRequested(id)) => {
                add_cache_listener_for_element(
                    image_cache,
                    id,
                    self,
                    selected_source.to_string(),
                    selected_pixel_density,
                );
                self.fetch_request(&img_url, id);
            },
        }
    }

    /// Step 15 for <https://html.spec.whatwg.org/multipage/#img-environment-changes>
    fn finish_reacting_to_environment_change(
        &self,
        src: DOMString,
        generation: u32,
        selected_pixel_density: f64,
    ) {
        let this = Trusted::new(self);
        let window = window_from_node(self);
        let src = src.to_string();
        let _ = window.task_manager().dom_manipulation_task_source().queue(
            task!(image_load_event: move || {
                let this = this.root();
                let relevant_mutation = this.generation.get() != generation;
                // Step 15.1
                if relevant_mutation {
                    this.abort_request(State::Unavailable, ImageRequestPhase::Pending);
                    return;
                }
                // Step 15.2
                *this.last_selected_source.borrow_mut() = Some(DOMString::from_string(src));

                {
                    let mut pending_request = this.pending_request.borrow_mut();
                    pending_request.current_pixel_density = Some(selected_pixel_density);

                    // Step 15.3
                    pending_request.state = State::CompletelyAvailable;

                    // Step 15.4
                    // Already a part of the list of available images due to Step 14

                    // Step 15.5
                    mem::swap(&mut this.current_request.borrow_mut(), &mut pending_request);
                    this.abort_request(State::Unavailable, ImageRequestPhase::Pending);
                }

                // Step 15.6
                this.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);

                // Step 15.7
                this.upcast::<EventTarget>().fire_event(atom!("load"));
            }),
            window.upcast(),
        );
    }

    fn uses_srcset_or_picture(elem: &Element) -> bool {
        let has_src = elem.has_attribute(&local_name!("srcset"));
        let is_parent_picture = elem
            .upcast::<Node>()
            .GetParentElement()
            .map_or(false, |p| p.is::<HTMLPictureElement>());
        has_src || is_parent_picture
    }

    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
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
                blocker: None,
                final_url: None,
                current_pixel_density: None,
            }),
            pending_request: DomRefCell::new(ImageRequest {
                state: State::Unavailable,
                parsed_url: None,
                source_url: None,
                image: None,
                metadata: None,
                blocker: None,
                final_url: None,
                current_pixel_density: None,
            }),
            form_owner: Default::default(),
            generation: Default::default(),
            source_set: DomRefCell::new(SourceSet::new()),
            last_selected_source: DomRefCell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> DomRoot<HTMLImageElement> {
        Node::reflect_node(
            Box::new(HTMLImageElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            HTMLImageElementBinding::Wrap,
        )
    }

    pub fn Image(
        window: &Window,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Fallible<DomRoot<HTMLImageElement>> {
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
    pub fn areas(&self) -> Option<Vec<DomRoot<HTMLAreaElement>>> {
        let elem = self.upcast::<Element>();
        let usemap_attr = elem.get_attribute(&ns!(), &local_name!("usemap"))?;

        let value = usemap_attr.value();

        if value.len() == 0 || !value.is_char_boundary(1) {
            return None;
        }

        let (first, last) = value.split_at(1);

        if first != "#" || last.len() == 0 {
            return None;
        }

        let useMapElements = document_from_node(self)
            .upcast::<Node>()
            .traverse_preorder()
            .filter_map(DomRoot::downcast::<HTMLMapElement>)
            .find(|n| {
                n.upcast::<Element>()
                    .get_string_attribute(&LocalName::from("name")) ==
                    last
            });

        useMapElements.map(|mapElem| mapElem.get_area_elements())
    }

    pub fn same_origin(&self, origin: &MutableOrigin) -> bool {
        self.current_request
            .borrow_mut()
            .final_url
            .as_ref()
            .map_or(false, |url| {
                url.scheme() == "data" || url.origin().same_origin(origin)
            })
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub enum ImageElementMicrotask {
    StableStateUpdateImageDataTask {
        elem: DomRoot<HTMLImageElement>,
        generation: u32,
    },
    EnvironmentChangesTask {
        elem: DomRoot<HTMLImageElement>,
        generation: u32,
    },
}

impl MicrotaskRunnable for ImageElementMicrotask {
    fn handler(&self) {
        match self {
            &ImageElementMicrotask::StableStateUpdateImageDataTask {
                ref elem,
                ref generation,
            } => {
                // Step 7 of https://html.spec.whatwg.org/multipage/#update-the-image-data,
                // stop here if other instances of this algorithm have been scheduled
                if elem.generation.get() == *generation {
                    elem.update_the_image_data_sync_steps();
                }
            },
            &ImageElementMicrotask::EnvironmentChangesTask {
                ref elem,
                ref generation,
            } => {
                elem.react_to_environment_changes_sync_steps(*generation);
            },
        }
    }
}

pub trait LayoutHTMLImageElementHelpers {
    #[allow(unsafe_code)]
    unsafe fn image(&self) -> Option<Arc<Image>>;

    #[allow(unsafe_code)]
    unsafe fn image_url(&self) -> Option<ServoUrl>;

    #[allow(unsafe_code)]
    unsafe fn image_density(&self) -> Option<f64>;

    #[allow(unsafe_code)]
    unsafe fn image_data(&self) -> (Option<Arc<Image>>, Option<ImageMetadata>);

    fn get_width(&self) -> LengthOrPercentageOrAuto;
    fn get_height(&self) -> LengthOrPercentageOrAuto;
}

impl LayoutHTMLImageElementHelpers for LayoutDom<HTMLImageElement> {
    #[allow(unsafe_code)]
    unsafe fn image(&self) -> Option<Arc<Image>> {
        (*self.unsafe_get())
            .current_request
            .borrow_for_layout()
            .image
            .clone()
    }

    #[allow(unsafe_code)]
    unsafe fn image_url(&self) -> Option<ServoUrl> {
        (*self.unsafe_get())
            .current_request
            .borrow_for_layout()
            .parsed_url
            .clone()
    }

    #[allow(unsafe_code)]
    unsafe fn image_data(&self) -> (Option<Arc<Image>>, Option<ImageMetadata>) {
        let current_request = (*self.unsafe_get()).current_request.borrow_for_layout();
        (
            current_request.image.clone(),
            current_request.metadata.clone(),
        )
    }

    #[allow(unsafe_code)]
    unsafe fn image_density(&self) -> Option<f64> {
        (*self.unsafe_get())
            .current_request
            .borrow_for_layout()
            .current_pixel_density
            .clone()
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

//https://html.spec.whatwg.org/multipage/#parse-a-sizes-attribute
pub fn parse_a_sizes_attribute(value: DOMString) -> SourceSizeList {
    let mut input = ParserInput::new(&value);
    let mut parser = Parser::new(&mut input);
    let url = ServoUrl::parse("about:blank").unwrap();
    let context = ParserContext::new(
        Origin::Author,
        &url,
        Some(CssRuleType::Style),
        // FIXME(emilio): why ::empty() instead of ::DEFAULT? Also, what do
        // browsers do regarding quirks-mode in a media list?
        ParsingMode::empty(),
        QuirksMode::NoQuirks,
        None,
        None,
    );
    SourceSizeList::parse(&context, &mut parser)
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

    // https://html.spec.whatwg.org/multipage/#dom-img-srcset
    make_getter!(Srcset, "srcset");
    // https://html.spec.whatwg.org/multipage/#dom-img-src
    make_setter!(SetSrcset, "srcset");

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
        let request = self.current_request.borrow();
        let pixel_density = request.current_pixel_density.unwrap_or(1f64);

        match request.metadata {
            Some(ref metadata) => (metadata.width as f64 / pixel_density) as u32,
            None => 0,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-naturalheight
    fn NaturalHeight(&self) -> u32 {
        let request = self.current_request.borrow();
        let pixel_density = request.current_pixel_density.unwrap_or(1f64);

        match request.metadata {
            Some(ref metadata) => (metadata.height as f64 / pixel_density) as u32,
            None => 0,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-complete
    fn Complete(&self) -> bool {
        let elem = self.upcast::<Element>();
        let srcset_absent = !elem.has_attribute(&local_name!("srcset"));
        if !elem.has_attribute(&local_name!("src")) && srcset_absent {
            return true;
        }
        let src = elem.get_string_attribute(&local_name!("src"));
        if srcset_absent && src.is_empty() {
            return true;
        }
        let request = self.current_request.borrow();
        let request_state = request.state;
        match request_state {
            State::CompletelyAvailable | State::Broken => return true,
            State::PartiallyAvailable | State::Unavailable => return false,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-currentsrc
    fn CurrentSrc(&self) -> DOMString {
        let current_request = self.current_request.borrow();
        let ref url = current_request.parsed_url;
        match *url {
            Some(ref url) => DOMString::from_string(url.clone().into_string()),
            None => {
                let ref unparsed_url = current_request.source_url;
                match *unparsed_url {
                    Some(ref url) => url.clone(),
                    None => DOMString::from(""),
                }
            },
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
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn adopting_steps(&self, old_doc: &Document) {
        self.super_type().unwrap().adopting_steps(old_doc);
        self.update_the_image_data();
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &local_name!("src") |
            &local_name!("srcset") |
            &local_name!("width") |
            &local_name!("crossorigin") |
            &local_name!("sizes") => self.update_the_image_data(),
            _ => {},
        }
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("name") => AttrValue::from_atomic(value.into()),
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

    fn handle_event(&self, event: &Event) {
        if event.type_() != atom!("click") {
            return;
        }

        let area_elements = self.areas();
        let elements = match area_elements {
            Some(x) => x,
            None => return,
        };

        // Fetch click coordinates
        let mouse_event = match event.downcast::<MouseEvent>() {
            Some(x) => x,
            None => return,
        };

        let point = Point2D::new(
            mouse_event.ClientX().to_f32().unwrap(),
            mouse_event.ClientY().to_f32().unwrap(),
        );
        let bcr = self.upcast::<Element>().GetBoundingClientRect();
        let bcr_p = Point2D::new(bcr.X() as f32, bcr.Y() as f32);

        // Walk HTMLAreaElements
        for element in elements {
            let shape = element.get_shape_from_coords();
            let shp = match shape {
                Some(x) => x.absolute_coords(bcr_p),
                None => return,
            };
            if shp.hit_test(&point) {
                element.activation_behavior(event, self.upcast());
                return;
            }
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }
        let document = document_from_node(self);
        if tree_in_doc {
            document.register_responsive_image(self);
        }

        // The element is inserted into a picture parent element
        // https://html.spec.whatwg.org/multipage/#relevant-mutations
        if let Some(parent) = self.upcast::<Node>().GetParentElement() {
            if parent.is::<HTMLPictureElement>() {
                self.update_the_image_data();
            }
        }
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);
        let document = document_from_node(self);
        document.unregister_responsive_image(self);

        // The element is removed from a picture parent element
        // https://html.spec.whatwg.org/multipage/#relevant-mutations
        if context.parent.is::<HTMLPictureElement>() {
            self.update_the_image_data();
        }
    }
}

impl FormControl for HTMLImageElement {
    fn form_owner(&self) -> Option<DomRoot<HTMLFormElement>> {
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
    let value = if value > UNSIGNED_LONG_MAX { 0 } else { value };

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

/// Collect sequence of code points
pub fn collect_sequence_characters<F>(s: &str, predicate: F) -> (&str, &str)
where
    F: Fn(&char) -> bool,
{
    for (i, ch) in s.chars().enumerate() {
        if !predicate(&ch) {
            return (&s[0..i], &s[i..]);
        }
    }

    return (s, "");
}

/// Parse an `srcset` attribute - https://html.spec.whatwg.org/multipage/#parsing-a-srcset-attribute.
pub fn parse_a_srcset_attribute(input: &str) -> Vec<ImageSource> {
    let mut url_len = 0;
    let mut candidates: Vec<ImageSource> = vec![];
    while url_len < input.len() {
        let position = &input[url_len..];
        let (spaces, position) =
            collect_sequence_characters(position, |c| *c == ',' || char::is_whitespace(*c));
        // add the length of the url that we parse to advance the start index
        let space_len = spaces.char_indices().count();
        url_len += space_len;
        if position.is_empty() {
            return candidates;
        }
        let (url, spaces) = collect_sequence_characters(position, |c| !char::is_whitespace(*c));
        // add the counts of urls that we parse to advance the start index
        url_len += url.chars().count();
        let comma_count = url.chars().rev().take_while(|c| *c == ',').count();
        let url: String = url
            .chars()
            .take(url.chars().count() - comma_count)
            .collect();
        // add 1 to start index, for the comma
        url_len += comma_count + 1;
        let (space, position) = collect_sequence_characters(spaces, |c| char::is_whitespace(*c));
        let space_len = space.len();
        url_len += space_len;
        let mut descriptors = Vec::new();
        let mut current_descriptor = String::new();
        let mut state = ParseState::InDescriptor;
        let mut char_stream = position.chars().enumerate();
        let mut buffered: Option<(usize, char)> = None;
        loop {
            let next_char = buffered.take().or_else(|| char_stream.next());
            if next_char.is_some() {
                url_len += 1;
            }
            match state {
                ParseState::InDescriptor => match next_char {
                    Some((_, ' ')) => {
                        if !current_descriptor.is_empty() {
                            descriptors.push(current_descriptor.clone());
                            current_descriptor = String::new();
                            state = ParseState::AfterDescriptor;
                        }
                        continue;
                    },
                    Some((_, ',')) => {
                        if !current_descriptor.is_empty() {
                            descriptors.push(current_descriptor.clone());
                        }
                        break;
                    },
                    Some((_, c @ '(')) => {
                        current_descriptor.push(c);
                        state = ParseState::InParens;
                        continue;
                    },
                    Some((_, c)) => {
                        current_descriptor.push(c);
                    },
                    None => {
                        if !current_descriptor.is_empty() {
                            descriptors.push(current_descriptor.clone());
                        }
                        break;
                    },
                },
                ParseState::InParens => match next_char {
                    Some((_, c @ ')')) => {
                        current_descriptor.push(c);
                        state = ParseState::InDescriptor;
                        continue;
                    },
                    Some((_, c)) => {
                        current_descriptor.push(c);
                        continue;
                    },
                    None => {
                        if !current_descriptor.is_empty() {
                            descriptors.push(current_descriptor.clone());
                        }
                        break;
                    },
                },
                ParseState::AfterDescriptor => match next_char {
                    Some((_, ' ')) => {
                        state = ParseState::AfterDescriptor;
                        continue;
                    },
                    Some((idx, c)) => {
                        state = ParseState::InDescriptor;
                        buffered = Some((idx, c));
                        continue;
                    },
                    None => {
                        if !current_descriptor.is_empty() {
                            descriptors.push(current_descriptor.clone());
                        }
                        break;
                    },
                },
            }
        }

        let mut error = false;
        let mut width: Option<u32> = None;
        let mut density: Option<f64> = None;
        let mut future_compat_h: Option<u32> = None;
        for descriptor in descriptors {
            let (digits, remaining) =
                collect_sequence_characters(&descriptor, |c| is_ascii_digit(c) || *c == '.');
            let valid_non_negative_integer = parse_unsigned_integer(digits.chars());
            let has_w = remaining == "w";
            let valid_floating_point = parse_double(digits);
            let has_x = remaining == "x";
            let has_h = remaining == "h";
            if valid_non_negative_integer.is_ok() && has_w {
                let result = valid_non_negative_integer;
                error = result.is_err();
                if width.is_some() || density.is_some() {
                    error = true;
                }
                if let Ok(w) = result {
                    width = Some(w);
                }
            } else if valid_floating_point.is_ok() && has_x {
                let result = valid_floating_point;
                error = result.is_err();
                if width.is_some() || density.is_some() || future_compat_h.is_some() {
                    error = true;
                }
                if let Ok(x) = result {
                    density = Some(x);
                }
            } else if valid_non_negative_integer.is_ok() && has_h {
                let result = valid_non_negative_integer;
                error = result.is_err();
                if density.is_some() || future_compat_h.is_some() {
                    error = true;
                }
                if let Ok(h) = result {
                    future_compat_h = Some(h);
                }
            } else {
                error = true;
            }
        }
        if future_compat_h.is_some() && width.is_none() {
            error = true;
        }
        if !error {
            let descriptor = Descriptor {
                wid: width,
                den: density,
            };
            let image_source = ImageSource {
                url: url,
                descriptor: descriptor,
            };
            candidates.push(image_source);
        }
    }
    candidates
}
