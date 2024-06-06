/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::{mem, thread};

use embedder_traits::resources::{self, Resource};
use imsz::imsz_from_reader;
use ipc_channel::ipc::IpcSender;
use log::{debug, warn};
use net_traits::image_cache::{
    ImageCache, ImageCacheResult, ImageOrMetadataAvailable, ImageResponder, ImageResponse,
    PendingImageId, PendingImageResponse, UsePlaceholder,
};
use net_traits::request::CorsSettings;
use net_traits::{FetchMetadata, FetchResponseMsg, FilteredMetadata, NetworkError};
use pixels::{load_from_memory, CorsStatus, Image, ImageMetadata, PixelFormat};
use servo_url::{ImmutableOrigin, ServoUrl};
use webrender_api::units::DeviceIntSize;
use webrender_api::{ImageData, ImageDescriptor, ImageDescriptorFlags, ImageFormat};
use webrender_traits::WebRenderNetApi;

use crate::resource_thread::CoreResourceThreadPool;

///
/// TODO(gw): Remaining work on image cache:
///     * Make use of the prefetch support in various parts of the code.
///     * Profile time in GetImageIfAvailable - might be worth caching these
///       results per paint / layout.
///
/// MAYBE(Yoric):
///     * For faster lookups, it might be useful to store the LoadKey in the
///       DOM once we have performed a first load.

// ======================================================================
// Helper functions.
// ======================================================================

fn decode_bytes_sync(key: LoadKey, bytes: &[u8], cors: CorsStatus) -> DecoderMsg {
    let image = load_from_memory(bytes, cors);
    DecoderMsg { key, image }
}

fn get_placeholder_image(webrender_api: &WebRenderNetApi, data: &[u8]) -> Arc<Image> {
    let mut image = load_from_memory(data, CorsStatus::Unsafe).unwrap();
    set_webrender_image_key(webrender_api, &mut image);
    Arc::new(image)
}

fn set_webrender_image_key(webrender_api: &WebRenderNetApi, image: &mut Image) {
    if image.id.is_some() {
        return;
    }
    let mut bytes = Vec::new();
    let is_opaque = match image.format {
        PixelFormat::BGRA8 => {
            bytes.extend_from_slice(&image.bytes);
            pixels::rgba8_premultiply_inplace(bytes.as_mut_slice())
        },
        PixelFormat::RGB8 => {
            bytes.reserve(image.bytes.len() / 3 * 4);
            for bgr in image.bytes.chunks(3) {
                bytes.extend_from_slice(&[bgr[2], bgr[1], bgr[0], 0xff]);
            }

            true
        },
        PixelFormat::K8 | PixelFormat::KA8 | PixelFormat::RGBA8 => {
            panic!("Not support by webrender yet");
        },
    };
    let mut flags = ImageDescriptorFlags::ALLOW_MIPMAPS;
    flags.set(ImageDescriptorFlags::IS_OPAQUE, is_opaque);
    let descriptor = ImageDescriptor {
        size: DeviceIntSize::new(image.width as i32, image.height as i32),
        stride: None,
        format: ImageFormat::BGRA8,
        offset: 0,
        flags,
    };
    let data = ImageData::new(bytes);
    let image_key = webrender_api.generate_image_key();
    webrender_api.add_image(image_key, descriptor, data);
    image.id = Some(image_key);
}

// ======================================================================
// Aux structs and enums.
// ======================================================================

/// <https://html.spec.whatwg.org/multipage/#list-of-available-images>
type ImageKey = (ServoUrl, ImmutableOrigin, Option<CorsSettings>);

// Represents all the currently pending loads/decodings. For
// performance reasons, loads are indexed by a dedicated load key.
struct AllPendingLoads {
    // The loads, indexed by a load key. Used during most operations,
    // for performance reasons.
    loads: HashMap<LoadKey, PendingLoad>,

    // Get a load key from its url and requesting origin. Used ony when starting and
    // finishing a load or when adding a new listener.
    url_to_load_key: HashMap<ImageKey, LoadKey>,

    // A counter used to generate instances of LoadKey
    keygen: LoadKeyGenerator,
}

impl AllPendingLoads {
    fn new() -> AllPendingLoads {
        AllPendingLoads {
            loads: HashMap::new(),
            url_to_load_key: HashMap::new(),
            keygen: LoadKeyGenerator::new(),
        }
    }

    // get a PendingLoad from its LoadKey.
    fn get_by_key_mut(&mut self, key: &LoadKey) -> Option<&mut PendingLoad> {
        self.loads.get_mut(key)
    }

    fn remove(&mut self, key: &LoadKey) -> Option<PendingLoad> {
        self.loads.remove(key).map(|pending_load| {
            self.url_to_load_key
                .remove(&(
                    pending_load.url.clone(),
                    pending_load.load_origin.clone(),
                    pending_load.cors_setting,
                ))
                .unwrap();
            pending_load
        })
    }

    fn get_cached(
        &mut self,
        url: ServoUrl,
        origin: ImmutableOrigin,
        cors_status: Option<CorsSettings>,
    ) -> CacheResult<'_> {
        match self
            .url_to_load_key
            .entry((url.clone(), origin.clone(), cors_status))
        {
            Occupied(url_entry) => {
                let load_key = url_entry.get();
                CacheResult::Hit(*load_key, self.loads.get_mut(load_key).unwrap())
            },
            Vacant(url_entry) => {
                let load_key = self.keygen.next();
                url_entry.insert(load_key);

                let pending_load = PendingLoad::new(url, origin, cors_status);
                match self.loads.entry(load_key) {
                    Occupied(_) => unreachable!(),
                    Vacant(load_entry) => {
                        let mut_load = load_entry.insert(pending_load);
                        CacheResult::Miss(Some((load_key, mut_load)))
                    },
                }
            },
        }
    }
}

/// Result of accessing a cache.
enum CacheResult<'a> {
    /// The value was in the cache.
    Hit(LoadKey, &'a mut PendingLoad),
    /// The value was not in the cache and needed to be regenerated.
    Miss(Option<(LoadKey, &'a mut PendingLoad)>),
}

/// Represents an image that has completed loading.
/// Images that fail to load (due to network or decode
/// failure) are still stored here, so that they aren't
/// fetched again.
struct CompletedLoad {
    image_response: ImageResponse,
    id: PendingImageId,
}

impl CompletedLoad {
    fn new(image_response: ImageResponse, id: PendingImageId) -> CompletedLoad {
        CompletedLoad { image_response, id }
    }
}

/// Message that the decoder worker threads send to the image cache.
struct DecoderMsg {
    key: LoadKey,
    image: Option<Image>,
}

enum ImageBytes {
    InProgress(Vec<u8>),
    Complete(Arc<Vec<u8>>),
}

impl ImageBytes {
    fn extend_from_slice(&mut self, data: &[u8]) {
        match *self {
            ImageBytes::InProgress(ref mut bytes) => bytes.extend_from_slice(data),
            ImageBytes::Complete(_) => panic!("attempted modification of complete image bytes"),
        }
    }

    fn mark_complete(&mut self) -> Arc<Vec<u8>> {
        let bytes = {
            let own_bytes = match *self {
                ImageBytes::InProgress(ref mut bytes) => bytes,
                ImageBytes::Complete(_) => panic!("attempted modification of complete image bytes"),
            };
            mem::take(own_bytes)
        };
        let bytes = Arc::new(bytes);
        *self = ImageBytes::Complete(bytes.clone());
        bytes
    }

    fn as_slice(&self) -> &[u8] {
        match *self {
            ImageBytes::InProgress(ref bytes) => bytes,
            ImageBytes::Complete(ref bytes) => bytes,
        }
    }
}

// A key used to communicate during loading.
type LoadKey = PendingImageId;

struct LoadKeyGenerator {
    counter: u64,
}

impl LoadKeyGenerator {
    fn new() -> LoadKeyGenerator {
        LoadKeyGenerator { counter: 0 }
    }
    fn next(&mut self) -> PendingImageId {
        self.counter += 1;
        PendingImageId(self.counter)
    }
}

#[derive(Debug)]
enum LoadResult {
    Loaded(Image),
    PlaceholderLoaded(Arc<Image>),
    None,
}

/// Represents an image that is either being loaded
/// by the resource thread, or decoded by a worker thread.
struct PendingLoad {
    /// The bytes loaded so far. Reset to an empty vector once loading
    /// is complete and the buffer has been transmitted to the decoder.
    bytes: ImageBytes,

    /// Image metadata, if available.
    metadata: Option<ImageMetadata>,

    /// Once loading is complete, the result of the operation.
    result: Option<Result<(), NetworkError>>,

    /// The listeners that are waiting for this response to complete.
    listeners: Vec<ImageResponder>,

    /// The url being loaded. Do not forget that this may be several Mb
    /// if we are loading a data: url.
    url: ServoUrl,

    /// The origin that requested this load.
    load_origin: ImmutableOrigin,

    /// The CORS attribute setting for the requesting
    cors_setting: Option<CorsSettings>,

    /// The CORS status of this image response.
    cors_status: CorsStatus,

    /// The URL of the final response that contains a body.
    final_url: Option<ServoUrl>,
}

impl PendingLoad {
    fn new(
        url: ServoUrl,
        load_origin: ImmutableOrigin,
        cors_setting: Option<CorsSettings>,
    ) -> PendingLoad {
        PendingLoad {
            bytes: ImageBytes::InProgress(vec![]),
            metadata: None,
            result: None,
            listeners: vec![],
            url,
            load_origin,
            final_url: None,
            cors_setting,
            cors_status: CorsStatus::Unsafe,
        }
    }

    fn add_listener(&mut self, listener: ImageResponder) {
        self.listeners.push(listener);
    }
}

// ======================================================================
// Image cache implementation.
// ======================================================================
struct ImageCacheStore {
    // Images that are loading over network, or decoding.
    pending_loads: AllPendingLoads,

    // Images that have finished loading (successful or not)
    completed_loads: HashMap<ImageKey, CompletedLoad>,

    // The placeholder image used when an image fails to load
    placeholder_image: Arc<Image>,

    // The URL used for the placeholder image
    placeholder_url: ServoUrl,

    // Webrender API instance.
    webrender_api: WebRenderNetApi,
}

impl ImageCacheStore {
    // Change state of a url from pending -> loaded.
    fn complete_load(&mut self, key: LoadKey, mut load_result: LoadResult) {
        debug!("Completed decoding for {:?}", load_result);
        let pending_load = match self.pending_loads.remove(&key) {
            Some(load) => load,
            None => return,
        };

        match load_result {
            LoadResult::Loaded(ref mut image) => {
                set_webrender_image_key(&self.webrender_api, image)
            },
            LoadResult::PlaceholderLoaded(..) | LoadResult::None => {},
        }

        let url = pending_load.final_url.clone();
        let image_response = match load_result {
            LoadResult::Loaded(image) => ImageResponse::Loaded(Arc::new(image), url.unwrap()),
            LoadResult::PlaceholderLoaded(image) => {
                ImageResponse::PlaceholderLoaded(image, self.placeholder_url.clone())
            },
            LoadResult::None => ImageResponse::None,
        };

        let completed_load = CompletedLoad::new(image_response.clone(), key);
        self.completed_loads.insert(
            (
                pending_load.url,
                pending_load.load_origin,
                pending_load.cors_setting,
            ),
            completed_load,
        );

        for listener in pending_load.listeners {
            listener.respond(image_response.clone());
        }
    }

    /// Return a completed image if it exists, or None if there is no complete load
    /// or the complete load is not fully decoded or is unavailable.
    fn get_completed_image_if_available(
        &self,
        url: ServoUrl,
        origin: ImmutableOrigin,
        cors_setting: Option<CorsSettings>,
        placeholder: UsePlaceholder,
    ) -> Option<Result<(Arc<Image>, ServoUrl), ()>> {
        self.completed_loads
            .get(&(url, origin, cors_setting))
            .map(
                |completed_load| match (&completed_load.image_response, placeholder) {
                    (&ImageResponse::Loaded(ref image, ref url), _) |
                    (
                        &ImageResponse::PlaceholderLoaded(ref image, ref url),
                        UsePlaceholder::Yes,
                    ) => Ok((image.clone(), url.clone())),
                    (&ImageResponse::PlaceholderLoaded(_, _), UsePlaceholder::No) |
                    (&ImageResponse::None, _) |
                    (&ImageResponse::MetadataLoaded(_), _) => Err(()),
                },
            )
    }

    /// Handle a message from one of the decoder worker threads or from a sync
    /// decoding operation.
    fn handle_decoder(&mut self, msg: DecoderMsg) {
        let image = match msg.image {
            None => LoadResult::None,
            Some(image) => LoadResult::Loaded(image),
        };
        self.complete_load(msg.key, image);
    }
}

pub struct ImageCacheImpl {
    store: Arc<Mutex<ImageCacheStore>>,

    /// Thread pool for image decoding
    thread_pool: CoreResourceThreadPool,
}

impl ImageCache for ImageCacheImpl {
    fn new(webrender_api: WebRenderNetApi) -> ImageCacheImpl {
        debug!("New image cache");

        let rippy_data = resources::read_bytes(Resource::RippyPNG);
        // Uses an estimate of the system cpus to decode images
        // See https://doc.rust-lang.org/stable/std/thread/fn.available_parallelism.html
        // If no information can be obtained about the system, uses 4 threads as a default
        let thread_count = thread::available_parallelism()
            .unwrap_or(NonZeroUsize::new(4).unwrap())
            .get();

        ImageCacheImpl {
            store: Arc::new(Mutex::new(ImageCacheStore {
                pending_loads: AllPendingLoads::new(),
                completed_loads: HashMap::new(),
                placeholder_image: get_placeholder_image(&webrender_api, &rippy_data),
                placeholder_url: ServoUrl::parse("chrome://resources/rippy.png").unwrap(),
                webrender_api,
            })),
            thread_pool: CoreResourceThreadPool::new(thread_count),
        }
    }

    fn get_image(
        &self,
        url: ServoUrl,
        origin: ImmutableOrigin,
        cors_setting: Option<CorsSettings>,
    ) -> Option<Arc<Image>> {
        let store = self.store.lock().unwrap();
        let result =
            store.get_completed_image_if_available(url, origin, cors_setting, UsePlaceholder::No);
        match result {
            Some(Ok((img, _))) => Some(img),
            _ => None,
        }
    }

    fn get_cached_image_status(
        &self,
        url: ServoUrl,
        origin: ImmutableOrigin,
        cors_setting: Option<CorsSettings>,
        use_placeholder: UsePlaceholder,
    ) -> ImageCacheResult {
        let mut store = self.store.lock().unwrap();
        if let Some(result) = store.get_completed_image_if_available(
            url.clone(),
            origin.clone(),
            cors_setting,
            use_placeholder,
        ) {
            match result {
                Ok((image, image_url)) => {
                    debug!("{} is available", url);
                    let is_placeholder = image_url == store.placeholder_url;
                    return ImageCacheResult::Available(ImageOrMetadataAvailable::ImageAvailable {
                        image,
                        url: image_url,
                        is_placeholder,
                    });
                },
                Err(()) => {
                    debug!("{} is not available", url);
                    return ImageCacheResult::LoadError;
                },
            }
        }

        let decoded = {
            let result = store
                .pending_loads
                .get_cached(url.clone(), origin.clone(), cors_setting);
            match result {
                CacheResult::Hit(key, pl) => match (&pl.result, &pl.metadata) {
                    (&Some(Ok(_)), _) => {
                        debug!("Sync decoding {} ({:?})", url, key);
                        decode_bytes_sync(key, pl.bytes.as_slice(), pl.cors_status)
                    },
                    (&None, Some(meta)) => {
                        debug!("Metadata available for {} ({:?})", url, key);
                        return ImageCacheResult::Available(
                            ImageOrMetadataAvailable::MetadataAvailable(meta.clone()),
                        );
                    },
                    (&Some(Err(_)), _) | (&None, &None) => {
                        debug!("{} ({:?}) is still pending", url, key);
                        return ImageCacheResult::Pending(key);
                    },
                },
                CacheResult::Miss(Some((key, _pl))) => {
                    debug!("Should be requesting {} ({:?})", url, key);
                    return ImageCacheResult::ReadyForRequest(key);
                },
                CacheResult::Miss(None) => {
                    debug!("Couldn't find an entry for {}", url);
                    return ImageCacheResult::LoadError;
                },
            }
        };

        // In the case where a decode is ongoing (or waiting in a queue) but we
        // have the full response available, we decode the bytes synchronously
        // and ignore the async decode when it finishes later.
        // TODO: make this behaviour configurable according to the caller's needs.
        store.handle_decoder(decoded);
        match store.get_completed_image_if_available(url, origin, cors_setting, use_placeholder) {
            Some(Ok((image, image_url))) => {
                let is_placeholder = image_url == store.placeholder_url;
                ImageCacheResult::Available(ImageOrMetadataAvailable::ImageAvailable {
                    image,
                    url: image_url,
                    is_placeholder,
                })
            },
            _ => ImageCacheResult::LoadError,
        }
    }

    fn track_image(
        &self,
        url: ServoUrl,
        origin: ImmutableOrigin,
        cors_setting: Option<CorsSettings>,
        sender: IpcSender<PendingImageResponse>,
        use_placeholder: UsePlaceholder,
    ) -> ImageCacheResult {
        debug!("Track image for {} ({:?})", url, origin);
        let cache_result = self.get_cached_image_status(
            url.clone(),
            origin.clone(),
            cors_setting,
            use_placeholder,
        );

        match cache_result {
            ImageCacheResult::Available(ImageOrMetadataAvailable::MetadataAvailable(_)) => {
                let mut store = self.store.lock().unwrap();
                let id = *store
                    .pending_loads
                    .url_to_load_key
                    .get(&(url, origin, cors_setting))
                    .unwrap();
                self.add_listener_with_store(&mut store, id, ImageResponder::new(sender, id));
            },
            ImageCacheResult::Pending(id) | ImageCacheResult::ReadyForRequest(id) => {
                self.add_listener(id, ImageResponder::new(sender, id));
            },
            _ => {},
        }

        cache_result
    }

    /// Add a new listener for the given pending image id. If the image is already present,
    /// the responder will still receive the expected response.
    fn add_listener(&self, id: PendingImageId, listener: ImageResponder) {
        let mut store = self.store.lock().unwrap();
        self.add_listener_with_store(&mut store, id, listener);
    }

    /// Inform the image cache about a response for a pending request.
    fn notify_pending_response(&self, id: PendingImageId, action: FetchResponseMsg) {
        match (action, id) {
            (FetchResponseMsg::ProcessRequestBody, _) |
            (FetchResponseMsg::ProcessRequestEOF, _) => (),
            (FetchResponseMsg::ProcessResponse(response), _) => {
                debug!("Received {:?} for {:?}", response.as_ref().map(|_| ()), id);
                let mut store = self.store.lock().unwrap();
                let pending_load = store.pending_loads.get_by_key_mut(&id).unwrap();
                let (cors_status, metadata) = match response {
                    Ok(meta) => match meta {
                        FetchMetadata::Unfiltered(m) => (CorsStatus::Safe, Some(m)),
                        FetchMetadata::Filtered { unsafe_, filtered } => (
                            match filtered {
                                FilteredMetadata::Basic(_) | FilteredMetadata::Cors(_) => {
                                    CorsStatus::Safe
                                },
                                FilteredMetadata::Opaque | FilteredMetadata::OpaqueRedirect(_) => {
                                    CorsStatus::Unsafe
                                },
                            },
                            Some(unsafe_),
                        ),
                    },
                    Err(_) => (CorsStatus::Unsafe, None),
                };
                let final_url = metadata.as_ref().map(|m| m.final_url.clone());
                pending_load.final_url = final_url;
                pending_load.cors_status = cors_status;
            },
            (FetchResponseMsg::ProcessResponseChunk(data), _) => {
                debug!("Got some data for {:?}", id);
                let mut store = self.store.lock().unwrap();
                let pending_load = store.pending_loads.get_by_key_mut(&id).unwrap();
                pending_load.bytes.extend_from_slice(&data);

                //jmr0 TODO: possibly move to another task?
                let mut reader = std::io::Cursor::new(pending_load.bytes.as_slice());
                if let Ok(info) = imsz_from_reader(&mut reader) {
                    let img_metadata = ImageMetadata {
                        width: info.width as u32,
                        height: info.height as u32,
                    };
                    for listener in &pending_load.listeners {
                        listener.respond(ImageResponse::MetadataLoaded(img_metadata.clone()));
                    }
                    pending_load.metadata = Some(img_metadata);
                }
            },
            (FetchResponseMsg::ProcessResponseEOF(result), key) => {
                debug!("Received EOF for {:?}", key);
                match result {
                    Ok(_) => {
                        let (bytes, cors_status) = {
                            let mut store = self.store.lock().unwrap();
                            let pending_load = store.pending_loads.get_by_key_mut(&id).unwrap();
                            pending_load.result = Some(Ok(()));
                            debug!("Async decoding {} ({:?})", pending_load.url, key);
                            (pending_load.bytes.mark_complete(), pending_load.cors_status)
                        };

                        let local_store = self.store.clone();
                        self.thread_pool.spawn(move || {
                            let msg = decode_bytes_sync(key, &bytes, cors_status);
                            debug!("Image decoded");
                            local_store.lock().unwrap().handle_decoder(msg);
                        });
                    },
                    Err(_) => {
                        debug!("Processing error for {:?}", key);
                        let mut store = self.store.lock().unwrap();
                        let placeholder_image = store.placeholder_image.clone();
                        store.complete_load(id, LoadResult::PlaceholderLoaded(placeholder_image))
                    },
                }
            },
        }
    }
}

impl ImageCacheImpl {
    /// Require self.store.lock() before calling.
    fn add_listener_with_store(
        &self,
        store: &mut ImageCacheStore,
        id: PendingImageId,
        listener: ImageResponder,
    ) {
        if let Some(load) = store.pending_loads.get_by_key_mut(&id) {
            if let Some(ref metadata) = load.metadata {
                listener.respond(ImageResponse::MetadataLoaded(metadata.clone()));
            }
            load.add_listener(listener);
            return;
        }
        if let Some(load) = store.completed_loads.values().find(|l| l.id == id) {
            listener.respond(load.image_response.clone());
            return;
        }
        warn!("Couldn't find cached entry for listener {:?}", id);
    }
}
