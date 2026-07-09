/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use http::header::{CACHE_CONTROL, CONTENT_LENGTH, CONTENT_RANGE, EXPIRES, HeaderValue, RANGE};
use http::{HeaderMap, StatusCode};
use net::http_cache::{CacheKey, HttpCache, ValidationStatus, refresh};
use net_traits::blob_url_store::UrlWithBlobClaim;
use net_traits::request::{Referrer, RequestBuilder};
use net_traits::response::{Response, ResponseBody};
use net_traits::{ResourceFetchTiming, ResourceTimingType};
use servo_base::id::TEST_PIPELINE_ID;
use servo_url::ServoUrl;
use tokio::sync::mpsc::unbounded_channel as unbounded;

#[tokio::test]
async fn test_refreshing_resource_sets_done_chan_the_appropriate_value() {
    let response_bodies = vec![
        ResponseBody::Receiving(vec![]),
        ResponseBody::Empty,
        ResponseBody::Done(vec![]),
    ];
    let url = ServoUrl::parse("https://servo.org").unwrap();
    let request = RequestBuilder::new(
        None,
        UrlWithBlobClaim::new(url.clone(), None),
        Referrer::NoReferrer,
    )
    .pipeline_id(Some(TEST_PIPELINE_ID))
    .origin(url.origin())
    .build();
    let timing = ResourceFetchTiming::new(ResourceTimingType::Navigation);
    let mut response = Response::new(url.clone(), timing);
    // Expires header makes the response cacheable.
    response
        .headers
        .insert(EXPIRES, HeaderValue::from_str("-10").unwrap());
    let cache = HttpCache::default();
    for body in response_bodies {
        *response.body.lock() = body.clone();
        // First, store the 'normal' response.
        let mut resource = {
            let guard = cache.get_or_guard(CacheKey::new(&request)).await;
            guard.insert(&request, &response);
            cache.get_or_guard(CacheKey::new(&request)).await
        };
        // Second, mutate the response into a 304 response, and refresh the stored one.
        response.status = StatusCode::NOT_MODIFIED.into();
        let (send, recv) = unbounded();
        let mut done_chan = Some((send, recv));
        let refreshed_response = refresh(
            &request,
            response.clone(),
            &mut done_chan,
            resource.try_as_mut().unwrap(),
        );
        // Ensure a resource was found, and refreshed.
        assert!(refreshed_response.is_some());
        match body {
            ResponseBody::Receiving(_) => assert!(done_chan.is_some()),
            ResponseBody::Empty | ResponseBody::Done(_) => assert!(done_chan.is_none()),
        }
    }
}

#[tokio::test]
async fn test_skip_incomplete_cache_for_range_request_with_no_end_bound() {
    let actual_body_len = 10;
    let incomplete_response_body = &[1, 2, 3, 4, 5];
    let url = ServoUrl::parse("https://servo.org").unwrap();

    let cache = HttpCache::default();
    let mut headers = HeaderMap::new();

    headers.insert(
        RANGE,
        HeaderValue::from_str(&format!("bytes={}-", 0)).unwrap(),
    );
    let request = RequestBuilder::new(
        None,
        UrlWithBlobClaim::new(url.clone(), None),
        Referrer::NoReferrer,
    )
    .pipeline_id(Some(TEST_PIPELINE_ID))
    .origin(url.origin())
    .headers(headers)
    .build();

    // Store incomplete response to http_cache
    let timing = ResourceFetchTiming::new(ResourceTimingType::Navigation);
    let mut initial_incomplete_response = Response::new(url.clone(), timing);
    *initial_incomplete_response.body.lock() =
        ResponseBody::Done(incomplete_response_body.to_vec());
    initial_incomplete_response.headers.insert(
        CONTENT_RANGE,
        HeaderValue::from_str(&format!(
            "bytes 0-{}/{}",
            actual_body_len - 1,
            actual_body_len
        ))
        .unwrap(),
    );
    initial_incomplete_response.headers.insert(
        CONTENT_LENGTH,
        HeaderValue::from_str(&format!("{}", actual_body_len)).unwrap(),
    );
    initial_incomplete_response
        .headers
        .insert(EXPIRES, HeaderValue::from_str("0").unwrap());
    initial_incomplete_response.status = StatusCode::PARTIAL_CONTENT.into();
    cache.store(&request, &initial_incomplete_response).await;

    // Try to construct response from http_cache
    let mut done_chan = None;
    let consecutive_response = cache.construct_response(&request, &mut done_chan).await;
    assert!(
        consecutive_response.is_none(),
        "Should not construct response from incomplete response!"
    );
}

fn build_stale_while_revalidate_test_request() -> net_traits::request::Request {
    let url = ServoUrl::parse("https://servo.org").unwrap();
    RequestBuilder::new(
        None,
        UrlWithBlobClaim::new(url.clone(), None),
        Referrer::NoReferrer,
    )
    .pipeline_id(Some(TEST_PIPELINE_ID))
    .origin(url.origin())
    .build()
}

/// Store a response with the given `Cache-Control` value, then return the
async fn stale_while_revalidate_freshness_for_cache_control(
    cache_control: &str,
) -> ValidationStatus {
    let url = ServoUrl::parse("https://servo.org").unwrap();
    let request = build_stale_while_revalidate_test_request();

    let timing = ResourceFetchTiming::new(ResourceTimingType::Navigation);
    let mut response = Response::new(url, timing);
    *response.body.lock() = ResponseBody::Done(vec![1, 2, 3]);
    response
        .headers
        .insert(CACHE_CONTROL, HeaderValue::from_str(cache_control).unwrap());

    let cache = HttpCache::default();
    cache.store(&request, &response).await;

    let mut done_chan = None;
    cache
        .construct_response_freshness(&request, &mut done_chan)
        .await
        .expect("a response should be constructable from the cache")
}

#[tokio::test]
async fn test_stale_within_stale_while_revalidate_window_serves_immediately_and_revalidates_in_background()
 {
    let validation_status =
        stale_while_revalidate_freshness_for_cache_control("max-age=0, stale-while-revalidate=30")
            .await;
    assert_eq!(
        validation_status,
        ValidationStatus::Stale {
            revalidate_in_background: true
        },
        "stale response within the stale-while-revalidate window should be served immediately \
         and revalidated in the background"
    );
}

#[tokio::test]
async fn test_stale_without_stale_while_revalidate_requires_synchronous_validation() {
    let validation_status = stale_while_revalidate_freshness_for_cache_control("max-age=0").await;
    assert_eq!(
        validation_status,
        ValidationStatus::Stale {
            revalidate_in_background: false
        },
        "stale response without stale-while-revalidate must be synchronously revalidated"
    );
}

#[tokio::test]
async fn test_stale_while_revalidate_not_used_when_request_demands_revalidation() {
    let url = ServoUrl::parse("https://servo.org").unwrap();

    let timing = ResourceFetchTiming::new(ResourceTimingType::Navigation);
    let mut response = Response::new(url.clone(), timing);
    *response.body.lock() = ResponseBody::Done(vec![1, 2, 3]);
    response.headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_str("max-age=0, stale-while-revalidate=30").unwrap(),
    );

    let store_request = build_stale_while_revalidate_test_request();
    let cache = HttpCache::default();
    cache.store(&store_request, &response).await;

    let mut req_headers = HeaderMap::new();
    req_headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    let request = RequestBuilder::new(
        None,
        UrlWithBlobClaim::new(url.clone(), None),
        Referrer::NoReferrer,
    )
    .pipeline_id(Some(TEST_PIPELINE_ID))
    .origin(url.origin())
    .headers(req_headers)
    .build();

    let mut done_chan = None;
    let validation_status = cache
        .construct_response_freshness(&request, &mut done_chan)
        .await
        .expect("a response should be constructable from the cache");
    assert_eq!(
        validation_status,
        ValidationStatus::Stale {
            revalidate_in_background: false
        },
        "a no-cache request must trigger synchronous validation, not background revalidation"
    );
}
