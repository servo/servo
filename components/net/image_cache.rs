/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::OnceCell;
use std::cmp::min;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::{mem, thread};

use base::id::{PipelineId, WebViewId};
use base::threadpool::ThreadPool;
use compositing_traits::{CrossProcessPaintApi, ImageUpdate, SerializableImageData};
use imsz::imsz_from_reader;
use log::{debug, warn};
use malloc_size_of::{MallocConditionalSizeOf, MallocSizeOf as MallocSizeOfTrait, MallocSizeOfOps};
use malloc_size_of_derive::MallocSizeOf;
use mime::Mime;
use net_traits::image_cache::{
    Image, ImageCache, ImageCacheFactory, ImageCacheResponseCallback, ImageCacheResponseMessage,
    ImageCacheResult, ImageLoadListener, ImageOrMetadataAvailable, ImageResponse, PendingImageId,
    RasterizationCompleteResponse, VectorImage,
};
use net_traits::request::CorsSettings;
use net_traits::{FetchMetadata, FetchResponseMsg, FilteredMetadata, NetworkError};
use parking_lot::Mutex;
use pixels::{CorsStatus, ImageFrame, ImageMetadata, PixelFormat, RasterImage, load_from_memory};
use profile_traits::mem::{Report, ReportKind};
use profile_traits::path;
use resvg::tiny_skia;
use resvg::usvg::{self, fontdb};
use rustc_hash::FxHashMap;
use servo_config::pref;
use servo_url::{ImmutableOrigin, ServoUrl};
use webrender_api::ImageKey as WebRenderImageKey;
use webrender_api::units::DeviceIntSize;

// We bake in rippy.png as a fallback, in case the embedder does not provide a broken
// image icon resource. This version is 229 bytes, so don't exchange it against
// something of higher resolution.
const FALLBACK_RIPPY: &[u8] = include_bytes!("../../resources/rippy.png");

/// The current SVG stack relies on `resvg` to provide the natural dimensions of
/// the SVG, which it automatically infers from the width/height/viewBox properties
/// of the SVG. Since these can be arbitrarily large, this can cause us to allocate
/// a pixmap with very large dimensions leading to the process being killed due to
/// memory exhaustion. For example, the `/css/css-transforms/perspective-svg-001.html`
/// test uses very large values for viewBox. Hence, we just clamp the maximum
/// width/height of the pixmap allocated for rasterization.
const MAX_SVG_PIXMAP_DIMENSION: u32 = 5000;

//
// TODO(gw): Remaining work on image cache:
//     * Make use of the prefetch support in various parts of the code.
//     * Profile time in GetImageIfAvailable - might be worth caching these
//       results per paint / layout.
//
// MAYBE(Yoric):
//     * For faster lookups, it might be useful to store the LoadKey in the
//       DOM once we have performed a first load.

// ======================================================================
// Helper functions.
// ======================================================================

fn parse_svg_document_in_memory(
    bytes: &[u8],
    fontdb: Arc<fontdb::Database>,
) -> Result<usvg::Tree, &'static str> {
    let image_string_href_resolver = Box::new(move |_: &str, _: &usvg::Options| {
        // Do not try to load `href` in <image> as local file path.
        None
    });

    let opt = usvg::Options {
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: usvg::ImageHrefResolver::default_data_resolver(),
            resolve_string: image_string_href_resolver,
        },
        fontdb,
        ..usvg::Options::default()
    };

    usvg::Tree::from_data(bytes, &opt)
        .inspect_err(|error| {
            warn!("Error when parsing SVG data: {error}");
        })
        .map_err(|_| "Not a valid SVG document")
}

fn decode_bytes_sync(
    key: LoadKey,
    bytes: &[u8],
    cors: CorsStatus,
    content_type: Option<Mime>,
    fontdb: Arc<fontdb::Database>,
) -> DecoderMsg {
    let is_svg_document = content_type.is_some_and(|content_type| {
        (
            content_type.type_(),
            content_type.subtype(),
            content_type.suffix(),
        ) == (mime::IMAGE, mime::SVG, Some(mime::XML))
    });

    let image = if is_svg_document {
        parse_svg_document_in_memory(bytes, fontdb)
            .ok()
            .map(|svg_tree| {
                DecodedImage::Vector(VectorImageData {
                    svg_tree: Arc::new(svg_tree),
                    cors_status: cors,
                })
            })
    } else {
        load_from_memory(bytes, cors).map(DecodedImage::Raster)
    };

    DecoderMsg { key, image }
}

fn set_webrender_image_key(
    paint_api: &CrossProcessPaintApi,
    image: &mut RasterImage,
    image_key: WebRenderImageKey,
) {
    if image.id.is_some() {
        return;
    }

    let (descriptor, ipc_shared_memory) = image.webrender_image_descriptor_and_data_for_frame(0);
    let data = SerializableImageData::Raw(ipc_shared_memory);

    paint_api.add_image(image_key, descriptor, data);
    image.id = Some(image_key);
}

// ======================================================================
// Aux structs and enums.
// ======================================================================

/// <https://html.spec.whatwg.org/multipage/#list-of-available-images>
type ImageKey = (ServoUrl, ImmutableOrigin, Option<CorsSettings>);

// Represents all the currently pending loads/decodings. For
// performance reasons, loads are indexed by a dedicated load key.
#[derive(MallocSizeOf)]
struct AllPendingLoads {
    // The loads, indexed by a load key. Used during most operations,
    // for performance reasons.
    loads: FxHashMap<LoadKey, PendingLoad>,

    // Get a load key from its url and requesting origin. Used ony when starting and
    // finishing a load or when adding a new listener.
    url_to_load_key: HashMap<ImageKey, LoadKey>,

    // A counter used to generate instances of LoadKey
    keygen: LoadKeyGenerator,
}

impl AllPendingLoads {
    fn new() -> AllPendingLoads {
        AllPendingLoads {
            loads: FxHashMap::default(),
            url_to_load_key: HashMap::default(),
            keygen: LoadKeyGenerator::new(),
        }
    }

    // get a PendingLoad from its LoadKey.
    fn get_by_key_mut(&mut self, key: &LoadKey) -> Option<&mut PendingLoad> {
        self.loads.get_mut(key)
    }

    fn remove(&mut self, key: &LoadKey) -> Option<PendingLoad> {
        self.loads.remove(key).inspect(|pending_load| {
            self.url_to_load_key
                .remove(&(
                    pending_load.url.clone(),
                    pending_load.load_origin.clone(),
                    pending_load.cors_setting,
                ))
                .unwrap();
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
#[derive(MallocSizeOf)]
struct CompletedLoad {
    image_response: ImageResponse,
    id: PendingImageId,
}

impl CompletedLoad {
    fn new(image_response: ImageResponse, id: PendingImageId) -> CompletedLoad {
        CompletedLoad { image_response, id }
    }
}

#[derive(Clone, MallocSizeOf)]
struct VectorImageData {
    #[conditional_malloc_size_of]
    svg_tree: Arc<usvg::Tree>,
    cors_status: CorsStatus,
}

impl std::fmt::Debug for VectorImageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorImageData").finish()
    }
}

enum DecodedImage {
    Raster(RasterImage),
    Vector(VectorImageData),
}

/// Message that the decoder worker threads send to the image cache.
struct DecoderMsg {
    key: LoadKey,
    image: Option<DecodedImage>,
}

#[derive(MallocSizeOf)]
enum ImageBytes {
    InProgress(Vec<u8>),
    Complete(#[conditional_malloc_size_of] Arc<Vec<u8>>),
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

#[derive(MallocSizeOf)]
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
    LoadedRasterImage(RasterImage),
    LoadedVectorImage(VectorImageData),
    FailedToLoadOrDecode,
}

/// Represents an image that is either being loaded
/// by the resource thread, or decoded by a worker thread.
#[derive(MallocSizeOf)]
struct PendingLoad {
    /// The bytes loaded so far. Reset to an empty vector once loading
    /// is complete and the buffer has been transmitted to the decoder.
    bytes: ImageBytes,

    /// Image metadata, if available.
    metadata: Option<ImageMetadata>,

    /// Once loading is complete, the result of the operation.
    result: Option<Result<(), NetworkError>>,

    /// The listeners that are waiting for this response to complete.
    listeners: Vec<ImageLoadListener>,

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

    /// The MIME type from the `Content-type` header of the HTTP response, if any.
    content_type: Option<Mime>,
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
            content_type: None,
        }
    }

    fn add_listener(&mut self, listener: ImageLoadListener) {
        self.listeners.push(listener);
    }
}

#[derive(Default, MallocSizeOf)]
struct RasterizationTask {
    #[ignore_malloc_size_of = "Fn is difficult to measure"]
    listeners: Vec<(PipelineId, ImageCacheResponseCallback)>,
    result: Option<RasterImage>,
}

/// Used for storing images that do not have a `WebRenderImageKey` yet.
#[derive(Debug, MallocSizeOf)]
enum PendingKey {
    RasterImage((LoadKey, RasterImage)),
    Svg((LoadKey, RasterImage, DeviceIntSize)),
}

/// The state of the `WebRenderImageKey`` cache
#[derive(Debug, MallocSizeOf)]
enum KeyCacheState {
    /// We already requested a batch of keys.
    PendingBatch,
    /// We have some keys in the cache.
    Ready(Vec<WebRenderImageKey>),
}

impl KeyCacheState {
    fn size(&self) -> usize {
        match self {
            KeyCacheState::PendingBatch => 0,
            KeyCacheState::Ready(items) => items.len(),
        }
    }
}

/// As getting new keys takes a round trip over the constellation, we keep a small cache of them.
/// Additionally, this cache will store image resources that do not have a key yet because those
/// are needed to complete the load.
#[derive(MallocSizeOf)]
struct KeyCache {
    /// A cache of `WebRenderImageKey`.
    cache: KeyCacheState,
    /// These images are loaded but have no key assigned to yet.
    images_pending_keys: VecDeque<PendingKey>,
}

impl KeyCache {
    fn new() -> Self {
        KeyCache {
            cache: KeyCacheState::Ready(Vec::new()),
            images_pending_keys: VecDeque::new(),
        }
    }
}

/// ## Image cache implementation.
#[derive(MallocSizeOf)]
struct ImageCacheStore {
    /// Images that are loading over network, or decoding.
    pending_loads: AllPendingLoads,

    /// Images that have finished loading (successful or not)
    completed_loads: HashMap<ImageKey, CompletedLoad>,

    /// Vector (e.g. SVG) images that have been sucessfully loaded and parsed
    /// but are yet to be rasterized. Since the same SVG data can be used for
    /// rasterizing at different sizes, we use this hasmap to share the data.
    vector_images: FxHashMap<PendingImageId, VectorImageData>,

    /// Vector images for which rasterization at a particular size has started
    /// or completed. If completed, the `result` member of `RasterizationTask`
    /// contains the rasterized image.
    rasterized_vector_images: FxHashMap<(PendingImageId, DeviceIntSize), RasterizationTask>,

    /// The [`RasterImage`] used for the broken image icon, initialized lazily, only when necessary.
    #[conditional_malloc_size_of]
    broken_image_icon_image: OnceCell<Option<Arc<RasterImage>>>,

    /// Cross-process `Paint` API instance.
    #[ignore_malloc_size_of = "Channel from another crate"]
    paint_api: CrossProcessPaintApi,

    /// The [`WebView`] of the `Webview` associated with this [`ImageCache`].
    webview_id: WebViewId,

    /// The [`PipelineId`] of the `Pipeline` associated with this [`ImageCache`].
    pipeline_id: PipelineId,

    /// Main struct to handle the cache of `WebRenderImageKey` and
    /// images that do not have a key yet.
    key_cache: KeyCache,
}

impl ImageCacheStore {
    /// Finishes loading the image by setting the WebRenderImageKey and calling `compete_load` or `complete_load_svg`.
    fn set_key_and_finish_load(&mut self, pending_image: PendingKey, image_key: WebRenderImageKey) {
        match pending_image {
            PendingKey::RasterImage((pending_id, mut raster_image)) => {
                set_webrender_image_key(&self.paint_api, &mut raster_image, image_key);
                self.complete_load(pending_id, LoadResult::LoadedRasterImage(raster_image));
            },
            PendingKey::Svg((pending_id, mut raster_image, requested_size)) => {
                set_webrender_image_key(&self.paint_api, &mut raster_image, image_key);
                self.complete_load_svg(raster_image, pending_id, requested_size);
            },
        }
    }

    /// If a key is available the image will be immediately loaded, otherwise it will load then the next batch of
    /// keys is received. Only call this if the image does not have a `LoadKey` yet.
    fn load_image_with_keycache(&mut self, pending_image: PendingKey) {
        match self.key_cache.cache {
            KeyCacheState::PendingBatch => {
                self.key_cache.images_pending_keys.push_back(pending_image);
            },
            KeyCacheState::Ready(ref mut cache) => match cache.pop() {
                Some(image_key) => {
                    self.set_key_and_finish_load(pending_image, image_key);
                },
                None => {
                    self.key_cache.images_pending_keys.push_back(pending_image);
                    self.fetch_more_image_keys();
                },
            },
        }
    }

    fn fetch_more_image_keys(&mut self) {
        self.key_cache.cache = KeyCacheState::PendingBatch;
        self.paint_api
            .generate_image_key_async(self.webview_id, self.pipeline_id);
    }

    /// Insert received keys into the cache and complete the loading of images.
    fn insert_keys_and_load_images(&mut self, image_keys: Vec<WebRenderImageKey>) {
        if let KeyCacheState::PendingBatch = self.key_cache.cache {
            self.key_cache.cache = KeyCacheState::Ready(image_keys);
            let len = min(
                self.key_cache.cache.size(),
                self.key_cache.images_pending_keys.len(),
            );
            let images = self
                .key_cache
                .images_pending_keys
                .drain(0..len)
                .collect::<Vec<PendingKey>>();
            for key in images {
                self.load_image_with_keycache(key);
            }
            if !self.key_cache.images_pending_keys.is_empty() {
                self.paint_api
                    .generate_image_key_async(self.webview_id, self.pipeline_id);
                self.key_cache.cache = KeyCacheState::PendingBatch
            }
        } else {
            unreachable!("A batch was received while we didn't request one")
        }
    }

    /// Complete the loading the of the rasterized svg image. This needs the `RasterImage` to
    /// already have a `WebRenderImageKey`.
    fn complete_load_svg(
        &mut self,
        rasterized_image: RasterImage,
        pending_image_id: PendingImageId,
        requested_size: DeviceIntSize,
    ) {
        let listeners = {
            self.rasterized_vector_images
                .get_mut(&(pending_image_id, requested_size))
                .map(|task| {
                    task.result = Some(rasterized_image);
                    std::mem::take(&mut task.listeners)
                })
                .unwrap_or_default()
        };

        for (pipeline_id, callback) in listeners {
            callback(ImageCacheResponseMessage::VectorImageRasterizationComplete(
                RasterizationCompleteResponse {
                    pipeline_id,
                    image_id: pending_image_id,
                    requested_size,
                },
            ));
        }
    }

    /// The rest of complete load. This requires that images have a valid `WebRenderImageKey`.
    fn complete_load(&mut self, key: LoadKey, load_result: LoadResult) {
        debug!("Completed decoding for {:?}", load_result);
        let pending_load = match self.pending_loads.remove(&key) {
            Some(load) => load,
            None => return,
        };

        let url = pending_load.final_url.clone();
        let image_response = match load_result {
            LoadResult::LoadedRasterImage(raster_image) => {
                assert!(raster_image.id.is_some());
                ImageResponse::Loaded(Image::Raster(Arc::new(raster_image)), url.unwrap())
            },
            LoadResult::LoadedVectorImage(vector_image) => {
                self.vector_images.insert(key, vector_image.clone());
                let natural_dimensions = vector_image.svg_tree.size().to_int_size();
                let metadata = ImageMetadata {
                    width: natural_dimensions.width(),
                    height: natural_dimensions.height(),
                };

                let vector_image = VectorImage {
                    id: key,
                    metadata,
                    cors_status: vector_image.cors_status,
                };
                ImageResponse::Loaded(Image::Vector(vector_image), url.unwrap())
            },
            LoadResult::FailedToLoadOrDecode => ImageResponse::FailedToLoadOrDecode,
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
    ) -> Option<Result<(Image, ServoUrl), ()>> {
        self.completed_loads
            .get(&(url, origin, cors_setting))
            .map(|completed_load| match &completed_load.image_response {
                ImageResponse::Loaded(image, url) => Ok((image.clone(), url.clone())),
                ImageResponse::FailedToLoadOrDecode | ImageResponse::MetadataLoaded(_) => Err(()),
            })
    }

    /// Handle a message from one of the decoder worker threads or from a sync
    /// decoding operation.
    fn handle_decoder(&mut self, msg: DecoderMsg) {
        let image = match msg.image {
            None => LoadResult::FailedToLoadOrDecode,
            Some(DecodedImage::Raster(raster_image)) => {
                self.load_image_with_keycache(PendingKey::RasterImage((msg.key, raster_image)));
                return;
            },
            Some(DecodedImage::Vector(vector_image_data)) => {
                LoadResult::LoadedVectorImage(vector_image_data)
            },
        };
        self.complete_load(msg.key, image);
    }
}

pub struct ImageCacheFactoryImpl {
    /// The data to use for the broken image icon used when images cannot load.
    broken_image_icon_data: Arc<Vec<u8>>,
    /// Thread pool for image decoding
    thread_pool: Arc<ThreadPool>,
    /// A shared font database to be used by system fonts accessed when rasterizing vector
    /// images.
    fontdb: Arc<fontdb::Database>,
}

impl ImageCacheFactoryImpl {
    pub fn new(broken_image_icon_data: Vec<u8>) -> Self {
        debug!("Creating new ImageCacheFactoryImpl");

        // Uses an estimate of the system cpus to decode images
        // See https://doc.rust-lang.org/stable/std/thread/fn.available_parallelism.html
        // If no information can be obtained about the system, uses 4 threads as a default
        let thread_count = thread::available_parallelism()
            .map(|i| i.get())
            .unwrap_or(pref!(threadpools_fallback_worker_num) as usize)
            .min(pref!(threadpools_image_cache_workers_max).max(1) as usize);

        let mut fontdb = fontdb::Database::new();
        fontdb.load_system_fonts();

        Self {
            broken_image_icon_data: Arc::new(broken_image_icon_data),
            thread_pool: Arc::new(ThreadPool::new(thread_count, "ImageCache".to_string())),
            fontdb: Arc::new(fontdb),
        }
    }
}

impl ImageCacheFactory for ImageCacheFactoryImpl {
    fn create(
        &self,
        webview_id: WebViewId,
        pipeline_id: PipelineId,
        paint_api: &CrossProcessPaintApi,
    ) -> Arc<dyn ImageCache> {
        Arc::new(ImageCacheImpl {
            store: Arc::new(Mutex::new(ImageCacheStore {
                pending_loads: AllPendingLoads::new(),
                completed_loads: HashMap::new(),
                vector_images: FxHashMap::default(),
                rasterized_vector_images: FxHashMap::default(),
                broken_image_icon_image: OnceCell::new(),
                paint_api: paint_api.clone(),
                pipeline_id,
                webview_id,
                key_cache: KeyCache::new(),
            })),
            broken_image_icon_data: self.broken_image_icon_data.clone(),
            thread_pool: self.thread_pool.clone(),
            fontdb: self.fontdb.clone(),
        })
    }
}

pub struct ImageCacheImpl {
    /// Per-[`ImageCache`] data.
    store: Arc<Mutex<ImageCacheStore>>,
    /// The data to use for the broken image icon used when images cannot load.
    broken_image_icon_data: Arc<Vec<u8>>,
    /// Thread pool for image decoding. This is shared with other [`ImageCache`]s in the
    /// same process.
    thread_pool: Arc<ThreadPool>,
    /// A shared font database to be used by system fonts accessed when rasterizing vector
    /// images. This is shared with other [`ImageCache`]s in the same process.
    fontdb: Arc<fontdb::Database>,
}

impl ImageCache for ImageCacheImpl {
    fn memory_reports(&self, prefix: &str, ops: &mut MallocSizeOfOps) -> Vec<Report> {
        let store_size = self.store.lock().size_of(ops);
        let fontdb_size = self.fontdb.conditional_size_of(ops);
        vec![
            Report {
                path: path![prefix, "image-cache"],
                kind: ReportKind::ExplicitSystemHeapSize,
                size: store_size,
            },
            Report {
                path: path![prefix, "image-cache", "fontdb"],
                kind: ReportKind::ExplicitSystemHeapSize,
                size: fontdb_size,
            },
        ]
    }

    fn get_image_key(&self) -> Option<WebRenderImageKey> {
        let mut store = self.store.lock();
        if let KeyCacheState::Ready(ref mut cache) = store.key_cache.cache {
            if let Some(image_key) = cache.pop() {
                return Some(image_key);
            }

            store.fetch_more_image_keys();
        }

        store
            .paint_api
            .generate_image_key_blocking(store.webview_id)
    }

    fn get_image(
        &self,
        url: ServoUrl,
        origin: ImmutableOrigin,
        cors_setting: Option<CorsSettings>,
    ) -> Option<Image> {
        let store = self.store.lock();
        let result = store.get_completed_image_if_available(url, origin, cors_setting);
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
    ) -> ImageCacheResult {
        let mut store = self.store.lock();
        if let Some(result) =
            store.get_completed_image_if_available(url.clone(), origin.clone(), cors_setting)
        {
            match result {
                Ok((image, image_url)) => {
                    debug!("{} is available", url);
                    return ImageCacheResult::Available(ImageOrMetadataAvailable::ImageAvailable {
                        image,
                        url: image_url,
                    });
                },
                Err(()) => {
                    debug!("{} is not available", url);
                    return ImageCacheResult::FailedToLoadOrDecode;
                },
            }
        }

        let (key, decoded) = {
            let result = store
                .pending_loads
                .get_cached(url.clone(), origin.clone(), cors_setting);
            match result {
                CacheResult::Hit(key, pl) => match (&pl.result, &pl.metadata) {
                    (&Some(Ok(_)), _) => {
                        debug!("Sync decoding {} ({:?})", url, key);
                        (
                            key,
                            decode_bytes_sync(
                                key,
                                pl.bytes.as_slice(),
                                pl.cors_status,
                                pl.content_type.clone(),
                                self.fontdb.clone(),
                            ),
                        )
                    },
                    (&None, Some(meta)) => {
                        debug!("Metadata available for {} ({:?})", url, key);
                        return ImageCacheResult::Available(
                            ImageOrMetadataAvailable::MetadataAvailable(*meta, key),
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
                    return ImageCacheResult::FailedToLoadOrDecode;
                },
            }
        };

        // In the case where a decode is ongoing (or waiting in a queue) but we
        // have the full response available, we decode the bytes synchronously
        // and ignore the async decode when it finishes later.
        // TODO: make this behaviour configurable according to the caller's needs.
        store.handle_decoder(decoded);
        match store.get_completed_image_if_available(url, origin, cors_setting) {
            Some(Ok((image, image_url))) => {
                ImageCacheResult::Available(ImageOrMetadataAvailable::ImageAvailable {
                    image,
                    url: image_url,
                })
            },
            // Note: this happens if we are pending a batch of image keys.
            _ => ImageCacheResult::Pending(key),
        }
    }

    fn add_rasterization_complete_listener(
        &self,
        pipeline_id: PipelineId,
        image_id: PendingImageId,
        requested_size: DeviceIntSize,
        callback: ImageCacheResponseCallback,
    ) {
        {
            let mut store = self.store.lock();
            let key = (image_id, requested_size);
            if !store.vector_images.contains_key(&image_id) {
                warn!("Unknown image requested for rasterization for key {key:?}");
                return;
            };

            let Some(task) = store.rasterized_vector_images.get_mut(&key) else {
                warn!("Image rasterization task not found in the cache for key {key:?}");
                return;
            };

            // If `result` is `None`, the task is still pending.
            if task.result.is_none() {
                task.listeners.push((pipeline_id, callback));
                return;
            }
        }

        callback(ImageCacheResponseMessage::VectorImageRasterizationComplete(
            RasterizationCompleteResponse {
                pipeline_id,
                image_id,
                requested_size,
            },
        ));
    }

    fn rasterize_vector_image(
        &self,
        image_id: PendingImageId,
        requested_size: DeviceIntSize,
    ) -> Option<RasterImage> {
        let mut store = self.store.lock();
        let Some(vector_image) = store.vector_images.get(&image_id).cloned() else {
            warn!("Unknown image id {image_id:?} requested for rasterization");
            return None;
        };

        // This early return relies on the fact that the result of image rasterization cannot
        // ever be `None`. If that were the case we would need to check whether the entry
        // in the `HashMap` was `Occupied` or not.
        let entry = store
            .rasterized_vector_images
            .entry((image_id, requested_size))
            .or_default();
        if let Some(result) = entry.result.as_ref() {
            return Some(result.clone());
        }

        let store = self.store.clone();
        self.thread_pool.spawn(move || {
            let natural_size = vector_image.svg_tree.size().to_int_size();
            let tinyskia_requested_size = {
                let width = requested_size
                    .width
                    .try_into()
                    .unwrap_or(0)
                    .min(MAX_SVG_PIXMAP_DIMENSION);
                let height = requested_size
                    .height
                    .try_into()
                    .unwrap_or(0)
                    .min(MAX_SVG_PIXMAP_DIMENSION);
                tiny_skia::IntSize::from_wh(width, height).unwrap_or(natural_size)
            };
            let transform = tiny_skia::Transform::from_scale(
                tinyskia_requested_size.width() as f32 / natural_size.width() as f32,
                tinyskia_requested_size.height() as f32 / natural_size.height() as f32,
            );
            let mut pixmap = tiny_skia::Pixmap::new(
                tinyskia_requested_size.width(),
                tinyskia_requested_size.height(),
            )
            .unwrap();
            resvg::render(&vector_image.svg_tree, transform, &mut pixmap.as_mut());

            let bytes = pixmap.take();
            let frame = ImageFrame {
                delay: None,
                byte_range: 0..bytes.len(),
                width: tinyskia_requested_size.width(),
                height: tinyskia_requested_size.height(),
            };

            let rasterized_image = RasterImage {
                metadata: ImageMetadata {
                    width: tinyskia_requested_size.width(),
                    height: tinyskia_requested_size.height(),
                },
                format: PixelFormat::RGBA8,
                frames: vec![frame],
                bytes: Arc::new(bytes),
                id: None,
                cors_status: vector_image.cors_status,
                is_opaque: false,
            };

            let mut store = store.lock();
            store.load_image_with_keycache(PendingKey::Svg((
                image_id,
                rasterized_image,
                requested_size,
            )));
        });

        None
    }

    /// Add a new listener for the given pending image id. If the image is already present,
    /// the responder will still receive the expected response.
    fn add_listener(&self, listener: ImageLoadListener) {
        let mut store = self.store.lock();
        self.add_listener_with_store(&mut store, listener);
    }

    /// Inform the image cache about a response for a pending request.
    fn notify_pending_response(&self, id: PendingImageId, action: FetchResponseMsg) {
        match (action, id) {
            (FetchResponseMsg::ProcessRequestBody(..), _) |
            (FetchResponseMsg::ProcessRequestEOF(..), _) |
            (FetchResponseMsg::ProcessCspViolations(..), _) => (),
            (FetchResponseMsg::ProcessResponse(_, response), _) => {
                debug!("Received {:?} for {:?}", response.as_ref().map(|_| ()), id);
                let mut store = self.store.lock();
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
                pending_load.content_type = metadata
                    .as_ref()
                    .and_then(|metadata| metadata.content_type.clone())
                    .map(|content_type| content_type.into_inner().into());
            },
            (FetchResponseMsg::ProcessResponseChunk(_, data), _) => {
                debug!("Got some data for {:?}", id);
                let mut store = self.store.lock();
                let pending_load = store.pending_loads.get_by_key_mut(&id).unwrap();
                pending_load.bytes.extend_from_slice(&data);

                // jmr0 TODO: possibly move to another task?
                if pending_load.metadata.is_none() {
                    let mut reader = std::io::Cursor::new(pending_load.bytes.as_slice());
                    if let Ok(info) = imsz_from_reader(&mut reader) {
                        let img_metadata = ImageMetadata {
                            width: info.width as u32,
                            height: info.height as u32,
                        };
                        for listener in &pending_load.listeners {
                            listener.respond(ImageResponse::MetadataLoaded(img_metadata));
                        }
                        pending_load.metadata = Some(img_metadata);
                    }
                }
            },
            (FetchResponseMsg::ProcessResponseEOF(_, result), key) => {
                debug!("Received EOF for {:?}", key);
                match result {
                    Ok(_) => {
                        let (bytes, cors_status, content_type) = {
                            let mut store = self.store.lock();
                            let pending_load = store.pending_loads.get_by_key_mut(&id).unwrap();
                            pending_load.result = Some(Ok(()));
                            debug!("Async decoding {} ({:?})", pending_load.url, key);
                            (
                                pending_load.bytes.mark_complete(),
                                pending_load.cors_status,
                                pending_load.content_type.clone(),
                            )
                        };

                        let local_store = self.store.clone();
                        let fontdb = self.fontdb.clone();
                        self.thread_pool.spawn(move || {
                            let msg =
                                decode_bytes_sync(key, &bytes, cors_status, content_type, fontdb);
                            debug!("Image decoded");
                            local_store.lock().handle_decoder(msg);
                        });
                    },
                    Err(error) => {
                        debug!("Processing error for {key:?}: {error:?}");
                        let mut store = self.store.lock();
                        store.complete_load(id, LoadResult::FailedToLoadOrDecode)
                    },
                }
            },
        }
    }

    fn fill_key_cache_with_batch_of_keys(&self, image_keys: Vec<WebRenderImageKey>) {
        let mut store = self.store.lock();
        store.insert_keys_and_load_images(image_keys);
    }

    fn get_broken_image_icon(&self) -> Option<Arc<RasterImage>> {
        let store = self.store.lock();
        store
            .broken_image_icon_image
            .get_or_init(|| {
                let mut image = load_from_memory(&self.broken_image_icon_data, CorsStatus::Unsafe)
                    .or_else(|| load_from_memory(FALLBACK_RIPPY, CorsStatus::Unsafe))?;
                let image_key = store
                    .paint_api
                    .generate_image_key_blocking(store.webview_id)
                    .expect("Could not generate image key for broken image icon");
                set_webrender_image_key(&store.paint_api, &mut image, image_key);
                Some(Arc::new(image))
            })
            .clone()
    }
}

impl Drop for ImageCacheStore {
    fn drop(&mut self) {
        let image_updates = self
            .completed_loads
            .values()
            .filter_map(|load| match &load.image_response {
                ImageResponse::Loaded(Image::Raster(image), _) => {
                    image.id.map(ImageUpdate::DeleteImage)
                },
                _ => None,
            })
            .chain(
                self.rasterized_vector_images
                    .values()
                    .filter_map(|task| task.result.as_ref()?.id.map(ImageUpdate::DeleteImage)),
            )
            .collect();
        self.paint_api
            .update_images(self.webview_id.into(), image_updates);
    }
}

impl ImageCacheImpl {
    /// Require self.store.lock() before calling.
    fn add_listener_with_store(&self, store: &mut ImageCacheStore, listener: ImageLoadListener) {
        let id = listener.id;
        if let Some(load) = store.pending_loads.get_by_key_mut(&id) {
            if let Some(ref metadata) = load.metadata {
                listener.respond(ImageResponse::MetadataLoaded(*metadata));
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
