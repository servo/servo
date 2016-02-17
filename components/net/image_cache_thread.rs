/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use immeta::load_from_buf;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use net_traits::image::base::{Image, ImageMetadata, load_from_memory, PixelFormat};
use net_traits::image_cache_thread::ImageResponder;
use net_traits::image_cache_thread::{ImageCacheChan, ImageCacheCommand, ImageCacheThread, ImageState};
use net_traits::image_cache_thread::{ImageCacheResult, ImageOrMetadataAvailable, ImageResponse, UsePlaceholder};
use net_traits::{AsyncResponseTarget, ControlMsg, LoadConsumer, LoadData, ResourceThread};
use net_traits::{ResponseAction, LoadContext};
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::fs::File;
use std::io::Read;
use std::mem;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Select, Sender, channel};
use url::Url;
use util::resource_files::resources_dir_path;
use util::thread::spawn_named;
use util::threadpool::ThreadPool;
use webrender_traits;

///
/// TODO(gw): Remaining work on image cache:
///     * Make use of the prefetch support in various parts of the code.
///     * Profile time in GetImageIfAvailable - might be worth caching these results per paint / layout thread.
///
/// MAYBE(Yoric):
///     * For faster lookups, it might be useful to store the LoadKey in the DOM once we have performed a first load.

/// Represents an image that is either being loaded
/// by the resource thread, or decoded by a worker thread.
struct PendingLoad {
    // The bytes loaded so far. Reset to an empty vector once loading
    // is complete and the buffer has been transmitted to the decoder.
    bytes: Vec<u8>,

    // Image metadata, if available.
    metadata: Option<ImageMetadata>,

    // Once loading is complete, the result of the operation.
    result: Option<Result<(), String>>,
    listeners: Vec<ImageListener>,

    // The url being loaded. Do not forget that this may be several Mb
    // if we are loading a data: url.
    url: Arc<Url>
}

enum LoadResult {
    Loaded(Image),
    PlaceholderLoaded(Arc<Image>),
    None
}

impl PendingLoad {
    fn new(url: Arc<Url>) -> PendingLoad {
        PendingLoad {
            bytes: vec!(),
            metadata: None,
            result: None,
            listeners: vec!(),
            url: url,
        }
    }

    fn add_listener(&mut self, listener: ImageListener) {
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
    url_to_load_key: HashMap<Arc<Url>, LoadKey>,

    // A counter used to generate instances of LoadKey
    keygen: LoadKeyGenerator,
}

// Result of accessing a cache.
#[derive(Eq, PartialEq)]
enum CacheResult {
    Hit,  // The value was in the cache.
    Miss, // The value was not in the cache and needed to be regenerated.
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

    // get a PendingLoad from its LoadKey. Prefer this to `get_by_url`,
    // for performance reasons.
    fn get_by_key_mut(&mut self, key: &LoadKey) -> Option<&mut PendingLoad> {
        self.loads.get_mut(key)
    }

    // get a PendingLoad from its url. When possible, prefer `get_by_key_mut`.
    fn get_by_url(&self, url: &Url) -> Option<&PendingLoad> {
        self.url_to_load_key.get(url).
            and_then(|load_key|
                self.loads.get(load_key)
                )
    }

    fn remove(&mut self, key: &LoadKey) -> Option<PendingLoad> {
        self.loads.remove(key).
            and_then(|pending_load| {
                self.url_to_load_key.remove(&pending_load.url).unwrap();
                Some(pending_load)
            })
    }

    fn get_cached(&mut self, url: Arc<Url>) -> (CacheResult, LoadKey, &mut PendingLoad) {
        match self.url_to_load_key.entry(url.clone()) {
            Occupied(url_entry) => {
                let load_key = url_entry.get();
                (CacheResult::Hit, *load_key, self.loads.get_mut(load_key).unwrap())
            }
            Vacant(url_entry) => {
                let load_key = self.keygen.next();
                url_entry.insert(load_key);

                let pending_load = PendingLoad::new(url);
                match self.loads.entry(load_key) {
                    Occupied(_) => unreachable!(),
                    Vacant(load_entry) => {
                        let mut_load = load_entry.insert(pending_load);
                        (CacheResult::Miss, load_key, mut_load)
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
}

impl CompletedLoad {
    fn new(image_response: ImageResponse) -> CompletedLoad {
        CompletedLoad {
            image_response: image_response,
        }
    }
}

/// Stores information to notify a client when the state
/// of an image changes.
struct ImageListener {
    sender: ImageCacheChan,
    responder: Option<ImageResponder>,
    send_metadata_msg: bool,
}

// A key used to communicate during loading.
#[derive(Eq, Hash, PartialEq, Clone, Copy)]
struct LoadKey(u64);

struct LoadKeyGenerator {
    counter: u64
}

impl LoadKeyGenerator {
    fn new() -> LoadKeyGenerator {
        LoadKeyGenerator {
            counter: 0
        }
    }
    fn next(&mut self) -> LoadKey {
        self.counter += 1;
        LoadKey(self.counter)
    }
}

impl ImageListener {
    fn new(sender: ImageCacheChan, responder: Option<ImageResponder>, send_metadata_msg: bool) -> ImageListener {
        ImageListener {
            sender: sender,
            responder: responder,
            send_metadata_msg: send_metadata_msg,
        }
    }

    fn notify(&self, image_response: ImageResponse) {
        if !self.send_metadata_msg {
            if let ImageResponse::MetadataLoaded(_) = image_response {
                return;
            }
        }

        let ImageCacheChan(ref sender) = self.sender;
        let msg = ImageCacheResult {
            responder: self.responder.clone(),
            image_response: image_response,
        };
        sender.send(msg).ok();
    }
}

struct ResourceLoadInfo {
    action: ResponseAction,
    key: LoadKey,
}

/// Implementation of the image cache
struct ImageCache {
    // Receive commands from clients
    cmd_receiver: Receiver<ImageCacheCommand>,

    // Receive notifications from the resource thread
    progress_receiver: Receiver<ResourceLoadInfo>,
    progress_sender: Sender<ResourceLoadInfo>,

    // Receive notifications from the decoder thread pool
    decoder_receiver: Receiver<DecoderMsg>,
    decoder_sender: Sender<DecoderMsg>,

    // Worker threads for decoding images.
    thread_pool: ThreadPool,

    // Resource thread handle
    resource_thread: ResourceThread,

    // Images that are loading over network, or decoding.
    pending_loads: AllPendingLoads,

    // Images that have finished loading (successful or not)
    completed_loads: HashMap<Arc<Url>, CompletedLoad>,

    // The placeholder image used when an image fails to load
    placeholder_image: Option<Arc<Image>>,

    // Webrender API instance, if enabled.
    webrender_api: Option<webrender_traits::RenderApi>,
}

/// Message that the decoder worker threads send to main image cache thread.
struct DecoderMsg {
    key: LoadKey,
    image: Option<Image>,
}

/// The types of messages that the main image cache thread receives.
enum SelectResult {
    Command(ImageCacheCommand),
    Progress(ResourceLoadInfo),
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

impl ImageCache {
    fn run(&mut self) {
        let mut exit_sender: Option<IpcSender<()>> = None;

        loop {
            let result = {
                let sel = Select::new();

                let mut cmd_handle = sel.handle(&self.cmd_receiver);
                let mut progress_handle = sel.handle(&self.progress_receiver);
                let mut decoder_handle = sel.handle(&self.decoder_receiver);

                unsafe {
                    cmd_handle.add();
                    progress_handle.add();
                    decoder_handle.add();
                }

                let ret = sel.wait();

                if ret == cmd_handle.id() {
                    SelectResult::Command(self.cmd_receiver.recv().unwrap())
                } else if ret == decoder_handle.id() {
                    SelectResult::Decoder(self.decoder_receiver.recv().unwrap())
                } else {
                    SelectResult::Progress(self.progress_receiver.recv().unwrap())
                }
            };

            match result {
                SelectResult::Command(cmd) => {
                    exit_sender = self.handle_cmd(cmd);
                }
                SelectResult::Progress(msg) => {
                    self.handle_progress(msg);
                }
                SelectResult::Decoder(msg) => {
                    self.handle_decoder(msg);
                }
            }

            // Can only exit when all pending loads are complete.
            if let Some(ref exit_sender) = exit_sender {
                if self.pending_loads.is_empty() {
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
            ImageCacheCommand::RequestImage(url, result_chan, responder) => {
                self.request_image(url, result_chan, responder, false);
            }
            ImageCacheCommand::RequestImageAndMetadata(url, result_chan, responder) => {
                self.request_image(url, result_chan, responder, true);
            }
            ImageCacheCommand::GetImageIfAvailable(url, use_placeholder, consumer) => {
                let result = self.get_image_if_available(url, use_placeholder);
                consumer.send(result).unwrap();
            }
            ImageCacheCommand::GetImageOrMetadataIfAvailable(url, use_placeholder, consumer) => {
                let result = self.get_image_or_meta_if_available(url, use_placeholder);
                consumer.send(result).unwrap();
            }
        };

        None
    }

    // Handle progress messages from the resource thread
    fn handle_progress(&mut self, msg: ResourceLoadInfo) {
        match (msg.action, msg.key) {
            (ResponseAction::HeadersAvailable(_), _) => {}
            (ResponseAction::DataAvailable(data), _) => {
                let pending_load = self.pending_loads.get_by_key_mut(&msg.key).unwrap();
                pending_load.bytes.extend_from_slice(&data);
                //jmr0 TODO: possibly move to another task?
                if let None = pending_load.metadata {
                    if let Ok(metadata) = load_from_buf(&pending_load.bytes) {
                        let dimensions = metadata.dimensions();
                        let img_metadata = ImageMetadata { width: dimensions.width,
                                                         height: dimensions.height };
                        pending_load.metadata = Some(img_metadata.clone());
                        for listener in &pending_load.listeners {
                            listener.notify(ImageResponse::MetadataLoaded(img_metadata.clone()).clone());
                        }
                    }
                }
            }
            (ResponseAction::ResponseComplete(result), key) => {
                match result {
                    Ok(()) => {
                        let pending_load = self.pending_loads.get_by_key_mut(&msg.key).unwrap();
                        pending_load.result = Some(result);
                        let bytes = mem::replace(&mut pending_load.bytes, vec!());
                        let sender = self.decoder_sender.clone();

                        self.thread_pool.execute(move || {
                            let image = load_from_memory(&bytes);
                            let msg = DecoderMsg {
                                key: key,
                                image: image
                            };
                            sender.send(msg).unwrap();
                        });
                    }
                    Err(_) => {
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
        let pending_load = self.pending_loads.remove(&key).unwrap();

        if let Some(ref webrender_api) = self.webrender_api {
            match load_result {
                LoadResult::Loaded(ref mut image) => {
                    let format = convert_format(image.format);
                    let mut bytes = Vec::new();
                    bytes.extend_from_slice(&*image.bytes);
                    image.id = Some(webrender_api.add_image(image.width, image.height, format, bytes));
                }
                LoadResult::PlaceholderLoaded(..) | LoadResult::None => {}
            }
        }

        let image_response = match load_result {
            LoadResult::Loaded(image) => ImageResponse::Loaded(Arc::new(image)),
            LoadResult::PlaceholderLoaded(image) => ImageResponse::PlaceholderLoaded(image),
            LoadResult::None => ImageResponse::None,
        };

        let completed_load = CompletedLoad::new(image_response.clone());
        self.completed_loads.insert(pending_load.url, completed_load);

        for listener in pending_load.listeners {
            listener.notify(image_response.clone());
        }
    }

    // Request an image from the cache.  If the image hasn't been
    // loaded/decoded yet, it will be loaded/decoded in the
    // background. If send_metadata_msg is set, the channel will be notified
    // that image metadata is available, possibly before the image has finished
    // loading.
    fn request_image(&mut self,
                     url: Url,
                     result_chan: ImageCacheChan,
                     responder: Option<ImageResponder>,
                     send_metadata_msg: bool) {
        let image_listener = ImageListener::new(result_chan, responder, send_metadata_msg);
        // Let's avoid copying url everywhere.
        let ref_url = Arc::new(url);

        // Check if already completed
        match self.completed_loads.get(&ref_url) {
            Some(completed_load) => {
                // It's already completed, return a notify straight away
                image_listener.notify(completed_load.image_response.clone());
            }
            None => {
                // Check if the load is already pending
                let (cache_result, load_key, mut pending_load) = self.pending_loads.get_cached(ref_url.clone());
                pending_load.add_listener(image_listener);
                match cache_result {
                    CacheResult::Miss => {
                        // A new load request! Request the load from
                        // the resource thread.
                        let load_data = LoadData::new(LoadContext::Image, (*ref_url).clone(), None);
                        let (action_sender, action_receiver) = ipc::channel().unwrap();
                        let response_target = AsyncResponseTarget {
                            sender: action_sender,
                        };
                        let msg = ControlMsg::Load(load_data,
                                                   LoadConsumer::Listener(response_target),
                                                   None);
                        let progress_sender = self.progress_sender.clone();
                        ROUTER.add_route(action_receiver.to_opaque(), box move |message| {
                            let action: ResponseAction = message.to().unwrap();
                            progress_sender.send(ResourceLoadInfo {
                                action: action,
                                key: load_key,
                            }).unwrap();
                        });
                        self.resource_thread.send(msg).unwrap();
                    }
                    CacheResult::Hit => {
                        // Request is already on its way.
                    }
                }
            }
        }
    }

    fn get_image_if_available(&mut self,
                                   url: Url,
                                   placeholder: UsePlaceholder, )
                                   -> Result<Arc<Image>, ImageState> {
       let img_or_metadata = self.get_image_or_meta_if_available(url, placeholder);
       match img_or_metadata {
            Ok(ImageOrMetadataAvailable::ImageAvailable(image)) => Ok(image),
            Ok(ImageOrMetadataAvailable::MetadataAvailable(_)) => Err(ImageState::Pending),
            Err(err) => Err(err),
       }
    }

    fn get_image_or_meta_if_available(&mut self,
                                      url: Url,
                                      placeholder: UsePlaceholder)
                                      -> Result<ImageOrMetadataAvailable, ImageState> {
        match self.completed_loads.get(&url) {
            Some(completed_load) => {
                match (completed_load.image_response.clone(), placeholder) {
                    (ImageResponse::Loaded(image), _) |
                    (ImageResponse::PlaceholderLoaded(image), UsePlaceholder::Yes) => {
                        Ok(ImageOrMetadataAvailable::ImageAvailable(image))
                    }
                    (ImageResponse::PlaceholderLoaded(_), UsePlaceholder::No) |
                    (ImageResponse::None, _) |
                    (ImageResponse::MetadataLoaded(_), _) => {
                        Err(ImageState::LoadError)
                    }
                }
            }
            None => {
                self.pending_loads.get_by_url(&url).as_ref().
                    map_or(Err(ImageState::NotRequested), |pl| pl.metadata.as_ref().
                    map_or(Err(ImageState::Pending), |meta|
                           Ok(ImageOrMetadataAvailable::MetadataAvailable(meta.clone()))
                          )
                    )
            }
        }
    }
}

/// Create a new image cache.
pub fn new_image_cache_thread(resource_thread: ResourceThread,
                              webrender_api: Option<webrender_traits::RenderApi>) -> ImageCacheThread {
    let (ipc_command_sender, ipc_command_receiver) = ipc::channel().unwrap();
    let (progress_sender, progress_receiver) = channel();
    let (decoder_sender, decoder_receiver) = channel();

    spawn_named("ImageCacheThread".to_owned(), move || {

        // Preload the placeholder image, used when images fail to load.
        let mut placeholder_path = resources_dir_path();
        placeholder_path.push("rippy.png");

        let mut image_data = vec![];
        let result = File::open(&placeholder_path).and_then(|mut file| {
            file.read_to_end(&mut image_data)
        });
        let placeholder_image = result.ok().map(|_| {
            let mut image = load_from_memory(&image_data).unwrap();
            if let Some(ref webrender_api) = webrender_api {
                let format = convert_format(image.format);
                let mut bytes = Vec::new();
                bytes.extend_from_slice(&*image.bytes);
                image.id = Some(webrender_api.add_image(image.width, image.height, format, bytes));
            }
            Arc::new(image)
        });

        // Ask the router to proxy messages received over IPC to us.
        let cmd_receiver = ROUTER.route_ipc_receiver_to_new_mpsc_receiver(ipc_command_receiver);

        let mut cache = ImageCache {
            cmd_receiver: cmd_receiver,
            progress_sender: progress_sender,
            progress_receiver: progress_receiver,
            decoder_sender: decoder_sender,
            decoder_receiver: decoder_receiver,
            thread_pool: ThreadPool::new(4),
            pending_loads: AllPendingLoads::new(),
            completed_loads: HashMap::new(),
            resource_thread: resource_thread,
            placeholder_image: placeholder_image,
            webrender_api: webrender_api,
        };

        cache.run();
    });

    ImageCacheThread::new(ipc_command_sender)
}
