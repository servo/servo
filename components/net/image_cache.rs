/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp::min;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::{mem, thread};

use base::id::PipelineId;
use compositing_traits::{CrossProcessCompositorApi, ImageUpdate, SerializableImageData};
use imsz::imsz_from_reader;
use ipc_channel::ipc::{IpcSender, IpcSharedMemory};
use log::{debug, error, warn};
use malloc_size_of::{MallocSizeOf as MallocSizeOfTrait, MallocSizeOfOps};
use malloc_size_of_derive::MallocSizeOf;
use mime::Mime;
use net_traits::image_cache::{
    Image, ImageCache, ImageCacheResponseMessage, ImageCacheResult, ImageLoadListener,
    ImageOrMetadataAvailable, ImageResponse, PendingImageId, RasterizationCompleteResponse,
    UsePlaceholder, VectorImage,
};
use net_traits::request::CorsSettings;
use net_traits::{FetchMetadata, FetchResponseMsg, FilteredMetadata, NetworkError};
use pixels::{CorsStatus, ImageFrame, ImageMetadata, PixelFormat, RasterImage, load_from_memory};
use profile_traits::mem::{Report, ReportKind};
use profile_traits::path;
use resvg::{tiny_skia, usvg};
use servo_config::pref;
use servo_url::{ImmutableOrigin, ServoUrl};
use webrender_api::units::DeviceIntSize;
use webrender_api::{
    ImageDescriptor, ImageDescriptorFlags, ImageFormat, ImageKey as WebRenderImageKey,
};

use crate::resource_thread::CoreResourceThreadPool;

// We bake in rippy.png as a fallback, in case the embedder does not provide
// a rippy resource. this version is 253 bytes large, don't exchange it against
// something in higher resolution.
const FALLBACK_RIPPY: &[u8] = include_bytes!("../../resources/rippy.png");

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

fn parse_svg_document_in_memory(bytes: &[u8]) -> Result<usvg::Tree, &'static str> {
    let image_string_href_resolver = Box::new(move |_: &str, _: &usvg::Options| {
        // Do not try to load `href` in <image> as local file path.
        None
    });

    let mut opt = usvg::Options {
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: usvg::ImageHrefResolver::default_data_resolver(),
            resolve_string: image_string_href_resolver,
        },
        ..usvg::Options::default()
    };

    opt.fontdb_mut().load_system_fonts();

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
) -> DecoderMsg {
    let image = if content_type == Some(mime::IMAGE_SVG) {
        parse_svg_document_in_memory(bytes).ok().map(|svg_tree| {
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

/// This will block on getting an ImageKey
/// but that is ok because it is done once upon start-up of a script-thread.
fn get_placeholder_image(
    compositor_api: &CrossProcessCompositorApi,
    data: &[u8],
) -> Arc<RasterImage> {
    let mut image = load_from_memory(data, CorsStatus::Unsafe)
        .or_else(|| load_from_memory(FALLBACK_RIPPY, CorsStatus::Unsafe))
        .expect("load fallback image failed");
    let image_key = compositor_api
        .generate_image_key_blocking()
        .expect("Could not generate image key");
    set_webrender_image_key(compositor_api, &mut image, image_key);
    Arc::new(image)
}

fn set_webrender_image_key(
    compositor_api: &CrossProcessCompositorApi,
    image: &mut RasterImage,
    image_key: WebRenderImageKey,
) {
    if image.id.is_some() {
        return;
    }
    let mut bytes = Vec::new();
    let frame_bytes = image.first_frame().bytes;
    let is_opaque = match image.format {
        PixelFormat::BGRA8 | PixelFormat::RGBA8 => {
            bytes.extend_from_slice(frame_bytes);
            pixels::rgba8_premultiply_inplace(bytes.as_mut_slice())
        },
        PixelFormat::RGB8 => {
            bytes.reserve(frame_bytes.len() / 3 * 4);
            for bgr in frame_bytes.chunks(3) {
                bytes.extend_from_slice(&[bgr[2], bgr[1], bgr[0], 0xff]);
            }

            true
        },
        PixelFormat::K8 | PixelFormat::KA8 => {
            panic!("Not support by webrender yet");
        },
    };
    let format = if matches!(image.format, PixelFormat::RGBA8) {
        ImageFormat::RGBA8
    } else {
        ImageFormat::BGRA8
    };

    let mut flags = ImageDescriptorFlags::ALLOW_MIPMAPS;
    flags.set(ImageDescriptorFlags::IS_OPAQUE, is_opaque);

    let size = DeviceIntSize::new(image.metadata.width as i32, image.metadata.height as i32);
    let descriptor = ImageDescriptor {
        size,
        stride: None,
        format,
        offset: 0,
        flags,
    };
    let data = SerializableImageData::Raw(IpcSharedMemory::from_bytes(&bytes));
    compositor_api.add_image(image_key, descriptor, data);
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

#[derive(Clone, Debug, MallocSizeOf)]
struct VectorImageData {
    #[conditional_malloc_size_of]
    svg_tree: Arc<usvg::Tree>,
    cors_status: CorsStatus,
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
    PlaceholderLoaded(Arc<RasterImage>),
    None,
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
    listeners: Vec<(PipelineId, IpcSender<ImageCacheResponseMessage>)>,
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
    vector_images: HashMap<PendingImageId, VectorImageData>,

    /// Vector images for which rasterization at a particular size has started
    /// or completed. If completed, the `result` member of `RasterizationTask`
    /// contains the rasterized image.
    rasterized_vector_images: HashMap<(PendingImageId, DeviceIntSize), RasterizationTask>,

    /// The placeholder image used when an image fails to load
    #[conditional_malloc_size_of]
    placeholder_image: Arc<RasterImage>,

    /// The URL used for the placeholder image
    placeholder_url: ServoUrl,

    /// Cross-process compositor API instance.
    #[ignore_malloc_size_of = "Channel from another crate"]
    compositor_api: CrossProcessCompositorApi,

    // The PipelineId will initially be None because the constructed cache is not associated
    // with any pipeline yet. This will happen later by way of `create_new_image_cache`.
    pipeline_id: Option<PipelineId>,

    /// Main struct to handle the cache of `WebRenderImageKey` and
    /// images that do not have a key yet.
    key_cache: KeyCache,
}

impl ImageCacheStore {
    /// Finishes loading the image by setting the WebRenderImageKey and calling `compete_load` or `complete_load_svg`.
    fn set_key_and_finish_load(&mut self, pending_image: PendingKey, image_key: WebRenderImageKey) {
        match pending_image {
            PendingKey::RasterImage((pending_id, mut raster_image)) => {
                set_webrender_image_key(&self.compositor_api, &mut raster_image, image_key);
                self.complete_load(pending_id, LoadResult::LoadedRasterImage(raster_image));
            },
            PendingKey::Svg((pending_id, mut raster_image, requested_size)) => {
                set_webrender_image_key(&self.compositor_api, &mut raster_image, image_key);
                self.complete_load_svg(raster_image, pending_id, requested_size);
            },
        }
    }

    /// If a key is available the image will be immediately loaded, otherwise it will load then the next batch of
    /// keys is received. Only call this if the image does not have a `LoadKey` yet.
    fn load_image_with_keycache(&mut self, pending_image: PendingKey) {
        if let Some(pipeline_id) = self.pipeline_id {
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
                        self.compositor_api.generate_image_key_async(pipeline_id);
                        self.key_cache.cache = KeyCacheState::PendingBatch
                    },
                },
            }
        } else {
            error!("No pipeline id for this image key cache.");
        }
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
                self.compositor_api
                    .generate_image_key_async(self.pipeline_id.unwrap());
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

        for (pipeline_id, sender) in listeners {
            let _ = sender.send(ImageCacheResponseMessage::VectorImageRasterizationComplete(
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
    ) -> Option<Result<(Image, ServoUrl), ()>> {
        self.completed_loads
            .get(&(url, origin, cors_setting))
            .map(
                |completed_load| match (&completed_load.image_response, placeholder) {
                    (ImageResponse::Loaded(image, url), _) => Ok((image.clone(), url.clone())),
                    (ImageResponse::PlaceholderLoaded(image, url), UsePlaceholder::Yes) => {
                        Ok((Image::Raster(image.clone()), url.clone()))
                    },
                    (ImageResponse::PlaceholderLoaded(_, _), UsePlaceholder::No) |
                    (ImageResponse::None, _) |
                    (ImageResponse::MetadataLoaded(_), _) => Err(()),
                },
            )
    }

    /// Handle a message from one of the decoder worker threads or from a sync
    /// decoding operation.
    fn handle_decoder(&mut self, msg: DecoderMsg) {
        let image = match msg.image {
            None => LoadResult::None,
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

pub struct ImageCacheImpl {
    store: Arc<Mutex<ImageCacheStore>>,

    /// Thread pool for image decoding
    thread_pool: Arc<CoreResourceThreadPool>,
}

impl ImageCache for ImageCacheImpl {
    fn new(compositor_api: CrossProcessCompositorApi, rippy_data: Vec<u8>) -> ImageCacheImpl {
        debug!("New image cache");

        // Uses an estimate of the system cpus to decode images
        // See https://doc.rust-lang.org/stable/std/thread/fn.available_parallelism.html
        // If no information can be obtained about the system, uses 4 threads as a default
        let thread_count = thread::available_parallelism()
            .map(|i| i.get())
            .unwrap_or(pref!(threadpools_fallback_worker_num) as usize)
            .min(pref!(threadpools_image_cache_workers_max).max(1) as usize);

        ImageCacheImpl {
            store: Arc::new(Mutex::new(ImageCacheStore {
                pending_loads: AllPendingLoads::new(),
                completed_loads: HashMap::new(),
                vector_images: HashMap::new(),
                rasterized_vector_images: HashMap::new(),
                placeholder_image: get_placeholder_image(&compositor_api, &rippy_data),
                placeholder_url: ServoUrl::parse("chrome://resources/rippy.png").unwrap(),
                compositor_api: compositor_api.clone(),
                pipeline_id: None,
                key_cache: KeyCache::new(),
            })),
            thread_pool: Arc::new(CoreResourceThreadPool::new(
                thread_count,
                "ImageCache".to_string(),
            )),
        }
    }

    fn memory_report(&self, prefix: &str, ops: &mut MallocSizeOfOps) -> Report {
        let size = self.store.lock().unwrap().size_of(ops);
        Report {
            path: path![prefix, "image-cache"],
            kind: ReportKind::ExplicitSystemHeapSize,
            size,
        }
    }

    fn get_image(
        &self,
        url: ServoUrl,
        origin: ImmutableOrigin,
        cors_setting: Option<CorsSettings>,
    ) -> Option<Image> {
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
            // Note: this happens if we are pending a batch of image keys.
            _ => ImageCacheResult::Pending(key),
        }
    }

    fn add_rasterization_complete_listener(
        &self,
        pipeline_id: PipelineId,
        image_id: PendingImageId,
        requested_size: DeviceIntSize,
        sender: IpcSender<ImageCacheResponseMessage>,
    ) {
        let completed = {
            let mut store = self.store.lock().unwrap();
            let key = (image_id, requested_size);
            if !store.vector_images.contains_key(&image_id) {
                warn!("Unknown image requested for rasterization for key {key:?}");
                return;
            };

            let Some(task) = store.rasterized_vector_images.get_mut(&key) else {
                warn!("Image rasterization task not found in the cache for key {key:?}");
                return;
            };

            match task.result {
                Some(_) => true,
                None => {
                    task.listeners.push((pipeline_id, sender.clone()));
                    false
                },
            }
        };

        if completed {
            let _ = sender.send(ImageCacheResponseMessage::VectorImageRasterizationComplete(
                RasterizationCompleteResponse {
                    pipeline_id,
                    image_id,
                    requested_size,
                },
            ));
        }
    }

    fn rasterize_vector_image(
        &self,
        image_id: PendingImageId,
        requested_size: DeviceIntSize,
    ) -> Option<RasterImage> {
        let mut store = self.store.lock().unwrap();
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
                let width = requested_size.width.try_into().unwrap_or(0);
                let height = requested_size.height.try_into().unwrap_or(0);
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
                bytes: IpcSharedMemory::from_bytes(&bytes),
                id: None,
                cors_status: vector_image.cors_status,
            };

            let mut store = store.lock().unwrap();
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
        let mut store = self.store.lock().unwrap();
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
                pending_load.content_type = metadata
                    .as_ref()
                    .and_then(|metadata| metadata.content_type.clone())
                    .map(|content_type| content_type.into_inner().into());
            },
            (FetchResponseMsg::ProcessResponseChunk(_, data), _) => {
                debug!("Got some data for {:?}", id);
                let mut store = self.store.lock().unwrap();
                let pending_load = store.pending_loads.get_by_key_mut(&id).unwrap();
                pending_load.bytes.extend_from_slice(&data);

                //jmr0 TODO: possibly move to another task?
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
                            let mut store = self.store.lock().unwrap();
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
                        self.thread_pool.spawn(move || {
                            let msg = decode_bytes_sync(key, &bytes, cors_status, content_type);
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

    fn create_new_image_cache(
        &self,
        pipeline_id: Option<PipelineId>,
        compositor_api: CrossProcessCompositorApi,
    ) -> Arc<dyn ImageCache> {
        let store = self.store.lock().unwrap();
        let placeholder_image = store.placeholder_image.clone();
        let placeholder_url = store.placeholder_url.clone();
        Arc::new(ImageCacheImpl {
            store: Arc::new(Mutex::new(ImageCacheStore {
                pending_loads: AllPendingLoads::new(),
                completed_loads: HashMap::new(),
                placeholder_image,
                placeholder_url,
                compositor_api,
                vector_images: HashMap::new(),
                rasterized_vector_images: HashMap::new(),
                key_cache: KeyCache::new(),
                pipeline_id,
            })),
            thread_pool: self.thread_pool.clone(),
        })
    }

    fn fill_key_cache_with_batch_of_keys(&self, image_keys: Vec<WebRenderImageKey>) {
        let mut store = self.store.lock().unwrap();
        store.insert_keys_and_load_images(image_keys);
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
        self.compositor_api.update_images(image_updates);
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
