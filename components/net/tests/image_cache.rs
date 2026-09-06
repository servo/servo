/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender, unbounded};
use malloc_size_of_derive::MallocSizeOf;
use net::image_cache::ImageCacheFactoryImpl;
use net_traits::image_cache::{
    FontResolver, ImageCache, ImageCacheFactory, ImageCacheResponseMessage, ImageCacheResult,
    ImageLoadListener, ImageOrMetadataAvailable, ImageResponse, PendingImageId,
    PendingImageResponse,
};
use net_traits::request::RequestId;
use net_traits::{
    FetchMetadata, FetchResponseMsg, FilteredMetadata, Metadata, NetworkError, ResourceFetchTiming,
    ResourceTimingType,
};
use paint_api::{CrossProcessPaintApi, PaintMessage};
// For dummy Font Resolver
use resvg::usvg::{Font, fontdb};
use servo_base::id::{PipelineId, TEST_PIPELINE_ID, TEST_WEBVIEW_ID};
use servo_url::ServoUrl;
use uuid::Uuid;
use webrender_api::ImageKey;

use crate::mock_origin;

#[derive(MallocSizeOf)]
struct DummyFontResolver;

impl FontResolver for DummyFontResolver {
    fn resolve(&self, _: &Font, _: &mut Arc<fontdb::Database>) -> Option<fontdb::ID> {
        None
    }

    fn resolve_fallback(
        &self,
        _: char,
        _: &[fontdb::ID],
        _: &mut Arc<fontdb::Database>,
    ) -> Option<fontdb::ID> {
        None
    }
}

fn create_test_image_cache() -> (Arc<dyn ImageCache>, Receiver<PipelineId>) {
    let (sender, receiver) = unbounded();
    let paint_api = CrossProcessPaintApi::dummy_with_callback(Some(Box::new(move |msg| {
        if let PaintMessage::GenerateImageKeysForPipeline(_, pipeline_id) = msg {
            let _ = sender.send(pipeline_id);
        }
    })));
    let dummy_resolver = Arc::new(DummyFontResolver);

    let factory = ImageCacheFactoryImpl::new(vec![]);
    let cache = factory.create(
        TEST_WEBVIEW_ID,
        TEST_PIPELINE_ID,
        &paint_api,
        dummy_resolver.clone(),
    );
    (cache, receiver)
}

fn handle_pending_key_requests(cache: &Arc<dyn ImageCache>, receiver: &Receiver<PipelineId>) {
    while let Ok(_pipeline_id) = receiver.try_recv() {
        let keys: Vec<_> = (0..10)
            .map(|i| ImageKey::new(webrender_api::IdNamespace(42), i as u32))
            .collect();
        cache.dispatch_fill_key_cache_with_batch_of_keys(keys);
    }
}

fn create_test_listener(id: PendingImageId, sender: Sender<ImageResponse>) -> ImageLoadListener {
    let callback = Box::new(move |msg: ImageCacheResponseMessage| {
        if let ImageCacheResponseMessage::NotifyPendingImageLoadStatus(PendingImageResponse {
            response,
            ..
        }) = msg
        {
            let _ = sender.send(response);
        }
    });
    ImageLoadListener::new(callback, TEST_PIPELINE_ID, id)
}

fn jpeg_image_bytes() -> Vec<u8> {
    include_bytes!("test.jpeg").to_vec()
}

fn svg_image_bytes() -> Vec<u8> {
    br#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
    <circle cx="50" cy="50" r="40" fill="red"/>
</svg>"#
        .to_vec()
}

fn create_test_metadata(mime_type: Option<mime::Mime>) -> FetchMetadata {
    let url = ServoUrl::parse("http://example.com").unwrap();
    let mut metadata = Metadata::default(url);
    metadata.set_content_type(mime_type.as_ref());
    FetchMetadata::Filtered {
        filtered: FilteredMetadata::Opaque,
        unsafe_: metadata,
    }
}

fn create_request_id() -> RequestId {
    RequestId(Uuid::nil())
}

fn create_timing() -> ResourceFetchTiming {
    ResourceFetchTiming::new(ResourceTimingType::Resource)
}

#[test]
fn test_get_cached_image_status_before_request() {
    let (cache, _key_receiver) = create_test_image_cache();
    let url = ServoUrl::parse("http://example.com/image.png").unwrap();
    let origin = mock_origin();

    let result = cache.get_cached_image_status(url, origin, None);

    match result {
        ImageCacheResult::ReadyForRequest(id) => {
            assert!(id.0 > 0);
        },
        _ => panic!("Expected ReadyForRequest"),
    }
}

#[test]
fn test_get_cached_image_status_no_response_data() {
    let (cache, _key_receiver) = create_test_image_cache();
    let url = ServoUrl::parse("http://example.com/image.png").unwrap();
    let origin = mock_origin();

    let id = match cache.get_cached_image_status(url.clone(), origin.clone(), None) {
        ImageCacheResult::ReadyForRequest(id) => id,
        _ => panic!("Expected ReadyForRequest"),
    };

    let result = cache.get_cached_image_status(url, origin, None);

    match result {
        ImageCacheResult::Pending(pending_id) => {
            assert_eq!(id, pending_id);
        },
        _ => panic!("Expected Pending after initial request"),
    }
}

#[test]
fn test_notify_pending_response_with_headers() {
    let (cache, _key_receiver) = create_test_image_cache();
    let url = ServoUrl::parse("http://example.com/image.png").unwrap();
    let origin = mock_origin();

    let id = match cache.get_cached_image_status(url.clone(), origin.clone(), None) {
        ImageCacheResult::ReadyForRequest(id) => id,
        _ => panic!("Expected ReadyForRequest"),
    };

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponse(create_request_id(), Ok(create_test_metadata(None))),
    );

    let result = cache.get_cached_image_status(url, origin, None);

    match result {
        ImageCacheResult::Pending(pending_id) => {
            assert_eq!(id, pending_id);
        },
        _ => panic!("Expected Pending after headers received"),
    }
}

#[test]
fn test_notify_pending_response_with_partial_chunk() {
    let (cache, _key_receiver) = create_test_image_cache();
    let url = ServoUrl::parse("http://example.com/image.png").unwrap();
    let origin = mock_origin();

    let id = match cache.get_cached_image_status(url.clone(), origin.clone(), None) {
        ImageCacheResult::ReadyForRequest(id) => id,
        _ => panic!("Expected ReadyForRequest"),
    };

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponse(create_request_id(), Ok(create_test_metadata(None))),
    );

    let small_chunk = vec![0u8; 10];
    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), small_chunk.into()),
    );

    let result = cache.get_cached_image_status(url, origin, None);

    match result {
        ImageCacheResult::Pending(pending_id) => {
            assert_eq!(id, pending_id);
        },
        _ => panic!("Expected Pending with insufficient data"),
    }
}

#[test]
fn test_notify_pending_response_with_metadata_chunk() {
    let (cache, _key_receiver) = create_test_image_cache();
    let url = ServoUrl::parse("http://example.com/test.jpeg").unwrap();
    let origin = mock_origin();

    let id = match cache.get_cached_image_status(url.clone(), origin.clone(), None) {
        ImageCacheResult::ReadyForRequest(id) => id,
        _ => panic!("Expected ReadyForRequest"),
    };

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponse(create_request_id(), Ok(create_test_metadata(None))),
    );

    let jpeg_bytes = jpeg_image_bytes();
    let metadata_chunk = jpeg_bytes[..200.min(jpeg_bytes.len())].to_vec();
    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), metadata_chunk.into()),
    );

    let result = cache.get_cached_image_status(url, origin, None);

    match result {
        ImageCacheResult::Available(ImageOrMetadataAvailable::MetadataAvailable(metadata, _)) => {
            assert!(metadata.width > 0);
            assert!(metadata.height > 0);
        },
        ImageCacheResult::Pending(_) => {},
        _ => panic!("Expected MetadataAvailable or Pending"),
    }
}

#[test]
fn test_notify_pending_response_complete() {
    let (cache, key_receiver) = create_test_image_cache();
    let url = ServoUrl::parse("http://example.com/test.jpeg").unwrap();
    let origin = mock_origin();

    let id = match cache.get_cached_image_status(url.clone(), origin.clone(), None) {
        ImageCacheResult::ReadyForRequest(id) => id,
        _ => panic!("Expected ReadyForRequest"),
    };

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponse(create_request_id(), Ok(create_test_metadata(None))),
    );

    let jpeg_bytes = jpeg_image_bytes();
    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), jpeg_bytes.into()),
    );

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseEOF(create_request_id(), Ok(()), create_timing()),
    );

    loop {
        handle_pending_key_requests(&cache, &key_receiver);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let result = cache.get_cached_image_status(url.clone(), origin.clone(), None);
        if matches!(
            result,
            ImageCacheResult::Available(ImageOrMetadataAvailable::ImageAvailable { .. })
        ) {
            break;
        }
    }

    let image = cache.get_image(url, origin, None);
    assert!(image.is_some());
    assert!(image.unwrap().as_raster_image().is_some());
}

#[test]
fn test_notify_pending_response_network_error() {
    let (cache, _key_receiver) = create_test_image_cache();
    let url = ServoUrl::parse("http://example.com/image.png").unwrap();
    let origin = mock_origin();

    let id = match cache.get_cached_image_status(url.clone(), origin.clone(), None) {
        ImageCacheResult::ReadyForRequest(id) => id,
        _ => panic!("Expected ReadyForRequest"),
    };

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponse(create_request_id(), Err(NetworkError::InvalidMethod)),
    );

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseEOF(
            create_request_id(),
            Err(NetworkError::InvalidMethod),
            create_timing(),
        ),
    );

    let result = cache.get_cached_image_status(url, origin, None);
    assert!(matches!(result, ImageCacheResult::FailedToLoadOrDecode));
}

#[test]
fn test_image_listener_on_complete_response() {
    let (cache, key_receiver) = create_test_image_cache();
    let url = ServoUrl::parse("http://example.com/test.jpeg").unwrap();
    let origin = mock_origin();

    let id = match cache.get_cached_image_status(url.clone(), origin.clone(), None) {
        ImageCacheResult::ReadyForRequest(id) => id,
        _ => panic!("Expected ReadyForRequest"),
    };

    let (sender, receiver) = unbounded();
    let listener = create_test_listener(id, sender);

    cache.add_listener(listener);

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponse(create_request_id(), Ok(create_test_metadata(None))),
    );

    let jpeg_bytes = jpeg_image_bytes();
    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), jpeg_bytes.into()),
    );

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseEOF(create_request_id(), Ok(()), create_timing()),
    );

    loop {
        handle_pending_key_requests(&cache, &key_receiver);
        match receiver.recv_timeout(Duration::from_millis(10)) {
            Ok(response) => match response {
                ImageResponse::Loaded(..) | ImageResponse::MetadataLoaded(..) => break,
                _ => {},
            },
            Err(_) => {},
        }
    }
}

#[test]
fn test_image_listener_on_network_error() {
    let (cache, _key_receiver) = create_test_image_cache();
    let url = ServoUrl::parse("http://example.com/image.png").unwrap();
    let origin = mock_origin();

    let id = match cache.get_cached_image_status(url.clone(), origin.clone(), None) {
        ImageCacheResult::ReadyForRequest(id) => id,
        _ => panic!("Expected ReadyForRequest"),
    };

    let (sender, receiver) = unbounded();
    let listener = create_test_listener(id, sender);

    cache.add_listener(listener);

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponse(create_request_id(), Err(NetworkError::InvalidMethod)),
    );

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseEOF(
            create_request_id(),
            Err(NetworkError::InvalidMethod),
            create_timing(),
        ),
    );

    match receiver.recv_timeout(std::time::Duration::from_millis(100)) {
        Ok(ImageResponse::FailedToLoadOrDecode) => {},
        Ok(_) => panic!("Expected FailedToLoadOrDecode response"),
        Err(_) => panic!("Expected to receive error response"),
    }
}

#[test]
fn test_image_listener_on_metadata_available() {
    let (cache, _key_receiver) = create_test_image_cache();
    let url = ServoUrl::parse("http://example.com/test.jpeg").unwrap();
    let origin = mock_origin();

    let id = match cache.get_cached_image_status(url.clone(), origin.clone(), None) {
        ImageCacheResult::ReadyForRequest(id) => id,
        _ => panic!("Expected ReadyForRequest"),
    };

    let (sender, receiver) = unbounded();
    let listener = create_test_listener(id, sender);

    cache.add_listener(listener);

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponse(create_request_id(), Ok(create_test_metadata(None))),
    );

    let jpeg_bytes = jpeg_image_bytes();
    let metadata_chunk = jpeg_bytes[..200.min(jpeg_bytes.len())].to_vec();
    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), metadata_chunk.into()),
    );

    match receiver.recv_timeout(std::time::Duration::from_millis(100)) {
        Ok(ImageResponse::MetadataLoaded(metadata)) => {
            assert!(metadata.width > 0);
            assert!(metadata.height > 0);
        },
        Ok(_) => {},
        Err(_) => {},
    }
}

#[test]
fn test_get_image_returns_none_when_not_loaded() {
    let (cache, _key_receiver) = create_test_image_cache();
    let url = ServoUrl::parse("http://example.com/image.png").unwrap();
    let origin = mock_origin();

    let image = cache.get_image(url, origin, None);
    assert!(image.is_none());
}

#[test]
fn test_multiple_listeners_same_image() {
    let (cache, key_receiver) = create_test_image_cache();
    let url = ServoUrl::parse("http://example.com/test.jpeg").unwrap();
    let origin = mock_origin();

    let id = match cache.get_cached_image_status(url.clone(), origin.clone(), None) {
        ImageCacheResult::ReadyForRequest(id) => id,
        _ => panic!("Expected ReadyForRequest"),
    };

    let (sender1, receiver1) = unbounded();
    let listener1 = create_test_listener(id, sender1);

    let (sender2, receiver2) = unbounded();
    let listener2 = create_test_listener(id, sender2);

    cache.add_listener(listener1);
    cache.add_listener(listener2);

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponse(create_request_id(), Ok(create_test_metadata(None))),
    );

    let jpeg_bytes = jpeg_image_bytes();
    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), jpeg_bytes.into()),
    );

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseEOF(create_request_id(), Ok(()), create_timing()),
    );

    loop {
        handle_pending_key_requests(&cache, &key_receiver);
        match receiver1.recv_timeout(Duration::from_millis(10)) {
            Ok(ImageResponse::Loaded(..) | ImageResponse::MetadataLoaded(..)) => break,
            Ok(_) => {},
            Err(_) => {},
        }
    }

    loop {
        handle_pending_key_requests(&cache, &key_receiver);
        match receiver2.recv_timeout(Duration::from_millis(10)) {
            Ok(ImageResponse::Loaded(..) | ImageResponse::MetadataLoaded(..)) => break,
            Ok(_) => {},
            Err(_) => {},
        }
    }
}

#[test]
fn test_cached_image_reuse() {
    let (cache, key_receiver) = create_test_image_cache();
    let url = ServoUrl::parse("http://example.com/test.jpeg").unwrap();
    let origin = mock_origin();

    let id = match cache.get_cached_image_status(url.clone(), origin.clone(), None) {
        ImageCacheResult::ReadyForRequest(id) => id,
        _ => panic!("Expected ReadyForRequest"),
    };

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponse(create_request_id(), Ok(create_test_metadata(None))),
    );

    let jpeg_bytes = jpeg_image_bytes();
    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), jpeg_bytes.into()),
    );

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseEOF(create_request_id(), Ok(()), create_timing()),
    );

    loop {
        handle_pending_key_requests(&cache, &key_receiver);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let result = cache.get_cached_image_status(url.clone(), origin.clone(), None);
        if matches!(result, ImageCacheResult::Available(_)) {
            break;
        }
    }
}

#[test]
fn test_svg_rasterization() {
    let (cache, key_receiver) = create_test_image_cache();
    let url = ServoUrl::parse("http://example.com/image.svg").unwrap();
    let origin = mock_origin();

    let id = match cache.get_cached_image_status(url.clone(), origin.clone(), None) {
        ImageCacheResult::ReadyForRequest(id) => id,
        _ => panic!("Expected ReadyForRequest"),
    };

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponse(
            create_request_id(),
            Ok(create_test_metadata(Some(mime::IMAGE_SVG))),
        ),
    );

    let svg_bytes = svg_image_bytes();
    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), svg_bytes.into()),
    );

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseEOF(create_request_id(), Ok(()), create_timing()),
    );

    let vec_img = loop {
        handle_pending_key_requests(&cache, &key_receiver);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let result = cache.get_cached_image_status(url.clone(), origin.clone(), None);
        let ImageCacheResult::Available(ImageOrMetadataAvailable::ImageAvailable { image, .. }) =
            result
        else {
            continue;
        };

        let net_traits::image_cache::Image::Vector(vec_img) = image else {
            panic!("Expected vector image");
        };
        break vec_img;
    };

    let size = webrender_api::units::DeviceIntSize::new(100, 100);
    cache.rasterize_vector_image(vec_img.id, size, None);
}

#[test]
fn test_rasterization_listener() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let (cache, key_receiver) = create_test_image_cache();
    let url = ServoUrl::parse("http://example.com/image.svg").unwrap();
    let origin = mock_origin();

    let id = match cache.get_cached_image_status(url.clone(), origin.clone(), None) {
        ImageCacheResult::ReadyForRequest(id) => id,
        _ => panic!("Expected ReadyForRequest"),
    };

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponse(
            create_request_id(),
            Ok(create_test_metadata(Some(mime::IMAGE_SVG))),
        ),
    );

    let svg_bytes = svg_image_bytes();
    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), svg_bytes.into()),
    );

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseEOF(create_request_id(), Ok(()), create_timing()),
    );

    let vec_img = loop {
        handle_pending_key_requests(&cache, &key_receiver);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let result = cache.get_cached_image_status(url.clone(), origin.clone(), None);
        let ImageCacheResult::Available(ImageOrMetadataAvailable::ImageAvailable { image, .. }) =
            result
        else {
            continue;
        };

        let net_traits::image_cache::Image::Vector(vec_img) = image else {
            panic!("Expected vector image");
        };
        break vec_img;
    };

    let size = webrender_api::units::DeviceIntSize::new(100, 100);
    let notified = Arc::new(AtomicBool::new(false));
    let notified_clone = notified.clone();

    let callback = Box::new(move |msg: ImageCacheResponseMessage| {
        if let ImageCacheResponseMessage::VectorImageRasterizationComplete(_) = msg {
            notified_clone.store(true, Ordering::SeqCst);
        }
    });

    cache.rasterize_vector_image(vec_img.id, size, None);

    cache.add_rasterization_complete_listener(TEST_PIPELINE_ID, vec_img.id, size, callback);

    loop {
        handle_pending_key_requests(&cache, &key_receiver);
        if notified.load(Ordering::SeqCst) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

#[test]
/// Test if multiple rasterization requests rasterize the svg only once.
fn test_svg_rasterization_do_not_double_rasterize() {
    let (cache, _key_receiver) = create_test_image_cache();
    let url = ServoUrl::parse("http://example.com/image.svg").unwrap();
    let origin = mock_origin();

    let id = match cache.get_cached_image_status(url.clone(), origin.clone(), None) {
        ImageCacheResult::ReadyForRequest(id) => id,
        _ => panic!("Expected ReadyForRequest"),
    };

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponse(
            create_request_id(),
            Ok(create_test_metadata(Some(mime::IMAGE_SVG))),
        ),
    );

    let svg_bytes = svg_image_bytes();
    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), svg_bytes.into()),
    );

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseEOF(create_request_id(), Ok(()), create_timing()),
    );

    let vec_img = loop {
        std::thread::sleep(std::time::Duration::from_millis(10));
        let result = cache.get_cached_image_status(url.clone(), origin.clone(), None);
        let ImageCacheResult::Available(ImageOrMetadataAvailable::ImageAvailable { image, .. }) =
            result
        else {
            continue;
        };

        let net_traits::image_cache::Image::Vector(vec_img) = image else {
            panic!("Expected vector image");
        };
        break vec_img;
    };

    let size = webrender_api::units::DeviceIntSize::new(100, 100);
    // Because we do not set image keys yet, the rasterization task will never finish, so we know the added tasks will stay in the queue.
    assert!(
        cache
            .rasterize_vector_image(vec_img.id, size, None)
            .is_none()
    );
    assert!(
        cache
            .rasterize_vector_image(vec_img.id, size, None)
            .is_none()
    );
    assert_eq!(cache.number_of_rasterize_tasks(), 1);
}

#[test]
fn test_svg_not_rasterize_zero_size() {
    let (cache, key_receiver) = create_test_image_cache();
    let url = ServoUrl::parse("http://example.com/image.svg").unwrap();
    let origin = mock_origin();

    let id = match cache.get_cached_image_status(url.clone(), origin.clone(), None) {
        ImageCacheResult::ReadyForRequest(id) => id,
        _ => panic!("Expected ReadyForRequest"),
    };

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponse(
            create_request_id(),
            Ok(create_test_metadata(Some(mime::IMAGE_SVG))),
        ),
    );

    let svg_bytes = svg_image_bytes();
    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), svg_bytes.into()),
    );

    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseEOF(create_request_id(), Ok(()), create_timing()),
    );

    let vec_img = loop {
        handle_pending_key_requests(&cache, &key_receiver);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let result = cache.get_cached_image_status(url.clone(), origin.clone(), None);
        let ImageCacheResult::Available(ImageOrMetadataAvailable::ImageAvailable { image, .. }) =
            result
        else {
            continue;
        };

        let net_traits::image_cache::Image::Vector(vec_img) = image else {
            panic!("Expected vector image");
        };
        break vec_img;
    };

    let size = webrender_api::units::DeviceIntSize::new(0, 100);
    assert!(
        cache
            .rasterize_vector_image(vec_img.id, size, None)
            .is_none()
    );
    let size = webrender_api::units::DeviceIntSize::new(100, 0);
    assert!(
        cache
            .rasterize_vector_image(vec_img.id, size, None)
            .is_none()
    );
    let size = webrender_api::units::DeviceIntSize::new(0, 0);
    assert!(
        cache
            .rasterize_vector_image(vec_img.id, size, None)
            .is_none()
    );
}

/// A generated 1000 x 500 BMP keeps the tests independent of external resources.
fn large_static_bmp() -> Vec<u8> {
    let width = 1000u32;
    let height = 500u32;
    let length = 54 + width * height * 3;
    let mut bytes = vec![0; length as usize];
    bytes[..2].copy_from_slice(b"BM");
    bytes[2..6].copy_from_slice(&length.to_le_bytes());
    bytes[10..14].copy_from_slice(&54u32.to_le_bytes());
    bytes[14..18].copy_from_slice(&40u32.to_le_bytes());
    bytes[18..22].copy_from_slice(&width.to_le_bytes());
    bytes[22..26].copy_from_slice(&height.to_le_bytes());
    bytes[26..28].copy_from_slice(&1u16.to_le_bytes());
    bytes[28..30].copy_from_slice(&24u16.to_le_bytes());
    for pixel in bytes[54..].chunks_exact_mut(3) {
        pixel.copy_from_slice(&[0, 0, 255]);
    }
    bytes
}

fn static_test_cache() -> (Arc<dyn ImageCache>, Receiver<PaintMessage>) {
    let (sender, receiver) = unbounded();
    let paint_api = CrossProcessPaintApi::dummy_with_callback(Some(Box::new(move |message| {
        let _ = sender.send(message);
    })));
    let cache = ImageCacheFactoryImpl::new(vec![]).create(
        TEST_WEBVIEW_ID,
        TEST_PIPELINE_ID,
        &paint_api,
        Arc::new(DummyFontResolver),
    );
    (cache, receiver)
}

fn load_static_test_image(
    cache: &Arc<dyn ImageCache>,
) -> (PendingImageId, net_traits::image_cache::Image) {
    let url = ServoUrl::parse("http://example.com/static.bmp").unwrap();
    let ImageCacheResult::ReadyForRequest(id) =
        cache.get_cached_image_status(url, mock_origin(), None)
    else {
        panic!("expected a new load");
    };
    let (sender, receiver) = unbounded();
    cache.add_listener(create_test_listener(id, sender));
    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponse(create_request_id(), Ok(create_test_metadata(None))),
    );
    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), large_static_bmp().into()),
    );
    cache.notify_pending_response(
        id,
        FetchResponseMsg::ProcessResponseEOF(create_request_id(), Ok(()), create_timing()),
    );
    loop {
        match receiver.recv_timeout(Duration::from_secs(10)).unwrap() {
            ImageResponse::MetadataLoaded(_) => {},
            ImageResponse::Loaded(image, _) => return (id, image),
            ImageResponse::FailedToLoadOrDecode => panic!("valid BMP failed to load"),
        }
    }
}

fn supply_static_test_keys(cache: &Arc<dyn ImageCache>) {
    cache.dispatch_fill_key_cache_with_batch_of_keys(
        (0..10)
            .map(|id| ImageKey::new(webrender_api::IdNamespace(123), id))
            .collect(),
    );
}

fn set_static_test_sizes(cache: &Arc<dyn ImageCache>, id: PendingImageId, widths: &[i32]) {
    cache.set_static_raster_demands(
        widths
            .iter()
            .map(|width| {
                (
                    id,
                    webrender_api::units::DeviceIntSize::new(*width, *width / 2),
                )
            })
            .collect(),
        Box::new(|_| {}),
    );
}

fn next_static_upload(receiver: &Receiver<PaintMessage>) -> (ImageKey, i32, i32, bool) {
    use paint_api::ImageUpdate;
    loop {
        if let PaintMessage::UpdateImages(_, updates) =
            receiver.recv_timeout(Duration::from_secs(10)).unwrap()
        {
            for update in updates {
                match update {
                    ImageUpdate::AddImage(key, descriptor, _, _) => {
                        return (key, descriptor.size.width, descriptor.size.height, true);
                    },
                    ImageUpdate::UpdateImage(key, descriptor, _, _) => {
                        return (key, descriptor.size.width, descriptor.size.height, false);
                    },
                    _ => panic!("unexpected image update"),
                }
            }
        }
    }
}

#[test]
fn static_display_waits_for_layout_and_preserves_source_pixels() {
    let (cache, receiver) = static_test_cache();
    let (id, image) = load_static_test_image(&cache);
    assert!(matches!(
        image,
        net_traits::image_cache::Image::StaticRaster(_)
    ));
    assert!(
        receiver.try_recv().is_err(),
        "loading alone must not request a WebRender key"
    );
    assert!(cache.static_raster_image_key(id).is_none());
    let original = image.as_raster_image().unwrap();
    assert_eq!(
        original.metadata,
        pixels::ImageMetadata {
            width: 1000,
            height: 500
        }
    );
    assert_eq!(original.bytes.len(), 1000 * 500 * 4);
    assert!(original.id.is_none());

    supply_static_test_keys(&cache);
    set_static_test_sizes(&cache, id, &[300]);
    let (key, width, height, added) = next_static_upload(&receiver);
    assert_eq!((width, height, added), (300, 150, true));
    assert_eq!(cache.static_raster_image_key(id), Some(key));
    assert_eq!(image.metadata(), original.metadata);
    cache.clear();
}

#[test]
fn static_display_shares_largest_use_and_applies_resize_thresholds() {
    let (cache, receiver) = static_test_cache();
    let (id, _) = load_static_test_image(&cache);
    supply_static_test_keys(&cache);
    set_static_test_sizes(&cache, id, &[100, 300]);
    let (key, width, height, _) = next_static_upload(&receiver);
    assert_eq!((width, height), (300, 150));

    // Exactly 120% and exactly 50% reuse the existing decode.
    for width in [360, 150] {
        set_static_test_sizes(&cache, id, &[width]);
        assert!(receiver.recv_timeout(Duration::from_millis(100)).is_err());
    }
    set_static_test_sizes(&cache, id, &[362]);
    assert_eq!(next_static_upload(&receiver), (key, 600, 300, false));
    set_static_test_sizes(&cache, id, &[100, 298]);
    assert_eq!(next_static_upload(&receiver), (key, 298, 149, false));
    // Removing the largest use permits a further shrink.
    set_static_test_sizes(&cache, id, &[100]);
    assert_eq!(next_static_upload(&receiver), (key, 100, 50, false));
    // A large jump is satisfied directly, capped at the original size.
    set_static_test_sizes(&cache, id, &[1500]);
    assert_eq!(next_static_upload(&receiver), (key, 1000, 500, false));

    set_static_test_sizes(&cache, id, &[]);
    assert!(cache.static_raster_image_key(id).is_none());
    let PaintMessage::UpdateImages(_, updates) =
        receiver.recv_timeout(Duration::from_secs(10)).unwrap()
    else {
        panic!("expected resource deletion");
    };
    assert!(
        matches!(&updates[..], [paint_api::ImageUpdate::DeleteImage(deleted)] if *deleted == key)
    );
    cache.clear();
}

#[test]
fn static_display_discards_obsolete_results_waiting_for_keys() {
    let (cache, receiver) = static_test_cache();
    let (id, _) = load_static_test_image(&cache);
    set_static_test_sizes(&cache, id, &[100]);
    assert!(matches!(
        receiver.recv_timeout(Duration::from_secs(10)).unwrap(),
        PaintMessage::GenerateImageKeysForPipeline(_, _)
    ));
    // The 100px decode is now queued, with no WebRender key. Replace the demand
    // before allowing either result to be uploaded.
    set_static_test_sizes(&cache, id, &[400]);
    supply_static_test_keys(&cache);
    let (_, width, height, _) = next_static_upload(&receiver);
    assert_eq!((width, height), (400, 200));
    cache.clear();
}

#[test]
fn static_display_clear_cancels_results_waiting_for_keys() {
    let (cache, receiver) = static_test_cache();
    let (id, _) = load_static_test_image(&cache);
    set_static_test_sizes(&cache, id, &[100]);
    assert!(matches!(
        receiver.recv_timeout(Duration::from_secs(10)).unwrap(),
        PaintMessage::GenerateImageKeysForPipeline(_, _)
    ));
    cache.clear();
    supply_static_test_keys(&cache);
    assert!(cache.static_raster_image_key(id).is_none());
    assert!(receiver.recv_timeout(Duration::from_millis(100)).is_err());
}

#[test]
fn static_display_completion_tracks_the_current_generation() {
    let (cache, paint_receiver) = static_test_cache();
    let (id, _) = load_static_test_image(&cache);
    let (sender, receiver) = unbounded();
    let demand = vec![(id, webrender_api::units::DeviceIntSize::new(300, 150))];
    let callback_sender = sender.clone();
    let statuses = cache.set_static_raster_demands(
        demand.clone(),
        Box::new(move |message| {
            let _ = callback_sender.send(message);
        }),
    );
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].id, id);
    assert!(statuses[0].pending);
    let generation = statuses[0].generation;
    assert!(matches!(
        paint_receiver
            .recv_timeout(Duration::from_secs(10))
            .unwrap(),
        PaintMessage::GenerateImageKeysForPipeline(_, _)
    ));
    // Register a fresh callback while the decoded result waits for a key.
    let statuses = cache.set_static_raster_demands(
        demand.clone(),
        Box::new(move |message| {
            let _ = sender.send(message);
        }),
    );
    assert_eq!(statuses[0].generation, generation);
    assert!(statuses[0].pending);
    supply_static_test_keys(&cache);
    assert!(
        matches!(receiver.recv_timeout(Duration::from_secs(10)).unwrap(),
        ImageCacheResponseMessage::StaticRasterImageReady(TEST_PIPELINE_ID, completed_id, completed_generation)
        if completed_id == id && completed_generation == generation)
    );
    let statuses = cache.set_static_raster_demands(demand, Box::new(|_| {}));
    assert_eq!(statuses[0].generation, generation);
    assert!(!statuses[0].pending);
    // Zero-sized uses release the display decode and have no pending callback.
    let statuses = cache.set_static_raster_demands(
        vec![(id, webrender_api::units::DeviceIntSize::zero())],
        Box::new(|_| {}),
    );
    assert!(statuses.is_empty());
    assert!(cache.static_raster_image_key(id).is_none());
    cache.clear();
}
