/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use servo_base::id::{TEST_PIPELINE_ID, TEST_WEBVIEW_ID};
use webrender_api::IdNamespace;

use super::*;

fn size(width: u32) -> ImageMetadata {
    ImageMetadata {
        width,
        height: width,
    }
}

fn entry() -> DemandDrivenRasterEntry {
    DemandDrivenRasterEntry {
        encoded_image_source: Arc::new(EncodedImage::new(
            PendingImageId(0),
            size(2000),
            CorsStatus::Safe,
            None,
            Arc::new(vec![]),
        )),
        current_decoded_scaled_image: None,
        target_image_metadata: None,
        current_counter: RasterDecodeRequestCounter::default(),
        completed: None,
        pending: None,
        decoding: None,
        demand_epoch: RasterDecodeRequestCounter::default(),
        failed_target: None,
    }
}

fn image(width: u32) -> RasterImage {
    let mut image = load_from_memory(include_bytes!("tests/test.jpeg"), CorsStatus::Safe).unwrap();
    // These tests exercise scheduling and key installation, not pixel resampling.
    image.decoded_resolution = size(width);
    image
}

fn store(entry: DemandDrivenRasterEntry) -> ImageCacheStore {
    ImageCacheStore {
        pending_loads: AllPendingLoads::new(),
        completed_loads: HashMap::new(),
        encoded_raster_images: [(PendingImageId(0), entry)].into_iter().collect(),
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
    }
}

#[test]
fn continuous_resize_preserves_running_decode() {
    let mut entry = entry();
    entry.update_target(Some(size(300)));
    let counter = entry.current_counter;
    let epoch = entry.demand_epoch;
    entry.decoding = Some(size(300));
    for width in (150..=360).chain((150..=360).rev()).cycle().take(2000) {
        entry.update_target(Some(size(width)));
        assert_eq!(entry.target_image_metadata, Some(size(300)));
        assert_eq!(entry.current_counter, counter);
        assert!(entry.accepts_result(epoch, size(300)));
    }
}

#[test]
fn running_decode_uses_existing_hysteresis_boundaries() {
    let mut entry = entry();
    entry.decoding = Some(size(300));
    assert_eq!(entry.decode_target(size(150)), size(300));
    assert_eq!(entry.decode_target(size(149)), size(149));
    assert_eq!(entry.decode_target(size(360)), size(300));
    assert_eq!(entry.decode_target(size(361)), size(600));
    assert_eq!(entry.decode_target(size(1900)), size(1900));
}

#[test]
fn shrinking_decode_takes_precedence_over_installed_pixels() {
    let mut entry = entry();
    entry.current_decoded_scaled_image = Some(image(1000));
    entry.decoding = Some(size(400));
    entry.update_target(Some(size(401)));
    assert_eq!(entry.target_image_metadata, Some(size(400)));
    assert!(entry.accepts_result(entry.demand_epoch, size(400)));
}

#[test]
fn first_result_paints_after_large_demand_change_then_refines() {
    let mut entry = entry();
    entry.update_target(Some(size(300)));
    let epoch = entry.demand_epoch;
    entry.decoding = Some(size(300));
    // Even changes outside the band must not prevent the first paint.
    for width in [1000, 100, 1800, 900, 1000] {
        entry.update_target(Some(size(width)));
    }
    let counter = entry.current_counter;
    let mut store = store(entry);
    let (sender, receiver) = crossbeam_channel::unbounded();
    store.raster_decode_callback = Some(Box::new(move |message| sender.send(message).unwrap()));
    let key = WebRenderImageKey::new(IdNamespace(42), 0);
    store.key_cache.cache = KeyCacheState::Ready(vec![key]);
    store.finish_demand_decode(PendingImageId(0), 0, epoch, size(300), Some(image(300)));
    let entry = store.encoded_raster_images.get(&PendingImageId(0)).unwrap();
    assert_eq!(
        entry.current_decoded_scaled_image.as_ref().unwrap().id,
        Some(key)
    );
    assert_eq!(entry.target_image_metadata, Some(size(1000)));
    assert_ne!(entry.completed, Some(counter));
    assert!(matches!(receiver.try_recv().unwrap(),
        ImageCacheResponseMessage::RasterDecodeReady(_, PendingImageId(0), generation)
        if generation == counter));

    store.finish_demand_decode(PendingImageId(0), 0, epoch, size(1000), Some(image(1000)));
    let entry = store.encoded_raster_images.get(&PendingImageId(0)).unwrap();
    assert_eq!(
        entry
            .current_decoded_scaled_image
            .as_ref()
            .unwrap()
            .decoded_resolution,
        size(1000)
    );
    assert_eq!(entry.completed, Some(counter));
}

#[test]
fn first_result_survives_resizing_while_waiting_for_key() {
    let mut entry = entry();
    entry.update_target(Some(size(300)));
    entry.decoding = Some(size(300));
    let epoch = entry.demand_epoch;
    let mut store = store(entry);
    store.key_cache.cache = KeyCacheState::PendingBatch;
    store.finish_demand_decode(PendingImageId(0), 0, epoch, size(300), Some(image(300)));
    for width in [301, 350, 1000, 1001] {
        store
            .encoded_raster_images
            .get_mut(&PendingImageId(0))
            .unwrap()
            .update_target(Some(size(width)));
        store.prune_stale_demand_decode_pending_keys();
        assert_eq!(store.key_cache.images_pending_keys.len(), 1);
    }
    store.key_cache.cache = KeyCacheState::Processing;
    store.insert_keys_and_load_images(vec![WebRenderImageKey::new(IdNamespace(42), 0)]);
    assert!(
        store.encoded_raster_images[&PendingImageId(0)]
            .current_decoded_scaled_image
            .is_some()
    );
}

#[test]
fn withdrawn_demand_rejects_old_results_even_after_reappearing() {
    let mut entry = entry();
    entry.update_target(Some(size(300)));
    entry.decoding = Some(size(300));
    let epoch = entry.demand_epoch;
    entry.update_target(None);
    assert!(!entry.accepts_result(epoch, size(300)));
    entry.update_target(Some(size(301)));
    assert!(!entry.accepts_result(epoch, size(300)));
    assert!(entry.accepts_result(entry.demand_epoch, size(300)));
    let mut store = store(entry);
    store.finish_demand_decode(PendingImageId(0), 0, epoch, size(300), Some(image(300)));
    assert!(store.key_cache.images_pending_keys.is_empty());
    assert!(
        store.encoded_raster_images[&PendingImageId(0)]
            .current_decoded_scaled_image
            .is_none()
    );
}

#[test]
fn withdrawing_demand_prunes_key_waiting_result() {
    let mut entry = entry();
    entry.update_target(Some(size(300)));
    entry.decoding = Some(size(300));
    let epoch = entry.demand_epoch;
    let mut store = store(entry);
    store.key_cache.cache = KeyCacheState::PendingBatch;
    store.finish_demand_decode(PendingImageId(0), 0, epoch, size(300), Some(image(300)));
    assert_eq!(store.key_cache.images_pending_keys.len(), 1);
    store
        .encoded_raster_images
        .get_mut(&PendingImageId(0))
        .unwrap()
        .update_target(None);
    store.prune_stale_demand_decode_pending_keys();
    assert!(store.key_cache.images_pending_keys.is_empty());
    assert!(
        store.encoded_raster_images[&PendingImageId(0)]
            .pending
            .is_none()
    );
}

#[test]
fn unsuitable_refinement_does_not_replace_existing_pixels() {
    let mut entry = entry();
    entry.current_decoded_scaled_image = Some(image(300));
    entry.target_image_metadata = Some(size(1000));
    entry.pending = Some(size(400));
    let epoch = entry.demand_epoch;
    assert!(!entry.accepts_result(epoch, size(400)));
    let mut store = store(entry);
    store
        .key_cache
        .images_pending_keys
        .push_back(PendingKey::DemandDecodedRaster((
            PendingImageId(0),
            epoch,
            image(400),
        )));
    store.prune_stale_demand_decode_pending_keys();
    assert!(store.key_cache.images_pending_keys.is_empty());
    assert!(
        store.encoded_raster_images[&PendingImageId(0)]
            .pending
            .is_none()
    );
    store.clear();
    assert!(store.encoded_raster_images.is_empty());
    store.finish_demand_decode(PendingImageId(0), 0, epoch, size(400), Some(image(400)));
    assert!(store.encoded_raster_images.is_empty());
    assert!(store.key_cache.images_pending_keys.is_empty());
}

#[test]
fn cleared_cache_rejects_old_decode_with_reused_load_id() {
    let mut old_entry = entry();
    old_entry.update_target(Some(size(300)));
    let epoch = old_entry.demand_epoch;
    let mut store = store(old_entry);
    store.clear();
    let mut new_entry = entry();
    new_entry.update_target(Some(size(300)));
    new_entry.decoding = Some(size(300));
    store
        .encoded_raster_images
        .insert(PendingImageId(0), new_entry);
    store.finish_demand_decode(PendingImageId(0), 0, epoch, size(300), Some(image(300)));
    let entry = &store.encoded_raster_images[&PendingImageId(0)];
    assert_eq!(entry.decoding, Some(size(300)));
    assert!(entry.current_decoded_scaled_image.is_none());
    assert!(store.key_cache.images_pending_keys.is_empty());
}
