/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Controlled demand-decode fixture for integration tests. No decoder threads
//! run: tests supply completion pixels and explicitly deliver WebRender keys.

use servo_base::id::{TEST_PIPELINE_ID, TEST_WEBVIEW_ID};
use webrender_api::IdNamespace;

use super::*;

/// Captures the identity and target of a simulated running decode.
#[derive(Clone, Copy)]
pub struct DecodeJob {
    epoch: RasterDecodeRequestCounter,
    cache_clear_count: u64,
    target: ImageMetadata,
}

/// Read-only observations, without exposing mutable cache internals.
pub struct DecodeState {
    pub target: Option<ImageMetadata>,
    pub counter: RasterDecodeRequestCounter,
    pub decoding: Option<ImageMetadata>,
    pub pending: Option<ImageMetadata>,
    pub installed: Option<ImageMetadata>,
    pub key: Option<WebRenderImageKey>,
    pub completed: bool,
    pub pending_keys: usize,
}

/// A single-source cache with manually controlled decode completion.
pub struct DemandDecodeTestCache {
    store: ImageCacheStore,
    source: Arc<EncodedImage>,
    image: RasterImage,
}

impl DemandDecodeTestCache {
    /// The supplied pixels are only a template for installation; tests choose
    /// synthetic resolutions to exercise state transitions, not resampling.
    pub fn new(natural: ImageMetadata, image: RasterImage) -> Self {
        let source = Arc::new(EncodedImage::new(
            PendingImageId(0),
            natural,
            CorsStatus::Safe,
            None,
            Arc::new(vec![]),
        ));
        let mut fixture = Self {
            store: ImageCacheStore {
                pending_loads: AllPendingLoads::new(),
                completed_loads: HashMap::new(),
                encoded_raster_images: FxHashMap::default(),
                cache_clear_count: 0,
                raster_decode_callback: None,
                vector_images: FxHashMap::default(),
                rasterized_vector_images: FxHashMap::default(),
                broken_image_icon_image: OnceCell::new(),
                paint_api: CrossProcessPaintApi::dummy_with_callback(None),
                pipeline_id: TEST_PIPELINE_ID,
                webview_id: TEST_WEBVIEW_ID,
                key_cache: KeyCache::new(),
                svg_rasterization_task_store: SvgRasterizationTaskStore::default(),
            },
            source,
            image,
        };
        fixture.insert_source();
        fixture.provide_key();
        fixture
    }

    /// Reinsert the same load id, including after a cache clear.
    pub fn insert_source(&mut self) {
        self.store.encoded_raster_images.insert(
            PendingImageId(0),
            DemandDrivenRasterEntry {
                encoded_image_source: self.source.clone(),
                current_decoded_scaled_image: None,
                target_image_metadata: None,
                current_counter: RasterDecodeRequestCounter::default(),
                completed: None,
                pending: None,
                decoding: None,
                demand_epoch: RasterDecodeRequestCounter::default(),
                failed_target: None,
            },
        );
    }

    fn entry(&self) -> &DemandDrivenRasterEntry {
        &self.store.encoded_raster_images[&PendingImageId(0)]
    }

    fn entry_mut(&mut self) -> &mut DemandDrivenRasterEntry {
        self.store
            .encoded_raster_images
            .get_mut(&PendingImageId(0))
            .unwrap()
    }

    fn pixels(&self, size: ImageMetadata) -> RasterImage {
        let mut image = self.image.clone();
        image.decoded_resolution = size;
        image
    }

    pub fn update_demand(&mut self, required: Option<ImageMetadata>) {
        self.entry_mut().update_target(required);
        self.store.prune_stale_demand_decode_pending_keys();
    }

    pub fn select_target(&self, required: ImageMetadata) -> ImageMetadata {
        self.entry().decode_target(required)
    }

    pub fn begin_decode(&mut self, target: ImageMetadata) -> DecodeJob {
        assert!(self.entry().decoding.is_none());
        self.entry_mut().decoding = Some(target);
        DecodeJob {
            epoch: self.entry().demand_epoch,
            cache_clear_count: self.store.cache_clear_count,
            target,
        }
    }

    pub fn accepts(&self, job: DecodeJob) -> bool {
        self.entry().accepts_result(job.epoch, job.target)
    }

    pub fn finish_decode(&mut self, job: DecodeJob) {
        let image = self.pixels(job.target);
        self.store.finish_demand_decode(
            PendingImageId(0),
            job.cache_clear_count,
            job.epoch,
            job.target,
            Some(image),
        );
    }

    /// Seed installed pixels without a key for tests of resolution selection.
    pub fn set_installed(&mut self, size: ImageMetadata) {
        self.entry_mut().current_decoded_scaled_image = Some(self.pixels(size));
    }

    /// Seed a completed result waiting for a key, before checking its validity.
    pub fn queue_result(&mut self, job: DecodeJob) {
        self.entry_mut().pending = Some(job.target);
        let image = self.pixels(job.target);
        self.store
            .key_cache
            .images_pending_keys
            .push_back(PendingKey::DemandDecodedRaster((
                PendingImageId(0),
                job.epoch,
                image,
            )));
    }

    pub fn wait_for_key(&mut self) {
        self.store.key_cache.cache = KeyCacheState::PendingBatch;
    }

    pub fn provide_key(&mut self) {
        self.store.key_cache.cache = KeyCacheState::Processing;
        self.store
            .insert_keys_and_load_images(vec![WebRenderImageKey::new(IdNamespace(42), 0)]);
    }

    pub fn set_callback(&mut self, callback: ImageCacheResponseCallback) {
        self.store.raster_decode_callback = Some(callback);
    }

    pub fn clear(&mut self) {
        self.store.clear();
    }

    /// None means the source entry no longer exists (for example after clear).
    pub fn state(&self) -> Option<DecodeState> {
        let entry = self.store.encoded_raster_images.get(&PendingImageId(0))?;
        Some(DecodeState {
            target: entry.target_image_metadata,
            counter: entry.current_counter,
            decoding: entry.decoding,
            pending: entry.pending,
            installed: entry
                .current_decoded_scaled_image
                .as_ref()
                .map(|image| image.decoded_resolution),
            key: entry
                .current_decoded_scaled_image
                .as_ref()
                .and_then(|image| image.id),
            completed: entry.completed == Some(entry.current_counter),
            pending_keys: self.store.key_cache.images_pending_keys.len(),
        })
    }

    pub fn pending_keys(&self) -> usize {
        self.store.key_cache.images_pending_keys.len()
    }
}
