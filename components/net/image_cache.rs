/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use immeta::load_from_buf;
use net_traits::{FetchMetadata, FetchResponseMsg, NetworkError};
use net_traits::image::base::{Image, ImageMetadata, PixelFormat, load_from_memory};
use net_traits::image_cache::{CanRequestImages, ImageCache, ImageResponder};
use net_traits::image_cache::{ImageOrMetadataAvailable, ImageResponse, ImageState};
use net_traits::image_cache::{PendingImageId, UsePlaceholder};
use servo_config::resource_files::resources_dir_path;
use servo_url::ServoUrl;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::fs::File;
use std::io::{self, Read};
use std::mem;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use webrender_api;

///
/// TODO(gw): Remaining work on image cache:
///     * Make use of the prefetch support in various parts of the code.
///     * Profile time in GetImageIfAvailable - might be worth caching these
///       results per paint / layout thread.
///
/// MAYBE(Yoric):
///     * For faster lookups, it might be useful to store the LoadKey in the
///       DOM once we have performed a first load.

// ======================================================================
// Helper functions.
// ======================================================================

fn decode_bytes_sync(key: LoadKey, bytes: &[u8]) -> DecoderMsg {
    let image = load_from_memory(bytes);
    DecoderMsg {
        key: key,
        image: image
    }
}

fn get_placeholder_image(webrender_api: &webrender_api::RenderApi, path: &PathBuf) -> io::Result<Arc<Image>> {
    let mut file = File::open(path)?;
    let mut image_data = vec![];
    file.read_to_end(&mut image_data)?;
    let mut image = load_from_memory(&image_data).unwrap();
    set_webrender_image_key(webrender_api, &mut image);
    Ok(Arc::new(image))
}

fn set_webrender_image_key(webrender_api: &webrender_api::RenderApi, image: &mut Image) {
    if image.id.is_some() { return; }
    let mut bytes = Vec::new();
    let is_opaque = match image.format {
        PixelFormat::BGRA8 => {
            bytes.extend_from_slice(&*image.bytes);
            premultiply(bytes.as_mut_slice())
        }
        PixelFormat::RGB8 => {
            for bgr in image.bytes.chunks(3) {
                bytes.extend_from_slice(&[
                    bgr[2],
                    bgr[1],
                    bgr[0],
                    0xff
                ]);
            }

            true
        }
        PixelFormat::K8 | PixelFormat::KA8 => {
            panic!("Not support by webrender yet");
        }
    };
    let descriptor = webrender_api::ImageDescriptor {
        width: image.width,
        height: image.height,
        stride: None,
        format: webrender_api::ImageFormat::BGRA8,
        offset: 0,
        is_opaque,
    };
    let data = webrender_api::ImageData::new(bytes);
    let image_key = webrender_api.generate_image_key();
    let mut updates = webrender_api::ResourceUpdates::new();
    updates.add_image(image_key, descriptor, data, None);
    webrender_api.update_resources(updates);
    image.id = Some(image_key);
}

// Returns true if the image was found to be
// completely opaque.
fn premultiply(data: &mut [u8]) -> bool {
    let mut is_opaque = true;
    let length = data.len();

    let mut i = 0;
    while i < length {
        let b = data[i + 0] as u32;
        let g = data[i + 1] as u32;
        let r = data[i + 2] as u32;
        let a = data[i + 3] as u32;

        data[i + 0] = (b * a / 255) as u8;
        data[i + 1] = (g * a / 255) as u8;
        data[i + 2] = (r * a / 255) as u8;

        i += 4;
        is_opaque = is_opaque && a == 255;
    }

    is_opaque
}

// ======================================================================
// Aux structs and enums.
// ======================================================================

// Represents all the currently pending loads/decodings. For
// performance reasons, loads are indexed by a dedicated load key.
struct AllPendingLoads {
    // The loads, indexed by a load key. Used during most operations,
    // for performance reasons.
    loads: HashMap<LoadKey, PendingLoad>,

    // Get a load key from its url. Used ony when starting and
    // finishing a load or when adding a new listener.
    url_to_load_key: HashMap<ServoUrl, LoadKey>,

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
        self.loads.remove(key).
            and_then(|pending_load| {
                self.url_to_load_key.remove(&pending_load.url).unwrap();
                Some(pending_load)
            })
    }

    fn get_cached<'a>(&'a mut self, url: ServoUrl, can_request: CanRequestImages)
                      -> CacheResult<'a> {
        match self.url_to_load_key.entry(url.clone()) {
            Occupied(url_entry) => {
                let load_key = url_entry.get();
                CacheResult::Hit(*load_key, self.loads.get_mut(load_key).unwrap())
            }
            Vacant(url_entry) => {
                if can_request == CanRequestImages::No {
                    return CacheResult::Miss(None);
                }

                let load_key = self.keygen.next();
                url_entry.insert(load_key);

                let pending_load = PendingLoad::new(url);
                match self.loads.entry(load_key) {
                    Occupied(_) => unreachable!(),
                    Vacant(load_entry) => {
                        let mut_load = load_entry.insert(pending_load);
                        CacheResult::Miss(Some((load_key, mut_load)))
                    }
                }
            }
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
        CompletedLoad {
            image_response: image_response,
            id: id,
        }
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
            mem::replace(own_bytes, vec![])
        };
        let bytes = Arc::new(bytes);
        *self = ImageBytes::Complete(bytes.clone());
        bytes
    }

    fn as_slice(&self) -> &[u8] {
        match *self {
            ImageBytes::InProgress(ref bytes) => &bytes,
            ImageBytes::Complete(ref bytes) => &*bytes,
        }
    }
}

// A key used to communicate during loading.
type LoadKey = PendingImageId;

struct LoadKeyGenerator {
    counter: u64
}

impl LoadKeyGenerator {
    fn new() -> LoadKeyGenerator {
        LoadKeyGenerator {
            counter: 0
        }
    }
    fn next(&mut self) -> PendingImageId {
        self.counter += 1;
        PendingImageId(self.counter)
    }
}

enum LoadResult {
    Loaded(Image),
    PlaceholderLoaded(Arc<Image>),
    None
}

/// Represents an image that is either being loaded
/// by the resource thread, or decoded by a worker thread.
struct PendingLoad {
    // The bytes loaded so far. Reset to an empty vector once loading
    // is complete and the buffer has been transmitted to the decoder.
    bytes: ImageBytes,

    // Image metadata, if available.
    metadata: Option<ImageMetadata>,

    // Once loading is complete, the result of the operation.
    result: Option<Result<(), NetworkError>>,
    listeners: Vec<ImageResponder>,

    // The url being loaded. Do not forget that this may be several Mb
    // if we are loading a data: url.
    url: ServoUrl,

    final_url: Option<ServoUrl>,
}

impl PendingLoad {
    fn new(url: ServoUrl) -> PendingLoad {
        PendingLoad {
            bytes: ImageBytes::InProgress(vec!()),
            metadata: None,
            result: None,
            listeners: vec!(),
            url: url,
            final_url: None,
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
    completed_loads: HashMap<ServoUrl, CompletedLoad>,

    // The placeholder image used when an image fails to load
    placeholder_image: Option<Arc<Image>>,

    // The URL used for the placeholder image
    placeholder_url: ServoUrl,

    // Webrender API instance.
    webrender_api: webrender_api::RenderApi,
}

impl ImageCacheStore {
    // Change state of a url from pending -> loaded.
    fn complete_load(&mut self, key: LoadKey, mut load_result: LoadResult) {
        let pending_load = match self.pending_loads.remove(&key) {
            Some(load) => load,
            None => return,
        };

        match load_result {
            LoadResult::Loaded(ref mut image) => set_webrender_image_key(&self.webrender_api, image),
            LoadResult::PlaceholderLoaded(..) | LoadResult::None => {}
        }

        let url = pending_load.final_url.clone();
        let image_response = match load_result {
            LoadResult::Loaded(image) => ImageResponse::Loaded(Arc::new(image), url.unwrap()),
            LoadResult::PlaceholderLoaded(image) =>
                ImageResponse::PlaceholderLoaded(image, self.placeholder_url.clone()),
            LoadResult::None => ImageResponse::None,
        };

        let completed_load = CompletedLoad::new(image_response.clone(), key);
        self.completed_loads.insert(pending_load.url.into(), completed_load);

        for listener in pending_load.listeners {
            listener.respond(image_response.clone());
        }
    }

    /// Return a completed image if it exists, or None if there is no complete load
    /// or the complete load is not fully decoded or is unavailable.
    fn get_completed_image_if_available(&self,
                                        url: &ServoUrl,
                                        placeholder: UsePlaceholder)
                                        -> Option<Result<ImageOrMetadataAvailable, ImageState>> {
        self.completed_loads.get(url).map(|completed_load| {
            match (&completed_load.image_response, placeholder) {
                (&ImageResponse::Loaded(ref image, ref url), _) |
                (&ImageResponse::PlaceholderLoaded(ref image, ref url), UsePlaceholder::Yes) => {
                    Ok(ImageOrMetadataAvailable::ImageAvailable(image.clone(), url.clone()))
                }
                (&ImageResponse::PlaceholderLoaded(_, _), UsePlaceholder::No) |
                (&ImageResponse::None, _) |
                (&ImageResponse::MetadataLoaded(_), _) => {
                    Err(ImageState::LoadError)
                }
            }
        })
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
}

impl ImageCache for ImageCacheImpl {
    fn new(webrender_api: webrender_api::RenderApi) -> ImageCacheImpl {
        debug!("New image cache");

        let mut placeholder_path = resources_dir_path().expect("Can't figure out resources path.");
        placeholder_path.push("rippy.png");

        ImageCacheImpl {
            store: Arc::new(Mutex::new(ImageCacheStore {
                pending_loads: AllPendingLoads::new(),
                completed_loads: HashMap::new(),
                placeholder_image: get_placeholder_image(&webrender_api, &placeholder_path).ok(),
                placeholder_url: ServoUrl::from_file_path(&placeholder_path).unwrap(),
                webrender_api: webrender_api,
            }))
        }
    }

    /// Return any available metadata or image for the given URL,
    /// or an indication that the image is not yet available if it is in progress,
    /// or else reserve a slot in the cache for the URL if the consumer can request images.
    fn find_image_or_metadata(&self,
                              url: ServoUrl,
                              use_placeholder: UsePlaceholder,
                              can_request: CanRequestImages)
                              -> Result<ImageOrMetadataAvailable, ImageState> {
        debug!("Find image or metadata for {}", url);
        let mut store = self.store.lock().unwrap();
        if let Some(result) = store.get_completed_image_if_available(&url, use_placeholder) {
            debug!("{} is available", url);
            return result;
        }

        let decoded = {
            let result = store.pending_loads.get_cached(url.clone(), can_request);
            match result {
                CacheResult::Hit(key, pl) => match (&pl.result, &pl.metadata) {
                    (&Some(Ok(_)), _) => {
                        debug!("Sync decoding {} ({:?})", url, key);
                        decode_bytes_sync(key, &pl.bytes.as_slice())
                    }
                    (&None, &Some(ref meta)) => {
                        debug!("Metadata available for {} ({:?})", url, key);
                        return Ok(ImageOrMetadataAvailable::MetadataAvailable(meta.clone()))
                    }
                    (&Some(Err(_)), _) | (&None, &None) => {
                        debug!("{} ({:?}) is still pending", url, key);
                        return Err(ImageState::Pending(key));
                    }
                },
                CacheResult::Miss(Some((key, _pl))) => {
                    debug!("Should be requesting {} ({:?})", url, key);
                    return Err(ImageState::NotRequested(key));
                }
                CacheResult::Miss(None) => {
                    debug!("Couldn't find an entry for {}", url);
                    return Err(ImageState::LoadError);
                }
            }
        };

        // In the case where a decode is ongoing (or waiting in a queue) but we
        // have the full response available, we decode the bytes synchronously
        // and ignore the async decode when it finishes later.
        // TODO: make this behaviour configurable according to the caller's needs.
        store.handle_decoder(decoded);
        match store.get_completed_image_if_available(&url, use_placeholder) {
            Some(result) => result,
            None => Err(ImageState::LoadError),
        }
    }

    /// Add a new listener for the given pending image id. If the image is already present,
    /// the responder will still receive the expected response.
    fn add_listener(&self, id: PendingImageId, listener: ImageResponder) {
        let mut store = self.store.lock().unwrap();
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

    /// Inform the image cache about a response for a pending request.
    fn notify_pending_response(&self, id: PendingImageId, action: FetchResponseMsg) {
        match (action, id) {
            (FetchResponseMsg::ProcessRequestBody, _) |
            (FetchResponseMsg::ProcessRequestEOF, _) => return,
            (FetchResponseMsg::ProcessResponse(response), _) => {
                let mut store = self.store.lock().unwrap();
                let pending_load = store.pending_loads.get_by_key_mut(&id).unwrap();
                let metadata = match response {
                    Ok(meta) => {
                        Some(match meta {
                            FetchMetadata::Unfiltered(m) => m,
                            FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
                        })
                    },
                    Err(_) => None,
                };
                let final_url = metadata.as_ref().map(|m| m.final_url.clone());
                pending_load.final_url = final_url;
            }
            (FetchResponseMsg::ProcessResponseChunk(data), _) => {
                debug!("Got some data for {:?}", id);
                let mut store = self.store.lock().unwrap();
                let pending_load = store.pending_loads.get_by_key_mut(&id).unwrap();
                pending_load.bytes.extend_from_slice(&data);
                //jmr0 TODO: possibly move to another task?
                if let None = pending_load.metadata {
                    if let Ok(metadata) = load_from_buf(&pending_load.bytes.as_slice()) {
                        let dimensions = metadata.dimensions();
                        let img_metadata = ImageMetadata { width: dimensions.width,
                                                           height: dimensions.height };
                        for listener in &pending_load.listeners {
                            listener.respond(
                                ImageResponse::MetadataLoaded(img_metadata.clone()));
                        }
                        pending_load.metadata = Some(img_metadata);
                    }
                }
            }
            (FetchResponseMsg::ProcessResponseEOF(result), key) => {
                debug!("Received EOF for {:?}", key);
                match result {
                    Ok(()) => {
                        let bytes = {
                            let mut store = self.store.lock().unwrap();
                            let pending_load = store.pending_loads.get_by_key_mut(&id).unwrap();
                            pending_load.result = Some(result);
                            debug!("Async decoding {} ({:?})", pending_load.url, key);
                            pending_load.bytes.mark_complete()
                        };

                        let local_store = self.store.clone();
                        thread::spawn(move || {
                            let msg = decode_bytes_sync(key, &*bytes);
                            debug!("Image decoded");
                            local_store.lock().unwrap().handle_decoder(msg);
                        });
                    }
                    Err(_) => {
                        debug!("Processing error for {:?}", key);
                        let mut store = self.store.lock().unwrap();
                        match store.placeholder_image.clone() {
                            Some(placeholder_image) => {
                                store.complete_load(
                                    id, LoadResult::PlaceholderLoaded(placeholder_image))
                            }
                            None => store.complete_load(id, LoadResult::None),
                        }
                    }
                }
            }
        }
    }

    /// Ensure an image has a webrender key.
    fn set_webrender_image_key(&self, image: &mut Image) {
        set_webrender_image_key(&self.store.lock().unwrap().webrender_api, image);
    }
}
