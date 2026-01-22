/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;
use std::time::Duration;

use base::id::{PipelineId, TEST_PIPELINE_ID, TEST_WEBVIEW_ID};
use crossbeam_channel::{Receiver, Sender, unbounded};
use net::image_cache::ImageCacheFactoryImpl;
use net_traits::image_cache::{
    ImageCache, ImageCacheFactory, ImageCacheResponseMessage, ImageCacheResult, ImageLoadListener,
    ImageOrMetadataAvailable, ImageResponse, PendingImageId, PendingImageResponse,
};
use net_traits::request::RequestId;
use net_traits::{
    DebugVec, FetchMetadata, FetchResponseMsg, FilteredMetadata, Metadata, NetworkError,
    ResourceFetchTiming, ResourceTimingType,
};
use paint_api::{CrossProcessPaintApi, PaintMessage};
use servo_url::ServoUrl;
use uuid::Uuid;
use webrender_api::ImageKey;

use crate::mock_origin;

fn create_test_image_cache() -> (Arc<dyn ImageCache>, Receiver<PipelineId>) {
    let (sender, receiver) = unbounded();
    let paint_api = CrossProcessPaintApi::dummy_with_callback(Some(Box::new(move |msg| {
        if let PaintMessage::GenerateImageKeysForPipeline(_, pipeline_id) = msg {
            let _ = sender.send(pipeline_id);
        }
    })));

    let factory = ImageCacheFactoryImpl::new(vec![]);
    let cache = factory.create(TEST_WEBVIEW_ID, TEST_PIPELINE_ID, &paint_api);
    (cache, receiver)
}

fn handle_pending_key_requests(cache: &Arc<dyn ImageCache>, receiver: &Receiver<PipelineId>) {
    while let Ok(_pipeline_id) = receiver.try_recv() {
        let keys: Vec<_> = (0..10)
            .map(|i| ImageKey::new(webrender_api::IdNamespace(42), i as u32))
            .collect();
        cache.fill_key_cache_with_batch_of_keys(keys);
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
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), DebugVec(small_chunk)),
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
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), DebugVec(metadata_chunk)),
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
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), DebugVec(jpeg_bytes)),
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
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), DebugVec(jpeg_bytes)),
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
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), DebugVec(metadata_chunk)),
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
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), DebugVec(jpeg_bytes)),
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
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), DebugVec(jpeg_bytes)),
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
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), DebugVec(svg_bytes)),
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
    cache.rasterize_vector_image(vec_img.id, size);
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
        FetchResponseMsg::ProcessResponseChunk(create_request_id(), DebugVec(svg_bytes)),
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

    cache.rasterize_vector_image(vec_img.id, size);

    cache.add_rasterization_complete_listener(TEST_PIPELINE_ID, vec_img.id, size, callback);

    loop {
        handle_pending_key_requests(&cache, &key_receiver);
        if notified.load(Ordering::SeqCst) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
