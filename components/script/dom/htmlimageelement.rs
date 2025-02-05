/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::HashSet;
use std::default::Default;
use std::rc::Rc;
use std::sync::Arc;
use std::{char, mem};

use app_units::{Au, AU_PER_PX};
use cssparser::{Parser, ParserInput};
use dom_struct::dom_struct;
use euclid::Point2D;
use html5ever::{local_name, namespace_url, ns, LocalName, Prefix, QualName};
use js::jsapi::JSAutoRealm;
use js::rust::HandleObject;
use mime::{self, Mime};
use net_traits::http_status::HttpStatus;
use net_traits::image_cache::{
    ImageCache, ImageCacheResult, ImageOrMetadataAvailable, ImageResponder, ImageResponse,
    PendingImageId, UsePlaceholder,
};
use net_traits::request::{Destination, Initiator, RequestId};
use net_traits::{
    FetchMetadata, FetchResponseListener, FetchResponseMsg, NetworkError, ReferrerPolicy,
    ResourceFetchTiming, ResourceTimingType,
};
use num_traits::ToPrimitive;
use pixels::{CorsStatus, Image, ImageMetadata};
use servo_url::origin::MutableOrigin;
use servo_url::ServoUrl;
use style::attr::{parse_integer, parse_length, AttrValue, LengthOrPercentageOrAuto};
use style::context::QuirksMode;
use style::media_queries::MediaList;
use style::parser::ParserContext;
use style::stylesheets::{CssRuleType, Origin, UrlExtraData};
use style::values::specified::length::{Length, NoCalcLength};
use style::values::specified::source_size_list::SourceSizeList;
use style::values::specified::AbsoluteLength;
use style_traits::ParsingMode;
use url::Url;

use super::domexception::DOMErrorName;
use super::types::DOMException;
use crate::document_loader::{LoadBlocker, LoadType};
use crate::dom::activation::Activatable;
use crate::dom::attr::Attr;
use crate::dom::bindings::cell::{DomRefCell, RefMut};
use crate::dom::bindings::codegen::Bindings::DOMRectBinding::DOMRect_Binding::DOMRectMethods;
use crate::dom::bindings::codegen::Bindings::ElementBinding::Element_Binding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLImageElementBinding::HTMLImageElementMethods;
use crate::dom::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{DomRoot, LayoutDom, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::document::{determine_policy_for_token, Document};
use crate::dom::element::{
    cors_setting_for_element, referrer_policy_for_element, reflect_cross_origin_attribute,
    reflect_referrer_policy_attribute, set_cross_origin_attribute, AttributeMutation,
    CustomElementCreationMode, Element, ElementCreator, LayoutElementHelpers,
};
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlareaelement::HTMLAreaElement;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlformelement::{FormControl, HTMLFormElement};
use crate::dom::htmlmapelement::HTMLMapElement;
use crate::dom::htmlpictureelement::HTMLPictureElement;
use crate::dom::htmlsourceelement::HTMLSourceElement;
use crate::dom::mouseevent::MouseEvent;
use crate::dom::node::{BindContext, Node, NodeDamage, NodeTraits, ShadowIncluding, UnbindContext};
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::promise::Promise;
use crate::dom::values::UNSIGNED_LONG_MAX;
use crate::dom::virtualmethods::VirtualMethods;
use crate::dom::window::Window;
use crate::fetch::create_a_potential_cors_request;
use crate::microtask::{Microtask, MicrotaskRunnable};
use crate::network_listener::{self, PreInvoke, ResourceTimingListener};
use crate::realms::enter_realm;
use crate::script_runtime::CanGc;
use crate::script_thread::ScriptThread;

#[derive(Clone, Copy, Debug)]
enum ParseState {
    InDescriptor,
    InParens,
    AfterDescriptor,
}

pub(crate) struct SourceSet {
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
    pub width: Option<u32>,
    pub density: Option<f64>,
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
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct ImageRequest {
    state: State,
    #[no_trace]
    parsed_url: Option<ServoUrl>,
    source_url: Option<USVString>,
    blocker: DomRefCell<Option<LoadBlocker>>,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    image: Option<Arc<Image>>,
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
    #[ignore_malloc_size_of = "SourceSet"]
    source_set: DomRefCell<SourceSet>,
    last_selected_source: DomRefCell<Option<USVString>>,
    #[ignore_malloc_size_of = "promises are hard"]
    image_decode_promises: DomRefCell<Vec<Rc<Promise>>>,
}

impl HTMLImageElement {
    pub(crate) fn get_url(&self) -> Option<ServoUrl> {
        self.current_request.borrow().parsed_url.clone()
    }
    // https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument
    pub(crate) fn is_usable(&self) -> Fallible<bool> {
        // If image has an intrinsic width or intrinsic height (or both) equal to zero, then return bad.
        if let Some(image) = &self.current_request.borrow().image {
            if image.width == 0 || image.height == 0 {
                return Ok(false);
            }
        }

        match self.current_request.borrow().state {
            // If image's current request's state is broken, then throw an "InvalidStateError" DOMException.
            State::Broken => Err(Error::InvalidState),
            State::CompletelyAvailable => Ok(true),
            // If image is not fully decodable, then return bad.
            State::PartiallyAvailable | State::Unavailable => Ok(false),
        }
    }

    pub(crate) fn image_data(&self) -> Option<Arc<Image>> {
        self.current_request.borrow().image.clone()
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
    /// timing data for this resource
    resource_timing: ResourceFetchTiming,
    url: ServoUrl,
}

impl FetchResponseListener for ImageContext {
    fn process_request_body(&mut self, _: RequestId) {}
    fn process_request_eof(&mut self, _: RequestId) {}

    fn process_response(
        &mut self,
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
        if let Some(metadata) = metadata.as_ref() {
            if let Some(ref content_type) = metadata.content_type {
                let mime: Mime = content_type.clone().into_inner().into();
                if mime.type_() == mime::MULTIPART && mime.subtype().as_str() == "x-mixed-replace" {
                    self.aborted = true;
                }
            }
        }

        let status = metadata
            .as_ref()
            .map(|m| m.status.clone())
            .unwrap_or_else(HttpStatus::new_error);

        self.status = {
            if status.is_error() {
                Err(NetworkError::Internal(
                    "No http status code received".to_owned(),
                ))
            } else if status.is_success() {
                Ok(())
            } else {
                Err(NetworkError::Internal(format!(
                    "HTTP error code {}",
                    status.code()
                )))
            }
        };
    }

    fn process_response_chunk(&mut self, request_id: RequestId, payload: Vec<u8>) {
        if self.status.is_ok() {
            self.image_cache.notify_pending_response(
                self.id,
                FetchResponseMsg::ProcessResponseChunk(request_id, payload),
            );
        }
    }

    fn process_response_eof(
        &mut self,
        request_id: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    ) {
        self.image_cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessResponseEOF(request_id, response),
        );
    }

    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming {
        &mut self.resource_timing
    }

    fn resource_timing(&self) -> &ResourceFetchTiming {
        &self.resource_timing
    }

    fn submit_resource_timing(&mut self) {
        network_listener::submit_timing(self, CanGc::note())
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
        !self.aborted
    }
}

#[allow(non_snake_case)]
impl HTMLImageElement {
    /// Update the current image with a valid URL.
    fn fetch_image(&self, img_url: &ServoUrl, can_gc: CanGc) {
        let window = self.owner_window();

        let cache_result = window.image_cache().get_cached_image_status(
            img_url.clone(),
            window.origin().immutable().clone(),
            cors_setting_for_element(self.upcast()),
            UsePlaceholder::Yes,
        );

        match cache_result {
            ImageCacheResult::Available(ImageOrMetadataAvailable::ImageAvailable {
                image,
                url,
                is_placeholder,
            }) => {
                if is_placeholder {
                    self.process_image_response(
                        ImageResponse::PlaceholderLoaded(image, url),
                        can_gc,
                    )
                } else {
                    self.process_image_response(ImageResponse::Loaded(image, url), can_gc)
                }
            },
            ImageCacheResult::Available(ImageOrMetadataAvailable::MetadataAvailable(
                metadata,
                id,
            )) => {
                self.process_image_response(ImageResponse::MetadataLoaded(metadata), can_gc);
                self.register_image_cache_callback(id, ChangeType::Element);
            },
            ImageCacheResult::Pending(id) => {
                self.register_image_cache_callback(id, ChangeType::Element);
            },
            ImageCacheResult::ReadyForRequest(id) => {
                self.fetch_request(img_url, id);
                self.register_image_cache_callback(id, ChangeType::Element);
            },
            ImageCacheResult::LoadError => self.process_image_response(ImageResponse::None, can_gc),
        };
    }

    fn register_image_cache_callback(&self, id: PendingImageId, change_type: ChangeType) {
        let trusted_node = Trusted::new(self);
        let generation = self.generation_id();
        let window = self.owner_window();
        let sender = window.register_image_cache_listener(id, move |response| {
            let trusted_node = trusted_node.clone();
            let window = trusted_node.root().owner_window();
            let callback_type = change_type.clone();

            window
                .as_global_scope()
                .task_manager()
                .networking_task_source()
                .queue(task!(process_image_response: move || {
                let element = trusted_node.root();

                // Ignore any image response for a previous request that has been discarded.
                if generation != element.generation_id() {
                    return;
                }

                match callback_type {
                    ChangeType::Element => {
                        element.process_image_response(response.response, CanGc::note());
                    }
                    ChangeType::Environment { selected_source, selected_pixel_density } => {
                        element.process_image_response_for_environment_change(
                            response.response, selected_source, generation, selected_pixel_density, CanGc::note()
                        );
                    }
                }
            }));
        });

        window
            .image_cache()
            .add_listener(ImageResponder::new(sender, window.pipeline_id(), id));
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
            resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
            url: img_url.clone(),
        };

        // https://html.spec.whatwg.org/multipage/#update-the-image-data steps 17-20
        // This function is also used to prefetch an image in `script::dom::servoparser::prefetch`.
        let mut request = create_a_potential_cors_request(
            Some(window.webview_id()),
            img_url.clone(),
            Destination::Image,
            cors_setting_for_element(self.upcast()),
            None,
            document.global().get_referrer(),
            document.insecure_requests_policy(),
        )
        .origin(document.origin().immutable().clone())
        .pipeline_id(Some(document.global().pipeline_id()))
        .referrer_policy(referrer_policy_for_element(self.upcast()));

        if Self::uses_srcset_or_picture(self.upcast()) {
            request = request.initiator(Initiator::ImageSet);
        }

        // This is a background load because the load blocker already fulfills the
        // purpose of delaying the document's load event.
        document.fetch_background(request, context);
    }

    // Steps common to when an image has been loaded.
    fn handle_loaded_image(&self, image: Arc<Image>, url: ServoUrl, can_gc: CanGc) {
        self.current_request.borrow_mut().metadata = Some(ImageMetadata {
            height: image.height,
            width: image.width,
        });
        self.current_request.borrow_mut().final_url = Some(url);
        self.current_request.borrow_mut().image = Some(image);
        self.current_request.borrow_mut().state = State::CompletelyAvailable;
        LoadBlocker::terminate(&self.current_request.borrow().blocker, can_gc);
        // Mark the node dirty
        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
        self.resolve_image_decode_promises();
    }

    /// Step 24 of <https://html.spec.whatwg.org/multipage/#update-the-image-data>
    fn process_image_response(&self, image: ImageResponse, can_gc: CanGc) {
        // TODO: Handle multipart/x-mixed-replace
        let (trigger_image_load, trigger_image_error) = match (image, self.image_request.get()) {
            (ImageResponse::Loaded(image, url), ImageRequestPhase::Current) => {
                self.handle_loaded_image(image, url, can_gc);
                (true, false)
            },
            (ImageResponse::PlaceholderLoaded(image, url), ImageRequestPhase::Current) => {
                self.handle_loaded_image(image, url, can_gc);
                (false, true)
            },
            (ImageResponse::Loaded(image, url), ImageRequestPhase::Pending) => {
                self.abort_request(State::Unavailable, ImageRequestPhase::Pending, can_gc);
                self.image_request.set(ImageRequestPhase::Current);
                self.handle_loaded_image(image, url, can_gc);
                (true, false)
            },
            (ImageResponse::PlaceholderLoaded(image, url), ImageRequestPhase::Pending) => {
                self.abort_request(State::Unavailable, ImageRequestPhase::Pending, can_gc);
                self.image_request.set(ImageRequestPhase::Current);
                self.handle_loaded_image(image, url, can_gc);
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
                self.abort_request(State::Broken, ImageRequestPhase::Current, can_gc);
                (false, true)
            },
            (ImageResponse::None, ImageRequestPhase::Pending) => {
                self.abort_request(State::Broken, ImageRequestPhase::Current, can_gc);
                self.abort_request(State::Broken, ImageRequestPhase::Pending, can_gc);
                self.image_request.set(ImageRequestPhase::Current);
                (false, true)
            },
        };

        // Fire image.onload and loadend
        if trigger_image_load {
            // TODO: https://html.spec.whatwg.org/multipage/#fire-a-progress-event-or-event
            self.upcast::<EventTarget>()
                .fire_event(atom!("load"), can_gc);
            self.upcast::<EventTarget>()
                .fire_event(atom!("loadend"), can_gc);
        }

        // Fire image.onerror
        if trigger_image_error {
            self.upcast::<EventTarget>()
                .fire_event(atom!("error"), can_gc);
            self.upcast::<EventTarget>()
                .fire_event(atom!("loadend"), can_gc);
        }

        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }

    fn process_image_response_for_environment_change(
        &self,
        image: ImageResponse,
        src: USVString,
        generation: u32,
        selected_pixel_density: f64,
        can_gc: CanGc,
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
                self.abort_request(State::Unavailable, ImageRequestPhase::Pending, can_gc);
            },
        };
    }

    /// <https://html.spec.whatwg.org/multipage/#abort-the-image-request>
    fn abort_request(&self, state: State, request: ImageRequestPhase, can_gc: CanGc) {
        let mut request = match request {
            ImageRequestPhase::Current => self.current_request.borrow_mut(),
            ImageRequestPhase::Pending => self.pending_request.borrow_mut(),
        };
        LoadBlocker::terminate(&request.blocker, can_gc);
        request.state = state;
        request.image = None;
        request.metadata = None;

        if matches!(state, State::Broken) {
            self.reject_image_decode_promises();
        } else if matches!(state, State::CompletelyAvailable) {
            self.resolve_image_decode_promises();
        }
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
                    vec![DomRoot::from_ref(elem)]
                }
            },
            None => vec![DomRoot::from_ref(elem)],
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
            if *element == DomRoot::from_ref(elem) {
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
                    .all(|source| source.descriptor.density != Some(1.));
                let no_width_descriptor = source_set
                    .image_sources
                    .iter()
                    .all(|source| source.descriptor.width.is_none());
                if !is_src_empty && no_density_source_of_1 && no_width_descriptor {
                    source_set.image_sources.push(ImageSource {
                        url: src_attribute.to_string(),
                        descriptor: Descriptor {
                            width: None,
                            density: None,
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
        let document = self.owner_document();
        let quirks_mode = document.quirks_mode();
        let result = source_size_list.evaluate(document.window().layout().device(), quirks_mode);
        result
    }

    /// <https://html.spec.whatwg.org/multipage/#matches-the-environment>
    fn matches_environment(&self, media_query: String) -> bool {
        let document = self.owner_document();
        let quirks_mode = document.quirks_mode();
        let document_url_data = UrlExtraData(document.url().get_arc());
        // FIXME(emilio): This should do the same that we do for other media
        // lists regarding the rule type and such, though it doesn't really
        // matter right now...
        //
        // Also, ParsingMode::all() is wrong, and should be DEFAULT.
        let context = ParserContext::new(
            Origin::Author,
            &document_url_data,
            Some(CssRuleType::Style),
            ParsingMode::all(),
            quirks_mode,
            /* namespaces = */ Default::default(),
            None,
            None,
        );
        let mut parserInput = ParserInput::new(&media_query);
        let mut parser = Parser::new(&mut parserInput);
        let media_list = MediaList::parse(&context, &mut parser);
        let result = media_list.evaluate(document.window().layout().device(), quirks_mode);
        result
    }

    /// <https://html.spec.whatwg.org/multipage/#normalise-the-source-densities>
    fn normalise_source_densities(&self, source_set: &mut SourceSet, width: Option<Length>) {
        // Step 1
        let source_size = &mut source_set.source_size;

        // Find source_size_length for Step 2.2
        let source_size_length = self.evaluate_source_size_list(source_size, width);

        // Step 2
        for imgsource in &mut source_set.image_sources {
            // Step 2.1
            if imgsource.descriptor.density.is_some() {
                continue;
            }
            // Step 2.2
            if imgsource.descriptor.width.is_some() {
                let wid = imgsource.descriptor.width.unwrap();
                imgsource.descriptor.density = Some(wid as f64 / source_size_length.to_f64_px());
            } else {
                //Step 2.3
                imgsource.descriptor.density = Some(1_f64);
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#select-an-image-source>
    fn select_image_source(&self) -> Option<(USVString, f64)> {
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
            let pixel_density = imgsource.descriptor.density.unwrap();
            for inner_index in (outer_index + 1)..len {
                let imgsource2 = &source_set.image_sources[inner_index];
                if pixel_density == imgsource2.descriptor.density.unwrap() {
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
            let den = image_source.descriptor.density.unwrap();
            if max.0 < den {
                max = (den, img_sources.len());
            }
            img_sources.push(image_source);
        }

        // Step 5
        let mut best_candidate = max;
        let device_pixel_ratio = self
            .owner_document()
            .window()
            .window_size()
            .device_pixel_ratio
            .get() as f64;
        for (index, image_source) in img_sources.iter().enumerate() {
            let current_den = image_source.descriptor.density.unwrap();
            if current_den < best_candidate.0 && current_den >= device_pixel_ratio {
                best_candidate = (current_den, index);
            }
        }
        let selected_source = img_sources.remove(best_candidate.1).clone();
        Some((
            USVString(selected_source.url),
            selected_source.descriptor.density.unwrap(),
        ))
    }

    fn init_image_request(
        &self,
        request: &mut RefMut<ImageRequest>,
        url: &ServoUrl,
        src: &USVString,
        can_gc: CanGc,
    ) {
        request.parsed_url = Some(url.clone());
        request.source_url = Some(src.clone());
        request.image = None;
        request.metadata = None;
        let document = self.owner_document();
        LoadBlocker::terminate(&request.blocker, can_gc);
        *request.blocker.borrow_mut() =
            Some(LoadBlocker::new(&document, LoadType::Image(url.clone())));
    }

    /// Step 13-17 of html.spec.whatwg.org/multipage/#update-the-image-data
    fn prepare_image_request(
        &self,
        url: &ServoUrl,
        src: &USVString,
        selected_pixel_density: f64,
        can_gc: CanGc,
    ) {
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
                            LoadBlocker::terminate(&pending_request.blocker, can_gc);
                            // TODO: queue a task to restart animation, if restart-animation is set
                            return;
                        }
                        pending_request.current_pixel_density = Some(selected_pixel_density);
                        self.image_request.set(ImageRequestPhase::Pending);
                        self.init_image_request(&mut pending_request, url, src, can_gc);
                    },
                    (_, State::Broken) | (_, State::Unavailable) => {
                        // Step 17
                        current_request.current_pixel_density = Some(selected_pixel_density);
                        self.init_image_request(&mut current_request, url, src, can_gc);
                    },
                    (_, _) => {
                        // step 17
                        pending_request.current_pixel_density = Some(selected_pixel_density);
                        self.image_request.set(ImageRequestPhase::Pending);
                        self.init_image_request(&mut pending_request, url, src, can_gc);
                    },
                }
            },
        }
        self.fetch_image(url, can_gc);
    }

    /// Step 8-12 of html.spec.whatwg.org/multipage/#update-the-image-data
    fn update_the_image_data_sync_steps(&self, can_gc: CanGc) {
        let document = self.owner_document();
        let global = self.owner_global();
        let task_manager = global.task_manager();
        let task_source = task_manager.dom_manipulation_task_source();
        let this = Trusted::new(self);
        let (src, pixel_density) = match self.select_image_source() {
            // Step 8
            Some(data) => data,
            None => {
                self.abort_request(State::Broken, ImageRequestPhase::Current, can_gc);
                self.abort_request(State::Broken, ImageRequestPhase::Pending, can_gc);
                // Step 9.
                task_source.queue(task!(image_null_source_error: move || {
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
                        this.upcast::<EventTarget>().fire_event(atom!("error"), CanGc::note());
                    }
                }));
                return;
            },
        };

        // Step 11
        let base_url = document.base_url();
        let parsed_url = base_url.join(&src.0);
        match parsed_url {
            Ok(url) => {
                // Step 13-17
                self.prepare_image_request(&url, &src, pixel_density, can_gc);
            },
            Err(_) => {
                self.abort_request(State::Broken, ImageRequestPhase::Current, can_gc);
                self.abort_request(State::Broken, ImageRequestPhase::Pending, can_gc);
                // Step 12.1-12.5.
                let src = src.0;
                task_source.queue(task!(image_selected_source_error: move || {
                    let this = this.root();
                    {
                        let mut current_request =
                            this.current_request.borrow_mut();
                        current_request.source_url = Some(USVString(src))
                    }
                    this.upcast::<EventTarget>().fire_event(atom!("error"), CanGc::note());

                }));
            },
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#update-the-image-data>
    pub(crate) fn update_the_image_data(&self, can_gc: CanGc) {
        let document = self.owner_document();
        let window = document.window();
        let elem = self.upcast::<Element>();
        let src = elem.get_url_attribute(&local_name!("src"));
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
        let src_set = elem.get_url_attribute(&local_name!("srcset"));
        let is_parent_picture = elem
            .upcast::<Node>()
            .GetParentElement()
            .is_some_and(|p| p.is::<HTMLPictureElement>());
        if src_set.is_empty() && !is_parent_picture && !src.is_empty() {
            selected_source = Some(src.clone());
            pixel_density = Some(1_f64);
        };

        // Step 5
        self.last_selected_source
            .borrow_mut()
            .clone_from(&selected_source);

        // Step 6, check the list of available images
        if let Some(src) = selected_source {
            if let Ok(img_url) = base_url.join(&src) {
                let image_cache = window.image_cache();
                let response = image_cache.get_image(
                    img_url.clone(),
                    window.origin().immutable().clone(),
                    cors_setting_for_element(self.upcast()),
                );

                if let Some(image) = response {
                    // Cancel any outstanding tasks that were queued before the src was
                    // set on this element.
                    self.generation.set(self.generation.get() + 1);
                    // Step 6.3
                    let metadata = ImageMetadata {
                        height: image.height,
                        width: image.width,
                    };
                    // Step 6.3.2 abort requests
                    self.abort_request(
                        State::CompletelyAvailable,
                        ImageRequestPhase::Current,
                        can_gc,
                    );
                    self.abort_request(State::Unavailable, ImageRequestPhase::Pending, can_gc);
                    let mut current_request = self.current_request.borrow_mut();
                    current_request.final_url = Some(img_url.clone());
                    current_request.image = Some(image.clone());
                    current_request.metadata = Some(metadata);
                    // Step 6.3.6
                    current_request.current_pixel_density = pixel_density;
                    let this = Trusted::new(self);
                    let src = src.0;

                    self.owner_global()
                        .task_manager()
                        .dom_manipulation_task_source()
                        .queue(task!(image_load_event: move || {
                            let this = this.root();
                            {
                                let mut current_request =
                                    this.current_request.borrow_mut();
                                current_request.parsed_url = Some(img_url);
                                current_request.source_url = Some(USVString(src));
                            }
                            // TODO: restart animation, if set.
                            this.upcast::<EventTarget>().fire_event(atom!("load"), CanGc::note());
                        }));
                    return;
                }
            }
        }
        // step 7, await a stable state.
        self.generation.set(self.generation.get() + 1);
        let task = ImageElementMicrotask::StableStateUpdateImageData {
            elem: DomRoot::from_ref(self),
            generation: self.generation.get(),
        };
        ScriptThread::await_stable_state(Microtask::ImageElement(task));
    }

    /// <https://html.spec.whatwg.org/multipage/#img-environment-changes>
    pub(crate) fn react_to_environment_changes(&self) {
        // Step 1
        let task = ImageElementMicrotask::EnvironmentChanges {
            elem: DomRoot::from_ref(self),
            generation: self.generation.get(),
        };
        ScriptThread::await_stable_state(Microtask::ImageElement(task));
    }

    /// Step 2-12 of <https://html.spec.whatwg.org/multipage/#img-environment-changes>
    fn react_to_environment_changes_sync_steps(&self, generation: u32, can_gc: CanGc) {
        let elem = self.upcast::<Element>();
        let document = elem.owner_document();
        let has_pending_request = matches!(self.image_request.get(), ImageRequestPhase::Pending);

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
        let img_url = match base_url.join(&selected_source.0) {
            Ok(url) => url,
            Err(_) => return,
        };

        // Step 12
        self.image_request.set(ImageRequestPhase::Pending);
        self.init_image_request(
            &mut self.pending_request.borrow_mut(),
            &img_url,
            &selected_source,
            can_gc,
        );

        // Step 14
        let window = self.owner_window();
        let cache_result = window.image_cache().get_cached_image_status(
            img_url.clone(),
            window.origin().immutable().clone(),
            cors_setting_for_element(self.upcast()),
            UsePlaceholder::No,
        );

        let change_type = ChangeType::Environment {
            selected_source: selected_source.clone(),
            selected_pixel_density,
        };

        match cache_result {
            ImageCacheResult::Available(ImageOrMetadataAvailable::ImageAvailable { .. }) => {
                // Step 15
                self.finish_reacting_to_environment_change(
                    selected_source,
                    generation,
                    selected_pixel_density,
                )
            },
            ImageCacheResult::Available(ImageOrMetadataAvailable::MetadataAvailable(m, id)) => {
                self.process_image_response_for_environment_change(
                    ImageResponse::MetadataLoaded(m),
                    selected_source,
                    generation,
                    selected_pixel_density,
                    can_gc,
                );
                self.register_image_cache_callback(id, change_type);
            },
            ImageCacheResult::LoadError => {
                self.process_image_response_for_environment_change(
                    ImageResponse::None,
                    selected_source,
                    generation,
                    selected_pixel_density,
                    can_gc,
                );
            },
            ImageCacheResult::ReadyForRequest(id) => {
                self.fetch_request(&img_url, id);
                self.register_image_cache_callback(id, change_type);
            },
            ImageCacheResult::Pending(id) => {
                self.register_image_cache_callback(id, change_type);
            },
        }
    }

    // Step 2 for <https://html.spec.whatwg.org/multipage/#dom-img-decode>
    fn react_to_decode_image_sync_steps(&self, promise: Rc<Promise>) {
        let document = self.owner_document();
        // Step 2.1 of <https://html.spec.whatwg.org/multipage/#dom-img-decode>
        if !document.is_fully_active() ||
            matches!(self.current_request.borrow().state, State::Broken)
        {
            promise.reject_native(&DOMException::new(
                &document.global(),
                DOMErrorName::EncodingError,
            ));
        } else if matches!(
            self.current_request.borrow().state,
            State::CompletelyAvailable
        ) {
            // this doesn't follow the spec, but it's been discussed in <https://github.com/whatwg/html/issues/4217>
            promise.resolve_native(&());
        } else {
            self.image_decode_promises
                .borrow_mut()
                .push(promise.clone());
        }
    }

    fn resolve_image_decode_promises(&self) {
        for promise in self.image_decode_promises.borrow().iter() {
            promise.resolve_native(&());
        }
        self.image_decode_promises.borrow_mut().clear();
    }

    fn reject_image_decode_promises(&self) {
        let document = self.owner_document();
        for promise in self.image_decode_promises.borrow().iter() {
            promise.reject_native(&DOMException::new(
                &document.global(),
                DOMErrorName::EncodingError,
            ));
        }
        self.image_decode_promises.borrow_mut().clear();
    }

    /// Step 15 for <https://html.spec.whatwg.org/multipage/#img-environment-changes>
    fn finish_reacting_to_environment_change(
        &self,
        src: USVString,
        generation: u32,
        selected_pixel_density: f64,
    ) {
        let this = Trusted::new(self);
        let src = src.0;
        self.owner_global().task_manager().dom_manipulation_task_source().queue(
            task!(image_load_event: move || {
                let this = this.root();
                let relevant_mutation = this.generation.get() != generation;
                // Step 15.1
                if relevant_mutation {
                    this.abort_request(State::Unavailable, ImageRequestPhase::Pending, CanGc::note());
                    return;
                }
                // Step 15.2
                *this.last_selected_source.borrow_mut() = Some(USVString(src));

                {
                    let mut pending_request = this.pending_request.borrow_mut();
                    pending_request.current_pixel_density = Some(selected_pixel_density);

                    // Step 15.3
                    pending_request.state = State::CompletelyAvailable;

                    // Step 15.4
                    // Already a part of the list of available images due to Step 14

                    // Step 15.5
                    mem::swap(&mut this.current_request.borrow_mut(), &mut pending_request);
                }
                this.abort_request(State::Unavailable, ImageRequestPhase::Pending, CanGc::note());

                // Step 15.6
                this.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);

                // Step 15.7
                this.upcast::<EventTarget>().fire_event(atom!("load"), CanGc::note());
            })
        );
    }

    fn uses_srcset_or_picture(elem: &Element) -> bool {
        let has_src = elem.has_attribute(&local_name!("srcset"));
        let is_parent_picture = elem
            .upcast::<Node>()
            .GetParentElement()
            .is_some_and(|p| p.is::<HTMLPictureElement>());
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
            last_selected_source: DomRefCell::new(None),
            image_decode_promises: DomRefCell::new(vec![]),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLImageElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLImageElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        )
    }

    pub(crate) fn areas(&self) -> Option<Vec<DomRoot<HTMLAreaElement>>> {
        let elem = self.upcast::<Element>();
        let usemap_attr = elem.get_attribute(&ns!(), &local_name!("usemap"))?;

        let value = usemap_attr.value();

        if value.len() == 0 || !value.is_char_boundary(1) {
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
            return image.cors_status == CorsStatus::Safe;
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
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) enum ImageElementMicrotask {
    StableStateUpdateImageData {
        elem: DomRoot<HTMLImageElement>,
        generation: u32,
    },
    EnvironmentChanges {
        elem: DomRoot<HTMLImageElement>,
        generation: u32,
    },
    Decode {
        elem: DomRoot<HTMLImageElement>,
        #[ignore_malloc_size_of = "promises are hard"]
        promise: Rc<Promise>,
    },
}

impl MicrotaskRunnable for ImageElementMicrotask {
    fn handler(&self, can_gc: CanGc) {
        match *self {
            ImageElementMicrotask::StableStateUpdateImageData {
                ref elem,
                ref generation,
            } => {
                // Step 7 of https://html.spec.whatwg.org/multipage/#update-the-image-data,
                // stop here if other instances of this algorithm have been scheduled
                if elem.generation.get() == *generation {
                    elem.update_the_image_data_sync_steps(can_gc);
                }
            },
            ImageElementMicrotask::EnvironmentChanges {
                ref elem,
                ref generation,
            } => {
                elem.react_to_environment_changes_sync_steps(*generation, can_gc);
            },
            ImageElementMicrotask::Decode {
                ref elem,
                ref promise,
            } => {
                elem.react_to_decode_image_sync_steps(promise.clone());
            },
        }
    }

    fn enter_realm(&self) -> JSAutoRealm {
        match self {
            &ImageElementMicrotask::StableStateUpdateImageData { ref elem, .. } |
            &ImageElementMicrotask::EnvironmentChanges { ref elem, .. } |
            &ImageElementMicrotask::Decode { ref elem, .. } => enter_realm(&**elem),
        }
    }
}

pub(crate) trait LayoutHTMLImageElementHelpers {
    fn image_url(self) -> Option<ServoUrl>;
    fn image_density(self) -> Option<f64>;
    fn image_data(self) -> (Option<Arc<Image>>, Option<ImageMetadata>);
    fn get_width(self) -> LengthOrPercentageOrAuto;
    fn get_height(self) -> LengthOrPercentageOrAuto;
}

impl<'dom> LayoutDom<'dom, HTMLImageElement> {
    #[allow(unsafe_code)]
    fn current_request(self) -> &'dom ImageRequest {
        unsafe { self.unsafe_get().current_request.borrow_for_layout() }
    }
}

impl LayoutHTMLImageElementHelpers for LayoutDom<'_, HTMLImageElement> {
    fn image_url(self) -> Option<ServoUrl> {
        self.current_request().parsed_url.clone()
    }

    fn image_data(self) -> (Option<Arc<Image>>, Option<ImageMetadata>) {
        let current_request = self.current_request();
        (
            current_request.image.clone(),
            current_request.metadata.clone(),
        )
    }

    fn image_density(self) -> Option<f64> {
        self.current_request().current_pixel_density
    }

    fn get_width(self) -> LengthOrPercentageOrAuto {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("width"))
            .map(AttrValue::as_dimension)
            .cloned()
            .unwrap_or(LengthOrPercentageOrAuto::Auto)
    }

    fn get_height(self) -> LengthOrPercentageOrAuto {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("height"))
            .map(AttrValue::as_dimension)
            .cloned()
            .unwrap_or(LengthOrPercentageOrAuto::Auto)
    }
}

//https://html.spec.whatwg.org/multipage/#parse-a-sizes-attribute
pub(crate) fn parse_a_sizes_attribute(value: DOMString) -> SourceSizeList {
    let mut input = ParserInput::new(&value);
    let mut parser = Parser::new(&mut input);
    let url_data = Url::parse("about:blank").unwrap().into();
    let context = ParserContext::new(
        Origin::Author,
        &url_data,
        Some(CssRuleType::Style),
        // FIXME(emilio): why ::empty() instead of ::DEFAULT? Also, what do
        // browsers do regarding quirks-mode in a media list?
        ParsingMode::empty(),
        QuirksMode::NoQuirks,
        /* namespaces = */ Default::default(),
        None,
        None,
    );
    SourceSizeList::parse(&context, &mut parser)
}

fn get_correct_referrerpolicy_from_raw_token(token: &DOMString) -> DOMString {
    if token == "" {
        // Empty token is treated as the default referrer policy inside determine_policy_for_token,
        // so it should remain unchanged.
        DOMString::new()
    } else {
        let policy = determine_policy_for_token(token);

        if policy == ReferrerPolicy::EmptyString {
            return DOMString::new();
        }

        DOMString::from_string(policy.to_string())
    }
}

#[allow(non_snake_case)]
impl HTMLImageElementMethods<crate::DomTypeHolder> for HTMLImageElement {
    // https://html.spec.whatwg.org/multipage/#dom-image
    fn Image(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Fallible<DomRoot<HTMLImageElement>> {
        let element = Element::create(
            QualName::new(None, ns!(html), local_name!("img")),
            None,
            &window.Document(),
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Synchronous,
            proto,
            can_gc,
        );

        let image = DomRoot::downcast::<HTMLImageElement>(element).unwrap();
        if let Some(w) = width {
            image.SetWidth(w, can_gc);
        }
        if let Some(h) = height {
            image.SetHeight(h, can_gc);
        }

        // run update_the_image_data when the element is created.
        // https://html.spec.whatwg.org/multipage/#when-to-obtain-images
        image.update_the_image_data(can_gc);

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

    // https://html.spec.whatwg.org/multipage/#dom-img-crossOrigin
    fn GetCrossOrigin(&self) -> Option<DOMString> {
        reflect_cross_origin_attribute(self.upcast::<Element>())
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-crossOrigin
    fn SetCrossOrigin(&self, value: Option<DOMString>, can_gc: CanGc) {
        set_cross_origin_attribute(self.upcast::<Element>(), value, can_gc);
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
    fn Width(&self, can_gc: CanGc) -> u32 {
        let node = self.upcast::<Node>();
        match node.bounding_content_box(can_gc) {
            Some(rect) => rect.size.width.to_px() as u32,
            None => self.NaturalWidth(),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-width
    fn SetWidth(&self, value: u32, can_gc: CanGc) {
        image_dimension_setter(self.upcast(), local_name!("width"), value, can_gc);
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-height
    fn Height(&self, can_gc: CanGc) -> u32 {
        let node = self.upcast::<Node>();
        match node.bounding_content_box(can_gc) {
            Some(rect) => rect.size.height.to_px() as u32,
            None => self.NaturalHeight(),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-height
    fn SetHeight(&self, value: u32, can_gc: CanGc) {
        image_dimension_setter(self.upcast(), local_name!("height"), value, can_gc);
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
            State::CompletelyAvailable | State::Broken => true,
            State::PartiallyAvailable | State::Unavailable => false,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-img-currentsrc
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

    // https://html.spec.whatwg.org/multipage/#dom-img-referrerpolicy
    fn SetReferrerPolicy(&self, value: DOMString, can_gc: CanGc) {
        let referrerpolicy_attr_name = local_name!("referrerpolicy");
        let element = self.upcast::<Element>();
        let previous_correct_attribute_value = get_correct_referrerpolicy_from_raw_token(
            &element.get_string_attribute(&referrerpolicy_attr_name),
        );
        let correct_value_or_empty_string = get_correct_referrerpolicy_from_raw_token(&value);
        if previous_correct_attribute_value != correct_value_or_empty_string {
            // Setting the attribute to the same value will update the image.
            // We don't want to start an update if referrerpolicy is set to the same value.
            element.set_string_attribute(
                &referrerpolicy_attr_name,
                correct_value_or_empty_string,
                can_gc,
            );
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-img-decode>
    fn Decode(&self, can_gc: CanGc) -> Rc<Promise> {
        // Step 1
        let promise = Promise::new(&self.global(), can_gc);

        // Step 2
        let task = ImageElementMicrotask::Decode {
            elem: DomRoot::from_ref(self),
            promise: promise.clone(),
        };
        ScriptThread::await_stable_state(Microtask::ImageElement(task));

        // Step 3
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
        self.update_the_image_data(CanGc::note());
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &local_name!("src") |
            &local_name!("srcset") |
            &local_name!("width") |
            &local_name!("crossorigin") |
            &local_name!("sizes") |
            &local_name!("referrerpolicy") => self.update_the_image_data(CanGc::note()),
            _ => {},
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
        let bcr = self
            .upcast::<Element>()
            .GetBoundingClientRect(CanGc::note());
        let bcr_p = Point2D::new(bcr.X() as f32, bcr.Y() as f32);

        // Walk HTMLAreaElements
        for element in elements {
            let shape = element.get_shape_from_coords();
            let shp = match shape {
                Some(x) => x.absolute_coords(bcr_p),
                None => return,
            };
            if shp.hit_test(&point) {
                element.activation_behavior(event, self.upcast(), CanGc::note());
                return;
            }
        }
    }

    fn bind_to_tree(&self, context: &BindContext) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context);
        }
        let document = self.owner_document();
        if context.tree_connected {
            document.register_responsive_image(self);
        }

        // The element is inserted into a picture parent element
        // https://html.spec.whatwg.org/multipage/#relevant-mutations
        if let Some(parent) = self.upcast::<Node>().GetParentElement() {
            if parent.is::<HTMLPictureElement>() {
                self.update_the_image_data(CanGc::note());
            }
        }
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);
        let document = self.owner_document();
        document.unregister_responsive_image(self);

        // The element is removed from a picture parent element
        // https://html.spec.whatwg.org/multipage/#relevant-mutations
        if context.parent.is::<HTMLPictureElement>() {
            self.update_the_image_data(CanGc::note());
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

    fn to_element(&self) -> &Element {
        self.upcast::<Element>()
    }

    fn is_listed(&self) -> bool {
        false
    }
}

fn image_dimension_setter(element: &Element, attr: LocalName, value: u32, can_gc: CanGc) {
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
    element.set_attribute(&attr, value, can_gc);
}

/// Collect sequence of code points
pub(crate) fn collect_sequence_characters(
    s: &str,
    mut predicate: impl FnMut(&char) -> bool,
) -> (&str, &str) {
    let i = s.find(|ch| !predicate(&ch)).unwrap_or(s.len());
    (&s[0..i], &s[i..])
}

/// Parse an `srcset` attribute:
/// <https://html.spec.whatwg.org/multipage/#parsing-a-srcset-attribute>.
pub fn parse_a_srcset_attribute(input: &str) -> Vec<ImageSource> {
    // > 1. Let input be the value passed to this algorithm.
    // > 2. Let position be a pointer into input, initially pointing at the start of the string.
    let mut current_index = 0;

    // > 3. Let candidates be an initially empty source set.
    let mut candidates = vec![];
    while current_index < input.len() {
        let remaining_string = &input[current_index..];

        // > 4. Splitting loop: Collect a sequence of code points that are ASCII whitespace or
        // > U+002C COMMA characters from input given position. If any U+002C COMMA
        // > characters were collected, that is a parse error.
        let mut collected_comma = false;
        let (collected_characters, string_after_whitespace) =
            collect_sequence_characters(remaining_string, |character| {
                if *character == ',' {
                    collected_comma = true;
                }
                *character == ',' || character.is_ascii_whitespace()
            });
        if collected_comma {
            return Vec::new();
        }

        // Add the length of collected whitespace, to find the start of the URL we are going
        // to parse.
        current_index += collected_characters.len();

        // > 5. If position is past the end of input, return candidates.
        if string_after_whitespace.is_empty() {
            return candidates;
        }

        // 6. Collect a sequence of code points that are not ASCII whitespace from input
        // given position, and let that be url.
        let (url, _) =
            collect_sequence_characters(string_after_whitespace, |c| !char::is_ascii_whitespace(c));

        // Add the length of `url` that we will parse to advance the index of the next part
        // of the string to prase.
        current_index += url.len();

        // 7. Let descriptors be a new empty list.
        let mut descriptors = Vec::new();

        // > 8. If url ends with U+002C (,), then:
        // >    1. Remove all trailing U+002C COMMA characters from url. If this removed
        // >       more than one character, that is a parse error.
        if url.ends_with(',') {
            let image_source = ImageSource {
                url: url.trim_end_matches(',').into(),
                descriptor: Descriptor {
                    width: None,
                    density: None,
                },
            };
            candidates.push(image_source);
            continue;
        }

        // Otherwise:
        // > 8.1. Descriptor tokenizer: Skip ASCII whitespace within input given position.
        let descriptors_string = &input[current_index..];
        let (spaces, descriptors_string) =
            collect_sequence_characters(descriptors_string, |character| {
                character.is_ascii_whitespace()
            });
        current_index += spaces.len();

        // > 8.2. Let current descriptor be the empty string.
        let mut current_descriptor = String::new();

        // > 8.3. Let state be "in descriptor".
        let mut state = ParseState::InDescriptor;

        // > 8.4. Let c be the character at position. Do the following depending on the value of
        // > state. For the purpose of this step, "EOF" is a special character representing
        // > that position is past the end of input.
        let mut characters = descriptors_string.chars();
        let mut character = characters.next();
        if let Some(character) = character {
            current_index += character.len_utf8();
        }

        loop {
            match (state, character) {
                (ParseState::InDescriptor, Some(character)) if character.is_ascii_whitespace() => {
                    // > If current descriptor is not empty, append current descriptor to
                    // > descriptors and let current descriptor be the empty string. Set
                    // > state to after descriptor.
                    if !current_descriptor.is_empty() {
                        descriptors.push(current_descriptor);
                        current_descriptor = String::new();
                        state = ParseState::AfterDescriptor;
                    }
                },
                (ParseState::InDescriptor, Some(',')) => {
                    // > Advance position to the next character in input. If current descriptor
                    // > is not empty, append current descriptor to descriptors. Jump to the
                    // > step labeled descriptor parser.
                    if !current_descriptor.is_empty() {
                        descriptors.push(current_descriptor);
                    }
                    break;
                },
                (ParseState::InDescriptor, Some('(')) => {
                    // > Append c to current descriptor. Set state to in parens.
                    current_descriptor.push('(');
                    state = ParseState::InParens;
                },
                (ParseState::InDescriptor, Some(character)) => {
                    // > Append c to current descriptor.
                    current_descriptor.push(character);
                },
                (ParseState::InDescriptor, None) => {
                    // > If current descriptor is not empty, append current descriptor to
                    // > descriptors. Jump to the step labeled descriptor parser.
                    if !current_descriptor.is_empty() {
                        descriptors.push(current_descriptor);
                    }
                    break;
                },
                (ParseState::InParens, Some(')')) => {
                    // > Append c to current descriptor. Set state to in descriptor.
                    current_descriptor.push(')');
                    state = ParseState::InDescriptor;
                },
                (ParseState::InParens, Some(character)) => {
                    // Append c to current descriptor.
                    current_descriptor.push(character);
                },
                (ParseState::InParens, None) => {
                    // > Append current descriptor to descriptors. Jump to the step
                    // > labeled descriptor parser.
                    descriptors.push(current_descriptor);
                    break;
                },
                (ParseState::AfterDescriptor, Some(character))
                    if character.is_ascii_whitespace() =>
                {
                    // > Stay in this state.
                },
                (ParseState::AfterDescriptor, Some(_)) => {
                    // > Set state to in descriptor. Set position to the previous
                    // > character in input.
                    state = ParseState::InDescriptor;
                    continue;
                },
                (ParseState::AfterDescriptor, None) => {
                    // > Jump to the step labeled descriptor parser.
                    break;
                },
            }

            character = characters.next();
            if let Some(character) = character {
                current_index += character.len_utf8();
            }
        }

        // > 9. Descriptor parser: Let error be no.
        let mut error = false;
        // > 10. Let width be absent.
        let mut width: Option<u32> = None;
        // > 11. Let density be absent.
        let mut density: Option<f64> = None;
        // > 12. Let future-compat-h be absent.
        let mut future_compat_h: Option<u32> = None;

        // > 13. For each descriptor in descriptors, run the appropriate set of steps from
        // > the following list:
        for descriptor in descriptors.into_iter() {
            let Some(last_character) = descriptor.chars().last() else {
                break;
            };

            let first_part_of_string = &descriptor[0..descriptor.len() - last_character.len_utf8()];
            match last_character {
                // > If the descriptor consists of a valid non-negative integer followed by a
                // > U+0077 LATIN SMALL LETTER W character
                // > 1. If the user agent does not support the sizes attribute, let error be yes.
                // > 2. If width and density are not both absent, then let error be yes.
                // > 3. Apply the rules for parsing non-negative integers to the descriptor.
                // >    If the result is 0, let error be yes. Otherwise, let width be the result.
                'w' if density.is_none() && width.is_none() => {
                    match parse_integer(first_part_of_string.chars()) {
                        Ok(number) if number > 0 => {
                            width = Some(number as u32);
                            continue;
                        },
                        _ => error = true,
                    }
                },

                // > If the descriptor consists of a valid floating-point number followed by a
                // > U+0078 LATIN SMALL LETTER X character
                // > 1. If width, density and future-compat-h are not all absent, then let
                // >    error be yes.
                // > 2. Apply the rules for parsing floating-point number values to the
                // >    descriptor. If the result is less than 0, let error be yes. Otherwise, let
                // >    density be the result.
                //
                // The HTML specification has a procedure for parsing floats that is different enough from
                // the one that stylo uses, that it's better to use Rust's float parser here. This is
                // what Gecko does, but it also checks to see if the number is a valid HTML-spec compliant
                // number first. Not doing that means that we might be parsing numbers that otherwise
                // wouldn't parse.
                // TODO: Do what Gecko does and first validate the number passed to the Rust float parser.
                'x' if width.is_none() && density.is_none() && future_compat_h.is_none() => {
                    match first_part_of_string.parse::<f64>() {
                        Ok(number) if number.is_normal() && number > 0. => {
                            density = Some(number);
                            continue;
                        },
                        _ => error = true,
                    }
                },

                // > If the descriptor consists of a valid non-negative integer followed by a
                // > U+0068 LATIN SMALL LETTER H character
                // >   This is a parse error.
                // > 1. If future-compat-h and density are not both absent, then let error be
                // >    yes.
                // > 2. Apply the rules for parsing non-negative integers to the descriptor.
                // >    If the result is 0, let error be yes. Otherwise, let future-compat-h be the
                // >    result.
                'h' if future_compat_h.is_none() && density.is_none() => {
                    match parse_integer(first_part_of_string.chars()) {
                        Ok(number) if number > 0 => {
                            future_compat_h = Some(number as u32);
                            continue;
                        },
                        _ => error = true,
                    }
                },

                // > Anything else
                // >  Let error be yes.
                _ => error = true,
            }

            if error {
                break;
            }
        }

        // > 14. If future-compat-h is not absent and width is absent, let error be yes.
        if future_compat_h.is_some() && width.is_none() {
            error = true;
        }

        if !error {
            let image_source = ImageSource {
                url: url.into(),
                descriptor: Descriptor { width, density },
            };
            candidates.push(image_source);
        }
    }
    candidates
}

#[derive(Clone)]
enum ChangeType {
    Environment {
        selected_source: USVString,
        selected_pixel_density: f64,
    },
    Element,
}
