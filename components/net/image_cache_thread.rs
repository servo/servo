/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use immeta::load_from_buf;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use net_traits::{NetworkError, FetchResponseMsg};
use net_traits::image::base::{Image, ImageMetadata, PixelFormat, load_from_memory};
use net_traits::image_cache_thread::{ImageCacheCommand, ImageCacheThread, ImageState};
use net_traits::image_cache_thread::{ImageOrMetadataAvailable, ImageResponse, UsePlaceholder};
use net_traits::image_cache_thread::{ImageResponder, PendingImageId, CanRequestImages};
use servo_config::resource_files::resources_dir_path;
use servo_url::ServoUrl;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::fs::File;
use std::io::{self, Read};
use std::mem;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use threadpool::ThreadPool;
use webrender_traits;

///
/// TODO(gw): Remaining work on image cache:
///     * Make use of the prefetch support in various parts of the code.
///     * Profile time in GetImageIfAvailable - might be worth caching these results per paint / layout thread.
///
/// MAYBE(Yoric):
///     * For faster lookups, it might be useful to store the LoadKey in the DOM once we have performed a first load.

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

enum LoadResult {
    Loaded(Image),
    PlaceholderLoaded(Arc<Image>),
    None
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

/// Result of accessing a cache.
enum CacheResult<'a> {
    /// The value was in the cache.
    Hit(LoadKey, &'a mut PendingLoad),
    /// The value was not in the cache and needed to be regenerated.
    Miss(Option<(LoadKey, &'a mut PendingLoad)>),
}

impl AllPendingLoads {
    fn new() -> AllPendingLoads {
        AllPendingLoads {
            loads: HashMap::new(),
            url_to_load_key: HashMap::new(),
            keygen: LoadKeyGenerator::new(),
        }
    }

    // `true` if there is no currently pending load, `false` otherwise.
    fn is_empty(&self) -> bool {
        assert!(self.loads.is_empty() == self.url_to_load_key.is_empty());
        self.loads.is_empty()
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

struct ResourceLoadInfo {
    action: FetchResponseMsg,
    key: LoadKey,
}

/// Implementation of the image cache
struct ImageCache {
    decoder_sender: Sender<DecoderMsg>,

    // Worker threads for decoding images.
    thread_pool: ThreadPool,

    // Images that are loading over network, or decoding.
    pending_loads: AllPendingLoads,

    // Images that have finished loading (successful or not)
    completed_loads: HashMap<ServoUrl, CompletedLoad>,

    // The placeholder image used when an image fails to load
    placeholder_image: Option<Arc<Image>>,

    // Webrender API instance.
    webrender_api: webrender_traits::RenderApi,
}

/// Message that the decoder worker threads send to main image cache thread.
struct DecoderMsg {
    key: LoadKey,
    image: Option<Image>,
}

struct Receivers {
    cmd_receiver: Receiver<ImageCacheCommand>,
    decoder_receiver: Receiver<DecoderMsg>,
}

impl Receivers {
    #[allow(unsafe_code)]
    fn recv(&self) -> SelectResult {
        let cmd_receiver = &self.cmd_receiver;
        let decoder_receiver = &self.decoder_receiver;
        select! {
            msg = cmd_receiver.recv() => {
                SelectResult::Command(msg.unwrap())
            },
            msg = decoder_receiver.recv() => {
                SelectResult::Decoder(msg.unwrap())
            }
        }
    }
}

/// The types of messages that the main image cache thread receives.
enum SelectResult {
    Command(ImageCacheCommand),
    Decoder(DecoderMsg),
}

fn convert_format(format: PixelFormat) -> webrender_traits::ImageFormat {
    match format {
        PixelFormat::K8 | PixelFormat::KA8 => {
            panic!("Not support by webrender yet");
        }
        PixelFormat::RGB8 => webrender_traits::ImageFormat::RGB8,
        PixelFormat::RGBA8 => webrender_traits::ImageFormat::RGBA8,
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
        is_opaque: is_image_opaque(format, &bytes),
    };
    let data = webrender_traits::ImageData::new(bytes);
    image.id = Some(webrender_api.add_image(descriptor, data));
    Ok(Arc::new(image))
}

impl ImageCache {
    fn run(webrender_api: webrender_traits::RenderApi,
           ipc_command_receiver: IpcReceiver<ImageCacheCommand>) {
        // Preload the placeholder image, used when images fail to load.
        let placeholder_image = get_placeholder_image(&webrender_api).ok();

        // Ask the router to proxy messages received over IPC to us.
        let cmd_receiver = ROUTER.route_ipc_receiver_to_new_mpsc_receiver(ipc_command_receiver);

        let (decoder_sender, decoder_receiver) = channel();
        let mut cache = ImageCache {
            decoder_sender: decoder_sender,
            thread_pool: ThreadPool::new(4),
            pending_loads: AllPendingLoads::new(),
            completed_loads: HashMap::new(),
            placeholder_image: placeholder_image,
            webrender_api: webrender_api,
        };

        let receivers = Receivers {
            cmd_receiver: cmd_receiver,
            decoder_receiver: decoder_receiver,
        };

        let mut exit_sender: Option<IpcSender<()>> = None;

        loop {
            match receivers.recv() {
                SelectResult::Command(cmd) => {
                    exit_sender = cache.handle_cmd(cmd);
                }
                SelectResult::Decoder(msg) => {
                    cache.handle_decoder(msg);
                }
            }

            // Can only exit when all pending loads are complete.
            if let Some(ref exit_sender) = exit_sender {
                if cache.pending_loads.is_empty() {
                    exit_sender.send(()).unwrap();
                    break;
                }
            }
        }
    }

    // Handle a request from a client
    fn handle_cmd(&mut self, cmd: ImageCacheCommand) -> Option<IpcSender<()>> {
        match cmd {
            ImageCacheCommand::Exit(sender) => {
                return Some(sender);
            }
            ImageCacheCommand::AddListener(id, responder) => {
                self.add_listener(id, responder);
            }
            ImageCacheCommand::GetImageOrMetadataIfAvailable(url,
                                                             use_placeholder,
                                                             can_request,
                                                             consumer) => {
                let result = self.get_image_or_meta_if_available(url, use_placeholder, can_request);
                // TODO(#15501): look for opportunities to clean up cache if this send fails.
                let _ = consumer.send(result);
            }
            ImageCacheCommand::StoreDecodeImage(id, data) => {
                self.handle_progress(ResourceLoadInfo {
                    action: data,
                    key: id
                });
            }
        };

        None
    }

    // Handle progress messages from the resource thread
    fn handle_progress(&mut self, msg: ResourceLoadInfo) {
        match (msg.action, msg.key) {
            (FetchResponseMsg::ProcessRequestBody, _) |
            (FetchResponseMsg::ProcessRequestEOF, _) => return,
            (FetchResponseMsg::ProcessResponse(_response), _) => {}
            (FetchResponseMsg::ProcessResponseChunk(data), _) => {
                debug!("got some data for {:?}", msg.key);
                let pending_load = self.pending_loads.get_by_key_mut(&msg.key).unwrap();
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
                debug!("received EOF for {:?}", key);
                match result {
                    Ok(()) => {
                        let pending_load = self.pending_loads.get_by_key_mut(&msg.key).unwrap();
                        pending_load.result = Some(result);
                        let bytes = pending_load.bytes.mark_complete();
                        let sender = self.decoder_sender.clone();
                        debug!("async decoding {} ({:?})", pending_load.url, key);

                        self.thread_pool.execute(move || {
                            let msg = decode_bytes_sync(key, &*bytes);
                            sender.send(msg).unwrap();
                        });
                    }
                    Err(_) => {
                        debug!("processing error for {:?}", key);
                        match self.placeholder_image.clone() {
                            Some(placeholder_image) => {
                                self.complete_load(msg.key, LoadResult::PlaceholderLoaded(
                                        placeholder_image))
                            }
                            None => self.complete_load(msg.key, LoadResult::None),
                        }
                    }
                }
            }
        }
    }

    // Handle a message from one of the decoder worker threads
    fn handle_decoder(&mut self, msg: DecoderMsg) {
        let image = match msg.image {
            None => LoadResult::None,
            Some(image) => LoadResult::Loaded(image),
        };
        self.complete_load(msg.key, image);
    }

    // Change state of a url from pending -> loaded.
    fn complete_load(&mut self, key: LoadKey, mut load_result: LoadResult) {
        let pending_load = match self.pending_loads.remove(&key) {
            Some(load) => load,
            None => return,
        };

        match load_result {
            LoadResult::Loaded(ref mut image) => {
                let format = convert_format(image.format);
                let mut bytes = Vec::new();
                bytes.extend_from_slice(&*image.bytes);
                let descriptor = webrender_traits::ImageDescriptor {
                    width: image.width,
                    height: image.height,
                    stride: None,
                    format: format,
                    is_opaque: is_image_opaque(format, &bytes),
                };
                let data = webrender_traits::ImageData::new(bytes);
                image.id = Some(self.webrender_api.add_image(descriptor, data));
            }
            LoadResult::PlaceholderLoaded(..) | LoadResult::None => {}
        }

        let image_response = match load_result {
            LoadResult::Loaded(image) => ImageResponse::Loaded(Arc::new(image)),
            LoadResult::PlaceholderLoaded(image) => ImageResponse::PlaceholderLoaded(image),
            LoadResult::None => ImageResponse::None,
        };

        let completed_load = CompletedLoad::new(image_response.clone(), key);
        self.completed_loads.insert(pending_load.url.into(), completed_load);

        for listener in pending_load.listeners {
            listener.respond(image_response.clone());
        }
    }

    /// Add a listener for a given image if it is still pending, or notify the
    /// listener if the image is complete.
    fn add_listener(&mut self,
                    id: PendingImageId,
                    listener: ImageResponder) {
        if let Some(load) = self.pending_loads.get_by_key_mut(&id) {
            if let Some(ref metadata) = load.metadata {
                listener.respond(ImageResponse::MetadataLoaded(metadata.clone()));
            }
            load.add_listener(listener);
            return;
        }
        if let Some(load) = self.completed_loads.values().find(|l| l.id == id) {
            listener.respond(load.image_response.clone());
            return;
        }
        warn!("Couldn't find cached entry for listener {:?}", id);
    }

    /// Return a completed image if it exists, or None if there is no complete load
    /// or the complete load is not fully decoded or is unavailable.
    fn get_completed_image_if_available(&self,
                                        url: &ServoUrl,
                                        placeholder: UsePlaceholder)
                                        -> Option<Result<ImageOrMetadataAvailable, ImageState>> {
        self.completed_loads.get(url).map(|completed_load| {
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

    /// Return any available metadata or image for the given URL, or an indication that
    /// the image is not yet available if it is in progress, or else reserve a slot in
    /// the cache for the URL if the consumer can request images.
    fn get_image_or_meta_if_available(&mut self,
                                      url: ServoUrl,
                                      placeholder: UsePlaceholder,
                                      can_request: CanRequestImages)
                                      -> Result<ImageOrMetadataAvailable, ImageState> {
        if let Some(result) = self.get_completed_image_if_available(&url, placeholder) {
            debug!("{} is available", url);
            return result;
        }

        let decoded = {
            let result = self.pending_loads.get_cached(url.clone(), can_request);
            match result {
                CacheResult::Hit(key, pl) => match (&pl.result, &pl.metadata) {
                    (&Some(Ok(_)), _) => {
                        debug!("sync decoding {} ({:?})", url, key);
                        decode_bytes_sync(key, &pl.bytes.as_slice())
                    }
                    (&None, &Some(ref meta)) => {
                        debug!("metadata available for {} ({:?})", url, key);
                        return Ok(ImageOrMetadataAvailable::MetadataAvailable(meta.clone()))
                    }
                    (&Some(Err(_)), _) | (&None, &None) => {
                        debug!("{} ({:?}) is still pending", url, key);
                        return Err(ImageState::Pending(key));
                    }
                },
                CacheResult::Miss(Some((key, _pl))) => {
                    debug!("should be requesting {} ({:?})", url, key);
                    return Err(ImageState::NotRequested(key));
                }
                CacheResult::Miss(None) => {
                    debug!("couldn't find an entry for {}", url);
                    return Err(ImageState::LoadError);
                }
            }
        };

        // In the case where a decode is ongoing (or waiting in a queue) but we have the
        // full response available, we decode the bytes synchronously and ignore the
        // async decode when it finishes later.
        // TODO: make this behaviour configurable according to the caller's needs.
        self.handle_decoder(decoded);
        match self.get_completed_image_if_available(&url, placeholder) {
            Some(result) => result,
            None => Err(ImageState::LoadError),
        }
    }
}

/// Create a new image cache.
pub fn new_image_cache_thread(webrender_api: webrender_traits::RenderApi) -> ImageCacheThread {
    let (ipc_command_sender, ipc_command_receiver) = ipc::channel().unwrap();

    thread::Builder::new().name("ImageCacheThread".to_owned()).spawn(move || {
        ImageCache::run(webrender_api, ipc_command_receiver)
    }).expect("Thread spawning failed");

    ImageCacheThread::new(ipc_command_sender)
}

fn decode_bytes_sync(key: LoadKey, bytes: &[u8]) -> DecoderMsg {
    let image = load_from_memory(bytes);
    DecoderMsg {
        key: key,
        image: image
    }
}
