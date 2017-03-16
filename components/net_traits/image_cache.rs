/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use FetchResponseMsg;
use NetworkError;
use image::base::{Image, ImageMetadata, PixelFormat, load_from_memory};
use immeta::load_from_buf;
use ipc_channel::ipc::IpcSender;
use servo_config::resource_files::resources_dir_path;
use servo_url::ServoUrl;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::fs::File;
use std::io::{self, Read};
use std::mem;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;
use webrender_traits;

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

fn convert_format(format: PixelFormat) -> webrender_traits::ImageFormat {
    match format {
        PixelFormat::K8 | PixelFormat::KA8 => {
            panic!("Not support by webrender yet");
        }
        PixelFormat::RGB8 => webrender_traits::ImageFormat::RGB8,
        PixelFormat::RGBA8 => webrender_traits::ImageFormat::RGBA8,
    }
}

fn decode_bytes_sync(key: LoadKey, bytes: &[u8]) -> DecoderMsg {
    let image = load_from_memory(bytes);
    DecoderMsg {
        key: key,
        image: image
    }
}

fn get_placeholder_image(webrender_api: &webrender_traits::RenderApi) -> io::Result<Arc<Image>> {
    let mut placeholder_path = try!(resources_dir_path());
    placeholder_path.push("rippy.png");
    let mut file = try!(File::open(&placeholder_path));
    let mut image_data = vec![];
    try!(file.read_to_end(&mut image_data));
    let mut image = load_from_memory(&image_data).unwrap();
    let format = convert_format(image.format);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&*image.bytes);
    let descriptor = webrender_traits::ImageDescriptor {
        width: image.width,
        height: image.height,
        stride: None,
        format: format,
        offset: 0,
        is_opaque: is_image_opaque(format, &bytes),
    };
    let data = webrender_traits::ImageData::new(bytes);
    let image_key = webrender_api.generate_image_key();
    webrender_api.add_image(image_key, descriptor, data, None);
    image.id = Some(image_key);
    Ok(Arc::new(image))
}


// TODO(gw): This is a port of the old is_image_opaque code from WR.
//           Consider using SIMD to speed this up if it shows in profiles.
fn is_image_opaque(format: webrender_traits::ImageFormat, bytes: &[u8]) -> bool {
    match format {
        webrender_traits::ImageFormat::RGBA8 => {
            let mut is_opaque = true;
            for i in 0..(bytes.len() / 4) {
                if bytes[i * 4 + 3] != 255 {
                    is_opaque = false;
                    break;
                }
            }
            is_opaque
        }
        webrender_traits::ImageFormat::RGB8 => true,
        webrender_traits::ImageFormat::A8 => false,
        webrender_traits::ImageFormat::Invalid | webrender_traits::ImageFormat::RGBAF32 => unreachable!(),
    }
}

fn premultiply(data: &mut [u8]) {
    let length = data.len();

    for i in (0..length).step_by(4) {
        let b = data[i + 0] as u32;
        let g = data[i + 1] as u32;
        let r = data[i + 2] as u32;
        let a = data[i + 3] as u32;

        data[i + 0] = (b * a / 255) as u8;
        data[i + 1] = (g * a / 255) as u8;
        data[i + 2] = (r * a / 255) as u8;
    }
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

/// Whether a consumer is in a position to request images or not. This can occur
/// when animations are being processed by the layout thread while the script
/// thread is executing in parallel.
#[derive(Copy, Clone, PartialEq, Deserialize, Serialize)]
pub enum CanRequestImages {
    No,
    Yes,
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

/// Indicating either entire image or just metadata availability
#[derive(Clone, Deserialize, Serialize, HeapSizeOf)]
pub enum ImageOrMetadataAvailable {
    ImageAvailable(Arc<Image>),
    MetadataAvailable(ImageMetadata),
}

/// This is optionally passed to the image cache when requesting
/// and image, and returned to the specified event loop when the
/// image load completes. It is typically used to trigger a reflow
/// and/or repaint.
#[derive(Clone, Deserialize, Serialize)]
pub struct ImageResponder {
    id: PendingImageId,
    sender: IpcSender<PendingImageResponse>,
}

impl ImageResponder {
    pub fn new(sender: IpcSender<PendingImageResponse>, id: PendingImageId) -> ImageResponder {
        ImageResponder {
            sender: sender,
            id: id,
        }
    }

    pub fn respond(&self, response: ImageResponse) {
        debug!("Notifying listener");
        // This send can fail if thread waiting for this notification has panicked.
        // That's not a case that's worth warning about.
        // TODO(#15501): are there cases in which we should perform cleanup?
        let _ = self.sender.send(PendingImageResponse {
            response: response,
            id: self.id,
        });
    }
}

/// The returned image.
#[derive(Clone, Deserialize, Serialize, HeapSizeOf)]
pub enum ImageResponse {
    /// The requested image was loaded.
    Loaded(Arc<Image>),
    /// The request image metadata was loaded.
    MetadataLoaded(ImageMetadata),
    /// The requested image failed to load, so a placeholder was loaded instead.
    PlaceholderLoaded(Arc<Image>),
    /// Neither the requested image nor the placeholder could be loaded.
    None,
}

/// The current state of an image in the cache.
#[derive(PartialEq, Copy, Clone, Deserialize, Serialize)]
pub enum ImageState {
    Pending(PendingImageId),
    LoadError,
    NotRequested(PendingImageId),
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

/// The unique id for an image that has previously been requested.
#[derive(Copy, Clone, PartialEq, Eq, Deserialize, Serialize, HeapSizeOf, Hash, Debug)]
pub struct PendingImageId(pub u64);

#[derive(Deserialize, Serialize)]
pub struct PendingImageResponse {
    pub response: ImageResponse,
    pub id: PendingImageId,
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
}

impl PendingLoad {
    fn new(url: ServoUrl) -> PendingLoad {
        PendingLoad {
            bytes: ImageBytes::InProgress(vec!()),
            metadata: None,
            result: None,
            listeners: vec!(),
            url: url,
        }
    }

    fn add_listener(&mut self, listener: ImageResponder) {
        self.listeners.push(listener);
    }
}

#[derive(Copy, Clone, PartialEq, Hash, Eq, Deserialize, Serialize)]
pub enum UsePlaceholder {
    No,
    Yes,
}

// ======================================================================
// ImageCache implementation.
// ======================================================================

pub struct ImageCache {
    // Images that are loading over network, or decoding.
    pending_loads: Mutex<AllPendingLoads>,

    // Images that have finished loading (successful or not)
    completed_loads: Mutex<HashMap<ServoUrl, CompletedLoad>>,

    // The placeholder image used when an image fails to load
    placeholder_image: Option<Arc<Image>>,

    // Webrender API instance.
    webrender_api: Mutex<webrender_traits::RenderApi>,
}

impl ImageCache {
    // Change state of a url from pending -> loaded.
    fn complete_load(&self, key: LoadKey, mut load_result: LoadResult) {
        let mut pending_loads = self.pending_loads.lock().unwrap();
        let pending_load = match pending_loads.remove(&key) {
            Some(load) => load,
            None => return,
        };

        match load_result {
            LoadResult::Loaded(ref mut image) => {
                let format = convert_format(image.format);
                let mut bytes = Vec::new();
                bytes.extend_from_slice(&*image.bytes);
                if format == webrender_traits::ImageFormat::RGBA8 {
                    premultiply(bytes.as_mut_slice());
                }
                let descriptor = webrender_traits::ImageDescriptor {
                    width: image.width,
                    height: image.height,
                    stride: None,
                    format: format,
                    offset: 0,
                    is_opaque: is_image_opaque(format, &bytes),
                };
                let data = webrender_traits::ImageData::new(bytes);
                let webrender_api = self.webrender_api.lock().unwrap();
                let image_key = webrender_api.generate_image_key();
                webrender_api.add_image(image_key, descriptor, data, None);
                image.id = Some(image_key);
            }
            LoadResult::PlaceholderLoaded(..) | LoadResult::None => {}
        }

        let image_response = match load_result {
            LoadResult::Loaded(image) => ImageResponse::Loaded(Arc::new(image)),
            LoadResult::PlaceholderLoaded(image) => ImageResponse::PlaceholderLoaded(image),
            LoadResult::None => ImageResponse::None,
        };

        let completed_load = CompletedLoad::new(image_response.clone(), key);
        self.completed_loads.lock().unwrap().insert(pending_load.url.into(), completed_load);

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
        self.completed_loads.lock().unwrap().get(url).map(|completed_load| {
            match (&completed_load.image_response, placeholder) {
                (&ImageResponse::Loaded(ref image), _) |
                (&ImageResponse::PlaceholderLoaded(ref image), UsePlaceholder::Yes) => {
                    Ok(ImageOrMetadataAvailable::ImageAvailable(image.clone()))
                }
                (&ImageResponse::PlaceholderLoaded(_), UsePlaceholder::No) |
                (&ImageResponse::None, _) |
                (&ImageResponse::MetadataLoaded(_), _) => {
                    Err(ImageState::LoadError)
                }
            }
        })
    }

    /// Handle a message from one of the decoder worker threads or from a sync
    /// decoding operation.
    fn handle_decoder(&self, msg: DecoderMsg) {
        let image = match msg.image {
            None => LoadResult::None,
            Some(image) => LoadResult::Loaded(image),
        };
        self.complete_load(msg.key, image);
    }

    // Public API

    pub fn new(webrender_api: webrender_traits::RenderApi) -> Arc<ImageCache> {
        debug!("New image cache");
        Arc::new(ImageCache {
            pending_loads: Mutex::new(AllPendingLoads::new()),
            completed_loads: Mutex::new(HashMap::new()),
            placeholder_image: get_placeholder_image(&webrender_api).ok(),
            webrender_api: Mutex::new(webrender_api),
        })
    }

    /// Return any available metadata or image for the given URL,
    /// or an indication that the image is not yet available if it is in progress,
    /// or else reserve a slot in the cache for the URL if the consumer can request images.
    pub fn find_image_or_metadata(&self,
                                  url: ServoUrl,
                                  use_placeholder: UsePlaceholder,
                                  can_request: CanRequestImages)
                                  -> Result<ImageOrMetadataAvailable, ImageState> {
        debug!("Find image or metadata for {}", url);
        if let Some(result) = self.get_completed_image_if_available(&url, use_placeholder) {
            debug!("{} is available", url);
            return result;
        }

        let decoded = {
            let mut pending_loads = self.pending_loads.lock().unwrap();
            let result = pending_loads.get_cached(url.clone(), can_request);
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
        self.handle_decoder(decoded);
        match self.get_completed_image_if_available(&url, use_placeholder) {
            Some(result) => result,
            None => Err(ImageState::LoadError),
        }
    }

    /// Add a new listener for the given pending image id. If the image is already present,
    /// the responder will still receive the expected response.
    pub fn add_listener(&self, id: PendingImageId, listener: ImageResponder) {
        if let Some(load) = self.pending_loads.lock().unwrap().get_by_key_mut(&id) {
            if let Some(ref metadata) = load.metadata {
                listener.respond(ImageResponse::MetadataLoaded(metadata.clone()));
            }
            load.add_listener(listener);
            return;
        }
        if let Some(load) = self.completed_loads.lock().unwrap().values().find(|l| l.id == id) {
            listener.respond(load.image_response.clone());
            return;
        }
        warn!("Couldn't find cached entry for listener {:?}", id);
    }

    /// Inform the image cache about a response for a pending request.
    pub fn notify_pending_response(&self, id: PendingImageId, action: FetchResponseMsg) {
        match (action, id) {
            (FetchResponseMsg::ProcessRequestBody, _) |
            (FetchResponseMsg::ProcessRequestEOF, _) => return,
            (FetchResponseMsg::ProcessResponse(_response), _) => {}
            (FetchResponseMsg::ProcessResponseChunk(data), _) => {
                debug!("Got some data for {:?}", id);
                let mut pending_loads = self.pending_loads.lock().unwrap();
                let pending_load = pending_loads.get_by_key_mut(&id).unwrap();
                pending_load.bytes.extend_from_slice(&data);
                //jmr0 TODO: possibly move to another task?
                if let None = pending_load.metadata {
                    if let Ok(metadata) = load_from_buf(&pending_load.bytes.as_slice()) {
                        let dimensions = metadata.dimensions();
                        let img_metadata = ImageMetadata { width: dimensions.width,
                                                           height: dimensions.height };
                        for listener in &pending_load.listeners {
                            listener.respond(ImageResponse::MetadataLoaded(img_metadata.clone()));
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
                            let mut pending_loads = self.pending_loads.lock().unwrap();
                            let pending_load = pending_loads.get_by_key_mut(&id).unwrap();
                            pending_load.result = Some(result);
                            debug!("Async decoding {} ({:?})", pending_load.url, key);
                            pending_load.bytes.mark_complete()
                        };

                        let (tx, rx) = channel();
                        thread::Builder::new().name(
                            "Image decoding async operation".to_owned()).spawn(move || {
                            let msg = decode_bytes_sync(key, &*bytes);
                            tx.send(msg).unwrap();
                        }).expect("Thread spawning failed");

                        if let Some(msg) = rx.recv().ok() {
                            debug!("Image decoded {:?}", key);
                            self.handle_decoder(msg);
                        };
                    }
                    Err(_) => {
                        debug!("Processing error for {:?}", key);
                        match self.placeholder_image.clone() {
                            Some(placeholder_image) => {
                                self.complete_load(id, LoadResult::PlaceholderLoaded(
                                        placeholder_image))
                            }
                            None => self.complete_load(id, LoadResult::None),
                        }
                    }
                }
            }
        }
    }
}
