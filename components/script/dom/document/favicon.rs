/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use bytes::Bytes;
use embedder_traits::{Image as EmbedderImage, PixelFormat as EmbedderPixelFormat};
use js::context::JSContext;
use net_traits::image_cache::{
    Image, ImageCache, ImageCacheResponseCallback, ImageCacheResult, ImageLoadListener,
    ImageOrMetadataAvailable, ImageResponse, PendingImageId, RasterizationResult,
};
use net_traits::request::{Destination, RequestId};
use net_traits::{FetchMetadata, FetchResponseMsg, NetworkError, ResourceFetchTiming};
use pixels::PixelFormat;
use servo_base::generic_channel::GenericSharedMemory;
use servo_url::ServoUrl;
use webrender_api::units::DeviceIntSize;

use crate::dom::bindings::refcounted::Trusted;
use crate::dom::csp::{GlobalCspReporting, Violation};
use crate::dom::document::Document;
use crate::dom::globalscope::GlobalScope;
use crate::dom::node::NodeTraits;
use crate::dom::performance::performanceresourcetiming::InitiatorType;
use crate::fetch::fetch::create_a_potential_cors_request_with_claim;
use crate::fetch::network_listener::{
    FetchResponseListener, ResourceTimingListener, submit_timing,
};
use crate::url::ensure_blob_referenced_by_url_is_kept_alive;

pub(crate) struct FaviconFetchContext {
    /// A handle to the image cache for the [`Document`] of this context.
    pub(crate) image_cache: Arc<dyn ImageCache>,
    /// The [`PendingImageId`] of the favicon we are fetching.
    pub(crate) id: PendingImageId,
    /// The URL of the favicon that this context is fetching.
    pub(crate) url: ServoUrl,
    /// Whether or not the load of this favicon was successful (received a 2xx status code).
    pub(crate) successful: bool,
    /// Whether this is a fallback fetch for `/favicon.ico` rather than an explicit `<link rel="icon">`.
    pub(crate) is_fallback: bool,
}

impl FaviconFetchContext {
    pub(crate) fn new(
        image_cache: Arc<dyn ImageCache>,
        id: PendingImageId,
        url: ServoUrl,
        is_fallback: bool,
    ) -> Self {
        Self {
            image_cache,
            id,
            url,
            successful: true,
            is_fallback,
        }
    }
}

impl FetchResponseListener for FaviconFetchContext {
    fn process_request_body(&mut self, _: &GlobalScope, _: RequestId) {}

    fn process_response(
        &mut self,
        _: &mut JSContext,
        _: &GlobalScope,
        request_id: RequestId,
        metadata: Result<FetchMetadata, NetworkError>,
    ) {
        if let Ok(ref metadata) = metadata &&
            !metadata.metadata().status.is_success()
        {
            self.successful = false;
        }

        self.image_cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessResponse(request_id, metadata),
        );
    }

    fn process_response_chunk(
        &mut self,
        _: &mut JSContext,
        _: &GlobalScope,
        request_id: RequestId,
        chunk: Bytes,
    ) {
        self.image_cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessResponseChunk(request_id, chunk),
        );
    }

    fn process_response_eof(
        self,
        cx: &mut JSContext,
        global: &GlobalScope,
        request_id: RequestId,
        response: Result<(), NetworkError>,
        timing: ResourceFetchTiming,
    ) {
        let response = if !self.successful {
            Err(NetworkError::HttpError("Not an OK response".into()))
        } else {
            response
        };
        self.image_cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessResponseEOF(request_id, response.clone(), timing.clone()),
        );
        if !self.is_fallback {
            submit_timing(cx, global, &self, &response, &timing);
        }
    }

    fn process_csp_violations(
        &mut self,
        cx: &mut JSContext,
        global: &GlobalScope,
        _request_id: RequestId,
        violations: Vec<Violation>,
    ) {
        if !self.is_fallback {
            global.report_csp_violations(cx, violations, None, None);
        }
    }

    fn process_content_length(&mut self, request_id: RequestId, size: usize) {
        self.image_cache.notify_pending_response(
            self.id,
            FetchResponseMsg::ProcessContentLength(request_id, size),
        );
    }
}

impl ResourceTimingListener for FaviconFetchContext {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        (InitiatorType::LocalName("link".into()), self.url.clone())
    }
}

impl Document {
    /// Rasterizes a loaded favicon file if necessary and notifies the embedder about it.
    pub(crate) fn process_favicon_image(&self, image: Image, is_fallback: bool) {
        // If an icon link has been added in the meantime or a favicon is already present,
        // do not apply the fallback icon.
        if is_fallback && (self.ever_had_favicon_link() || self.favicon().is_some()) {
            return;
        }

        // TODO: Include the size attribute here
        let window = self.window();

        let send_rasterized_favicon_to_embedder = |raster_image: &pixels::RasterImage| {
            // Let's not worry about animated favicons...
            let frame = raster_image.first_frame();

            let format = match raster_image.format {
                PixelFormat::K8 => EmbedderPixelFormat::K8,
                PixelFormat::KA8 => EmbedderPixelFormat::KA8,
                PixelFormat::RGB8 => EmbedderPixelFormat::RGB8,
                PixelFormat::RGBA8 => EmbedderPixelFormat::RGBA8,
                PixelFormat::BGRA8 => EmbedderPixelFormat::BGRA8,
            };

            let embedder_image = EmbedderImage::new(
                frame.width,
                frame.height,
                Arc::new(GenericSharedMemory::from_arc_vec(
                    raster_image.bytes.clone(),
                )),
                raster_image.frames[0].byte_range.clone(),
                format,
            );
            self.set_favicon(embedder_image);
        };

        match image {
            Image::Raster(raster_image) => send_rasterized_favicon_to_embedder(&raster_image),
            Image::Vector(vector_image) => {
                // This size is completely arbitrary.
                let size = DeviceIntSize::new(250, 250);

                let image_cache = window.image_cache();
                match image_cache.rasterize_vector_image(vector_image.id, size, None) {
                    RasterizationResult::Image(raster_image) => {
                        send_rasterized_favicon_to_embedder(&raster_image);
                    },
                    RasterizationResult::PendingRasterization => {
                        // The rasterization callback will end up calling "process_favicon_image" again,
                        // but this time with a raster image.
                        let image_cache_sender = self
                            .register_favicon_image_cache_callback(vector_image.id, is_fallback);
                        image_cache.add_rasterization_complete_listener(
                            window.pipeline_id(),
                            vector_image.id,
                            size,
                            image_cache_sender,
                        );
                    },
                    RasterizationResult::InvalidSvgParameters => {},
                }
            },
        }
    }

    fn register_favicon_image_cache_callback(
        &self,
        id: PendingImageId,
        is_fallback: bool,
    ) -> ImageCacheResponseCallback {
        let trusted_doc = Trusted::new(self);
        let window = self.window();
        window.register_image_cache_listener(id, move |response, _| {
            let trusted_doc = trusted_doc.clone();
            let document = trusted_doc.root();
            let window = document.window();

            let ImageResponse::Loaded(image, _) = response.response else {
                return;
            };

            window
                .as_global_scope()
                .task_manager()
                .networking_task_source()
                .queue(task!(process_favicon_response: move || {
                    let document = trusted_doc.root();
                    document.process_favicon_image(image, is_fallback);
                }));
        })
    }

    /// Fetches `/favicon.ico` from the document's URL as a fallback when no
    /// `<link rel="icon">` was found in the page.
    ///
    /// From <https://html.spec.whatwg.org/multipage/#rel-icon>:
    ///
    /// > In the absence of a link with the icon keyword, for Document objects whose URL's scheme is
    /// > an HTTP(S) scheme, user agents may instead run these steps:
    /// > 1. Let request be a new request whose URL is the URL record obtained by resolving the URL
    /// >    "/favicon.ico" against the Document object's URL, client is the Document object's relevant
    /// >    settings object, destination is "image", credentials mode is "include", and whose
    /// >    use-URL-credentials flag is set.
    /// > 2. Fetch request, with processResponse set to the following steps given response response:
    /// >    - 2.1. Use response's unsafe response as an icon as if it had been declared using the icon
    /// >      keyword.
    pub(crate) fn maybe_fetch_fallback_favicon(&self) {
        if !self.window().is_top_level() {
            return;
        }
        if self.ever_had_favicon_link() {
            return;
        }
        if self.favicon().is_some() {
            return;
        }
        let document_url = self.url();
        if !matches!(document_url.scheme(), "http" | "https") {
            return;
        }

        let Ok(favicon_url) = document_url.join("/favicon.ico") else {
            return;
        };

        let window = self.window();
        let image_cache = window.image_cache();
        let cache_result = image_cache.get_cached_image_status(
            favicon_url.clone(),
            window.origin().immutable(),
            None,
        );

        match cache_result {
            ImageCacheResult::Available(ImageOrMetadataAvailable::ImageAvailable {
                image, ..
            }) => {
                self.process_favicon_image(image, true);
            },
            ImageCacheResult::Available(ImageOrMetadataAvailable::MetadataAvailable(_, id)) |
            ImageCacheResult::Pending(id) => {
                let sender = self.register_favicon_image_cache_callback(id, true);
                image_cache.add_listener(ImageLoadListener::new(sender, window.pipeline_id(), id));
            },
            ImageCacheResult::ReadyForRequest(id) => {
                let referrer = self.owner_global().get_referrer();
                let request = create_a_potential_cors_request_with_claim(
                    Some(self.webview_id()),
                    ensure_blob_referenced_by_url_is_kept_alive(
                        &self.owner_global(),
                        favicon_url.clone(),
                    ),
                    Destination::Image,
                    None,
                    None,
                    referrer,
                )
                .policy_container((*self.policy_container()).clone())
                .origin(window.origin().immutable().to_owned())
                .pipeline_id(Some(window.pipeline_id()))
                .synchronous(true);

                let sender = self.register_favicon_image_cache_callback(id, true);
                image_cache.add_listener(ImageLoadListener::new(sender, window.pipeline_id(), id));

                let fetch_context = FaviconFetchContext::new(image_cache, id, favicon_url, true);
                self.fetch_favicon(request, fetch_context);
            },
            ImageCacheResult::FailedToLoadOrDecode => {},
        };
    }
}
