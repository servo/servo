/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::document_loader::{LoadBlocker, LoadType};
use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLVideoElementBinding;
use crate::dom::bindings::codegen::Bindings::HTMLVideoElementBinding::HTMLVideoElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlmediaelement::{HTMLMediaElement, ReadyState};
use crate::dom::node::{document_from_node, window_from_node, Node};
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::virtualmethods::VirtualMethods;
use crate::fetch::FetchCanceller;
use crate::image_listener::{add_cache_listener_for_element, ImageCacheListener};
use crate::network_listener::{self, NetworkListener, PreInvoke, ResourceTimingListener};
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use net_traits::image_cache::UsePlaceholder;
use net_traits::image_cache::{CanRequestImages, ImageCache, ImageOrMetadataAvailable};
use net_traits::image_cache::{ImageResponse, ImageState, PendingImageId};
use net_traits::request::{CredentialsMode, Destination, RequestBuilder};
use net_traits::{
    CoreResourceMsg, FetchChannels, FetchMetadata, FetchResponseListener, FetchResponseMsg,
};
use net_traits::{NetworkError, ResourceFetchTiming, ResourceTimingType};
use servo_url::ServoUrl;
use std::cell::Cell;
use std::sync::{Arc, Mutex};

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 150;

#[dom_struct]
pub struct HTMLVideoElement {
    htmlmediaelement: HTMLMediaElement,
    /// https://html.spec.whatwg.org/multipage/#dom-video-videowidth
    video_width: Cell<u32>,
    /// https://html.spec.whatwg.org/multipage/#dom-video-videoheight
    video_height: Cell<u32>,
    /// Incremented whenever tasks associated with this element are cancelled.
    generation_id: Cell<u32>,
    /// Poster frame fetch request canceller.
    poster_frame_canceller: DomRefCell<FetchCanceller>,
    /// Load event blocker. Will block the load event while the poster frame
    /// is being fetched.
    load_blocker: DomRefCell<Option<LoadBlocker>>,
}

impl HTMLVideoElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLVideoElement {
        HTMLVideoElement {
            htmlmediaelement: HTMLMediaElement::new_inherited(local_name, prefix, document),
            video_width: Cell::new(DEFAULT_WIDTH),
            video_height: Cell::new(DEFAULT_HEIGHT),
            generation_id: Cell::new(0),
            poster_frame_canceller: DomRefCell::new(Default::default()),
            load_blocker: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> DomRoot<HTMLVideoElement> {
        Node::reflect_node(
            Box::new(HTMLVideoElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            HTMLVideoElementBinding::Wrap,
        )
    }

    pub fn get_video_width(&self) -> u32 {
        self.video_width.get()
    }

    pub fn set_video_width(&self, width: u32) {
        self.video_width.set(width);
    }

    pub fn get_video_height(&self) -> u32 {
        self.video_height.get()
    }

    pub fn set_video_height(&self, height: u32) {
        self.video_height.set(height);
    }

    pub fn allow_load_event(&self) {
        LoadBlocker::terminate(&mut *self.load_blocker.borrow_mut());
    }

    /// https://html.spec.whatwg.org/multipage/#poster-frame
    fn fetch_poster_frame(&self, poster_url: &str) {
        // Step 1.
        let cancel_receiver = self.poster_frame_canceller.borrow_mut().initialize();
        self.generation_id.set(self.generation_id.get() + 1);

        // Step 2.
        if poster_url.is_empty() {
            return;
        }

        // Step 3.
        let poster_url = match document_from_node(self).url().join(&poster_url) {
            Ok(url) => url,
            Err(_) => return,
        };

        // Step 4.
        // We use the image cache for poster frames so we save as much
        // network activity as possible.
        let window = window_from_node(self);
        let image_cache = window.image_cache();
        let response = image_cache.find_image_or_metadata(
            poster_url.clone().into(),
            UsePlaceholder::No,
            CanRequestImages::Yes,
        );
        match response {
            Ok(ImageOrMetadataAvailable::ImageAvailable(image, url)) => {
                self.process_image_response(ImageResponse::Loaded(image, url));
            },

            Err(ImageState::Pending(id)) => {
                add_cache_listener_for_element(image_cache, id, self);
            },

            Err(ImageState::NotRequested(id)) => {
                add_cache_listener_for_element(image_cache, id, self);
                self.do_fetch_poster_frame(poster_url, id, cancel_receiver);
            },

            _ => (),
        }
    }

    /// https://html.spec.whatwg.org/multipage/#poster-frame
    fn do_fetch_poster_frame(
        &self,
        poster_url: ServoUrl,
        id: PendingImageId,
        cancel_receiver: ipc::IpcReceiver<()>,
    ) {
        // Continuation of step 4.
        let document = document_from_node(self);
        let request = RequestBuilder::new(poster_url.clone())
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
        let mut blocker = self.load_blocker.borrow_mut();
        LoadBlocker::terminate(&mut *blocker);
        *blocker = Some(LoadBlocker::new(
            &document_from_node(self),
            LoadType::Image(poster_url.clone()),
        ));

        let window = window_from_node(self);
        let context = Arc::new(Mutex::new(PosterFrameFetchContext::new(
            self, poster_url, id,
        )));

        let (action_sender, action_receiver) = ipc::channel().unwrap();
        let (task_source, canceller) = window
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
        let global = self.global();
        global
            .core_resource_thread()
            .send(CoreResourceMsg::Fetch(
                request,
                FetchChannels::ResponseMsg(action_sender, Some(cancel_receiver)),
            ))
            .unwrap();
    }
}

impl HTMLVideoElementMethods for HTMLVideoElement {
    // https://html.spec.whatwg.org/multipage/#dom-video-videowidth
    fn VideoWidth(&self) -> u32 {
        if self.htmlmediaelement.get_ready_state() == ReadyState::HaveNothing {
            return 0;
        }
        self.video_width.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-video-videoheight
    fn VideoHeight(&self) -> u32 {
        if self.htmlmediaelement.get_ready_state() == ReadyState::HaveNothing {
            return 0;
        }
        self.video_height.get()
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

        if let Some(new_value) = mutation.new_value(attr) {
            match attr.local_name() {
                &local_name!("poster") => {
                    self.fetch_poster_frame(&new_value);
                },
                _ => (),
            };
        }
    }
}

impl ImageCacheListener for HTMLVideoElement {
    fn generation_id(&self) -> u32 {
        self.generation_id.get()
    }

    fn process_image_response(&self, response: ImageResponse) {
        self.htmlmediaelement.process_poster_response(response);
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
}

impl FetchResponseListener for PosterFrameFetchContext {
    fn process_request_body(&mut self) {}
    fn process_request_eof(&mut self) {}

    fn process_response(&mut self, metadata: Result<FetchMetadata, NetworkError>) {
        self.image_cache
            .notify_pending_response(self.id, FetchResponseMsg::ProcessResponse(metadata.clone()));

        let metadata = metadata.ok().map(|meta| match meta {
            FetchMetadata::Unfiltered(m) => m,
            FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
        });

        let status_is_ok = metadata
            .as_ref()
            .and_then(|m| m.status.as_ref())
            .map_or(true, |s| s.0 >= 200 && s.0 < 300);

        if !status_is_ok {
            self.cancelled = true;
            self.elem
                .root()
                .poster_frame_canceller
                .borrow_mut()
                .cancel();
        }
    }

    fn process_response_chunk(&mut self, payload: Vec<u8>) {
        if self.cancelled {
            // An error was received previously, skip processing the payload.
            return;
        }

        self.image_cache
            .notify_pending_response(self.id, FetchResponseMsg::ProcessResponseChunk(payload));
    }

    fn process_response_eof(&mut self, response: Result<ResourceFetchTiming, NetworkError>) {
        self.elem.root().allow_load_event();
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
        document_from_node(&*self.elem.root()).global()
    }
}

impl PreInvoke for PosterFrameFetchContext {
    fn should_invoke(&self) -> bool {
        true
    }
}

impl PosterFrameFetchContext {
    fn new(elem: &HTMLVideoElement, url: ServoUrl, id: PendingImageId) -> PosterFrameFetchContext {
        let window = window_from_node(elem);
        PosterFrameFetchContext {
            image_cache: window.image_cache(),
            elem: Trusted::new(elem),
            id,
            cancelled: false,
            resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
            url,
        }
    }
}
