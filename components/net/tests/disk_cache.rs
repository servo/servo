/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use http::header::CACHE_CONTROL;
use http::{HeaderValue, Method, StatusCode};
use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::{Request as HyperRequest, Response as HyperResponse};
use net::async_runtime;
use net::async_runtime::spawn_blocking_task;
use net::test_util::{make_body, make_server};
use net_traits::request::{CacheMode, Referrer, RequestBuilder};
use servo_base::id::TEST_PIPELINE_ID;
use servo_config::prefs::Preferences;

use crate::{fetch_with_context, new_fetch_context};

#[test]
fn simple_disk_restore() {
    let _runtime = async_runtime::init_async_runtime();
    let disk_cache_folder = tempfile::tempdir().unwrap();
    let mut disk_cache_path = disk_cache_folder.path().to_owned();

    disk_cache_path.push("test.sqlite");
    let disk_cache_path = disk_cache_path.as_os_str().to_string_lossy().into_owned();

    servo_config::prefs::set(Preferences {
        network_http_cache_size: 1,
        network_http_disk_cache: disk_cache_path,
        ..Default::default()
    });

    let mut fetch_context = new_fetch_context(None, None);
    {
        let cache = &fetch_context.state.http_cache;
        let disk_cache = cache.disk_cache();
        assert!(disk_cache.is_some());
        let disk_cache = disk_cache.unwrap();
        assert!(disk_cache.values_in_database().is_empty());
    }

    for _i in 0..4 {
        let handler =
            move |_request: HyperRequest<Incoming>,
                  response: &mut HyperResponse<BoxBody<Bytes, hyper::Error>>| {
                response
                    .headers_mut()
                    .insert(CACHE_CONTROL, HeaderValue::from_static("max-age=36000"));
                *response.body_mut() = make_body(vec![1, 2, 3]);
                *response.status_mut() = StatusCode::OK;
            };
        let (server, url) = make_server(handler);

        let request = RequestBuilder::new(None, url.clone(), Referrer::NoReferrer)
            .method(Method::GET)
            .cache_mode(CacheMode::ForceCache)
            .origin(url.origin())
            .pipeline_id(Some(TEST_PIPELINE_ID))
            .policy_container(Default::default())
            .build();

        let _response = fetch_with_context(request, &mut fetch_context);
        server.close();
    }

    let cache = &fetch_context.state.http_cache;
    assert_eq!(cache.len(), 1);

    let disk_cache = cache.disk_cache();
    assert!(disk_cache.is_some());
    let disk_cache = disk_cache.unwrap();

    assert_eq!(disk_cache.inner().blocking_lock().entries().len(), 4);

    let values_in_database = disk_cache.values_in_database();
    let value1 = values_in_database[0].key.clone();
    let value2 = values_in_database[1].key.clone();

    let (sender1, receiver1) = crossbeam_channel::unbounded();
    let (sender2, receiver2) = crossbeam_channel::unbounded();

    spawn_blocking_task(async move {
        let mut value = cache.get_or_guard(value1).await;
        assert!(value.try_as_mut().is_some());
        assert!(sender1.send(()).is_ok());
    });

    spawn_blocking_task(async move {
        let mut value = cache.get_or_guard(value2).await;
        assert!(value.try_as_mut().is_some());
        assert!(sender2.send(()).is_ok());
    });

    let _ = receiver1.recv();
    let _ = receiver2.recv();
}
