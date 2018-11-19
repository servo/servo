/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use hyper::header::{Expires, HttpDate};
use hyper::method::Method;
use hyper::status::StatusCode;
use msg::constellation_msg::TEST_PIPELINE_ID;
use net::http_cache::HttpCache;
use net_traits::request::{Destination, Request, RequestInit};
use net_traits::response::{Response, ResponseBody};
use servo_url::ServoUrl;
use std::sync::mpsc::channel;


#[test]
fn test_refreshing_resource_sets_done_chan_the_appropriate_value() {
    let response_bodies = vec![ResponseBody::Receiving(vec![]),
                               ResponseBody::Empty,
                               ResponseBody::Done(vec![])];
    let url = ServoUrl::parse("https://servo.org").unwrap();
    let request = Request::from_init(RequestInit {
        url: url.clone(),
        method: Method::Get,
        destination: Destination::Document,
        origin: url.clone().origin(),
        pipeline_id: Some(TEST_PIPELINE_ID),
        .. RequestInit::default()
    });
    let mut response = Response::new(url.clone());
    // Expires header makes the response cacheable.
    response.headers.set(Expires(HttpDate(time::now())));
    response_bodies.iter().for_each(|body| {
        let mut cache = HttpCache::new();
        *response.body.lock().unwrap() = body;
        // First, store the 'normal' response.
        cache.store(&request, &response);
        // Second, mutate the response into a 304 response, and refresh the stored one.
        response.status = Some(StatusCode::NotModified);
        let mut done_chan = Some(channel());
        let refreshed_response = cache.refresh(&request, response, &mut done_chan);
        // Ensure a resource was found, and refreshed.
        assert!(refreshed_response.is_some());
        match body {
            ResponseBody::Receiving(_) => assert!(done_chan.is_some()),
            ResponseBody::Empty | ResponseBody::Done(_) => assert!(done_chan.is_none())
        }
    })
}
