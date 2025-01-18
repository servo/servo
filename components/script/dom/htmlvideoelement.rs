/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::sync::Arc;

use dom_struct::dom_struct;
use euclid::default::Size2D;
use html5ever::{local_name, namespace_url, ns, LocalName, Prefix};
use ipc_channel::ipc;
use js::rust::HandleObject;
use net_traits::image_cache::{
    ImageCache, ImageCacheResult, ImageOrMetadataAvailable, ImageResponse, PendingImageId,
    UsePlaceholder,
};
use net_traits::request::{CredentialsMode, Destination, RequestBuilder, RequestId};
use net_traits::{
    FetchMetadata, FetchResponseListener, FetchResponseMsg, NetworkError, ResourceFetchTiming,
    ResourceTimingType,
};
use script_layout_interface::{HTMLMediaData, MediaMetadata};
use servo_media::player::video::VideoFrame;
use servo_url::ServoUrl;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};

use crate::document_loader::{LoadBlocker, LoadType};
use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLVideoElementBinding::HTMLVideoElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::{DomRoot, LayoutDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element, LayoutElementHelpers};
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlmediaelement::{HTMLMediaElement, ReadyState};
use crate::dom::node::{Node, NodeTraits};
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::virtualmethods::VirtualMethods;
use crate::fetch::FetchCanceller;
use crate::image_listener::{generate_cache_listener_for_element, ImageCacheListener};
use crate::network_listener::{self, PreInvoke, ResourceTimingListener};
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLVideoElement {
    htmlmediaelement: HTMLMediaElement,
    /// <https://html.spec.whatwg.org/multipage/#dom-video-videowidth>
    video_width: Cell<Option<u32>>,
    /// <https://html.spec.whatwg.org/multipage/#dom-video-videoheight>
    video_height: Cell<Option<u32>>,
    /// Incremented whenever tasks associated with this element are cancelled.
    generation_id: Cell<u32>,
    /// Load event blocker. Will block the load event while the poster frame
    /// is being fetched.
    load_blocker: DomRefCell<Option<LoadBlocker>>,
    /// A copy of the last frame
    #[ignore_malloc_size_of = "VideoFrame"]
    #[no_trace]
    last_frame: DomRefCell<Option<VideoFrame>>,
    /// Indicates if it has already sent a resize event for a given size
    sent_resize: Cell<Option<(u32, u32)>>,
}

impl HTMLVideoElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLVideoElement {
        HTMLVideoElement {
            htmlmediaelement: HTMLMediaElement::new_inherited(local_name, prefix, document),
            video_width: Cell::new(None),
            video_height: Cell::new(None),
            generation_id: Cell::new(0),
            load_blocker: Default::default(),
            last_frame: Default::default(),
            sent_resize: Cell::new(None),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLVideoElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLVideoElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        )
    }

    pub(crate) fn get_video_width(&self) -> Option<u32> {
        self.video_width.get()
    }

    pub(crate) fn get_video_height(&self) -> Option<u32> {
        self.video_height.get()
    }

    /// <https://html.spec.whatwg.org/multipage#event-media-resize>
    pub(crate) fn resize(&self, width: Option<u32>, height: Option<u32>) -> Option<(u32, u32)> {
        self.video_width.set(width);
        self.video_height.set(height);

        let width = width?;
        let height = height?;
        if self.sent_resize.get() == Some((width, height)) {
            return None;
        }

        let sent_resize = if self.htmlmediaelement.get_ready_state() == ReadyState::HaveNothing {
            None
        } else {
            self.owner_global()
                .task_manager()
                .media_element_task_source()
                .queue_simple_event(self.upcast(), atom!("resize"));
            Some((width, height))
        };

        self.sent_resize.set(sent_resize);
        sent_resize
    }

    pub(crate) fn get_current_frame_data(
        &self,
    ) -> Option<(Option<ipc::IpcSharedMemory>, Size2D<u32>)> {
        let frame = self.htmlmediaelement.get_current_frame();
        if frame.is_some() {
            *self.last_frame.borrow_mut() = frame;
        }

        match self.last_frame.borrow().as_ref() {
            Some(frame) => {
                let size = Size2D::new(frame.get_width() as u32, frame.get_height() as u32);
                if !frame.is_gl_texture() {
                    let data = Some(ipc::IpcSharedMemory::from_bytes(&frame.get_data()));
                    Some((data, size))
                } else {
                    // XXX(victor): here we only have the GL texture ID.
                    Some((None, size))
                }
            },
            None => None,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#poster-frame>
    fn fetch_poster_frame(&self, poster_url: &str, can_gc: CanGc) {
        // Step 1.
        self.generation_id.set(self.generation_id.get() + 1);

        // Step 2.
        if poster_url.is_empty() {
            return;
        }

        // Step 3.
        let poster_url = match self.owner_document().url().join(poster_url) {
            Ok(url) => url,
            Err(_) => return,
        };

        // Step 4.
        // We use the image cache for poster frames so we save as much
        // network activity as possible.
        let window = self.owner_window();
        let image_cache = window.image_cache();
        let sender = generate_cache_listener_for_element(self);
        let cache_result = image_cache.track_image(
            poster_url.clone(),
            window.origin().immutable().clone(),
            None,
            sender,
            UsePlaceholder::No,
        );

        match cache_result {
            ImageCacheResult::Available(ImageOrMetadataAvailable::ImageAvailable {
                image,
                url,
                ..
            }) => {
                self.process_image_response(ImageResponse::Loaded(image, url), can_gc);
            },
            ImageCacheResult::ReadyForRequest(id) => {
                self.do_fetch_poster_frame(poster_url, id, can_gc);
            },
            _ => (),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#poster-frame>
    fn do_fetch_poster_frame(&self, poster_url: ServoUrl, id: PendingImageId, can_gc: CanGc) {
        // Continuation of step 4.
        let document = self.owner_document();
        let request = RequestBuilder::new(poster_url.clone(), document.global().get_referrer())
            .destination(Destination::Image)
            .credentials_mode(CredentialsMode::Include)
            .use_url_credentials(true)
            .origin(document.origin().immutable().clone())
            .pipeline_id(Some(document.global().pipeline_id()));

        // Step 5.
        // This delay must be independent from the ones created by HTMLMediaElement during
        // its media load algorithm, otherwise a code like
        // <video poster="poster.png"></video>
        // (which triggers no media load algorithm unless a explicit call to .load() is done)
        // will block the document's load event forever.
        let blocker = &self.load_blocker;
        LoadBlocker::terminate(blocker, can_gc);
        *blocker.borrow_mut() = Some(LoadBlocker::new(
            &self.owner_document(),
            LoadType::Image(poster_url.clone()),
        ));

        let context = PosterFrameFetchContext::new(self, poster_url, id, request.id);
        self.owner_document().fetch_background(request, context);
    }
}

impl HTMLVideoElementMethods<crate::DomTypeHolder> for HTMLVideoElement {
    // https://html.spec.whatwg.org/multipage/#dom-video-videowidth
    fn VideoWidth(&self) -> u32 {
        if self.htmlmediaelement.get_ready_state() == ReadyState::HaveNothing {
            return 0;
        }
        self.video_width.get().unwrap_or(0)
    }

    // https://html.spec.whatwg.org/multipage/#dom-video-videoheight
    fn VideoHeight(&self) -> u32 {
        if self.htmlmediaelement.get_ready_state() == ReadyState::HaveNothing {
            return 0;
        }
        self.video_height.get().unwrap_or(0)
    }

    // https://html.spec.whatwg.org/multipage/#dom-video-poster
    make_getter!(Poster, "poster");

    // https://html.spec.whatwg.org/multipage/#dom-video-poster
    make_setter!(SetPoster, "poster");

    // For testing purposes only. This is not an event from
    // https://html.spec.whatwg.org/multipage/#dom-video-poster
    event_handler!(postershown, GetOnpostershown, SetOnpostershown);
}

impl VirtualMethods for HTMLVideoElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLMediaElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);

        if attr.local_name() == &local_name!("poster") {
            if let Some(new_value) = mutation.new_value(attr) {
                self.fetch_poster_frame(&new_value, CanGc::note())
            } else {
                self.htmlmediaelement.clear_current_frame_data();
                self.htmlmediaelement.set_show_poster(false);
            }
        };
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("width") | &local_name!("height") => {
                AttrValue::from_dimension(value.into())
            },
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }
}

impl ImageCacheListener for HTMLVideoElement {
    fn generation_id(&self) -> u32 {
        self.generation_id.get()
    }

    fn process_image_response(&self, response: ImageResponse, can_gc: CanGc) {
        match response {
            ImageResponse::Loaded(image, url) => {
                debug!("Loaded poster image for video element: {:?}", url);
                self.htmlmediaelement.process_poster_image_loaded(image);
                LoadBlocker::terminate(&self.load_blocker, can_gc);
            },
            ImageResponse::MetadataLoaded(..) => {},
            // The image cache may have loaded a placeholder for an invalid poster url
            ImageResponse::PlaceholderLoaded(..) | ImageResponse::None => {
                // A failed load should unblock the document load.
                LoadBlocker::terminate(&self.load_blocker, can_gc);
            },
        }
    }
}

struct PosterFrameFetchContext {
    /// Reference to the script thread image cache.
    image_cache: Arc<dyn ImageCache>,
    /// The element that initiated the request.
    elem: Trusted<HTMLVideoElement>,
    /// The cache ID for this request.
    id: PendingImageId,
    /// True if this response is invalid and should be ignored.
    cancelled: bool,
    /// Timing data for this resource
    resource_timing: ResourceFetchTiming,
    /// Url for the resource
    url: ServoUrl,
    /// A [`FetchCanceller`] for this request.
    fetch_canceller: FetchCanceller,
}

impl FetchResponseListener for PosterFrameFetchContext {
    fn process_request_body(&mut self, _: RequestId) {}

    fn process_request_eof(&mut self, _: RequestId) {
        self.fetch_canceller.ignore()
    }

    fn process_response(
        &mut self,
        request_id: RequestId,
        metadata: Result<FetchMetadata, NetworkError>,
    ) {
        self.image_cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessResponse(request_id, metadata.clone()),
        );

        let metadata = metadata.ok().map(|meta| match meta {
            FetchMetadata::Unfiltered(m) => m,
            FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
        });

        let status_is_ok = metadata
            .as_ref()
            .map_or(true, |m| m.status.in_range(200..300));

        if !status_is_ok {
            self.cancelled = true;
            self.fetch_canceller.cancel();
        }
    }

    fn process_response_chunk(&mut self, request_id: RequestId, payload: Vec<u8>) {
        if self.cancelled {
            // An error was received previously, skip processing the payload.
            return;
        }

        self.image_cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessResponseChunk(request_id, payload),
        );
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

impl ResourceTimingListener for PosterFrameFetchContext {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        let initiator_type = InitiatorType::LocalName(
            self.elem
                .root()
                .upcast::<Element>()
                .local_name()
                .to_string(),
        );
        (initiator_type, self.url.clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.elem.root().owner_document().global()
    }
}

impl PreInvoke for PosterFrameFetchContext {
    fn should_invoke(&self) -> bool {
        true
    }
}

impl PosterFrameFetchContext {
    fn new(
        elem: &HTMLVideoElement,
        url: ServoUrl,
        id: PendingImageId,
        request_id: RequestId,
    ) -> PosterFrameFetchContext {
        let window = elem.owner_window();
        PosterFrameFetchContext {
            image_cache: window.image_cache(),
            elem: Trusted::new(elem),
            id,
            cancelled: false,
            resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
            url,
            fetch_canceller: FetchCanceller::new(request_id),
        }
    }
}

pub(crate) trait LayoutHTMLVideoElementHelpers {
    fn data(self) -> HTMLMediaData;
    fn get_width(self) -> LengthOrPercentageOrAuto;
    fn get_height(self) -> LengthOrPercentageOrAuto;
}

impl LayoutDom<'_, HTMLVideoElement> {
    fn width_attr(self) -> Option<LengthOrPercentageOrAuto> {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("width"))
            .map(AttrValue::as_dimension)
            .cloned()
    }

    fn height_attr(self) -> Option<LengthOrPercentageOrAuto> {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("height"))
            .map(AttrValue::as_dimension)
            .cloned()
    }
}

impl LayoutHTMLVideoElementHelpers for LayoutDom<'_, HTMLVideoElement> {
    fn data(self) -> HTMLMediaData {
        let video = self.unsafe_get();

        // Get the current frame being rendered.
        let current_frame = video.htmlmediaelement.get_current_frame_data();

        // This value represents the natural width and height of the video.
        // It may exist even if there is no current frame (for example, after the
        // metadata of the video is loaded).
        let metadata = video
            .get_video_width()
            .zip(video.get_video_height())
            .map(|(width, height)| MediaMetadata { width, height });

        HTMLMediaData {
            current_frame,
            metadata,
        }
    }

    fn get_width(self) -> LengthOrPercentageOrAuto {
        self.width_attr().unwrap_or(LengthOrPercentageOrAuto::Auto)
    }

    fn get_height(self) -> LengthOrPercentageOrAuto {
        self.height_attr().unwrap_or(LengthOrPercentageOrAuto::Auto)
    }
}
