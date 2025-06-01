/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::future::Future;
use std::pin::Pin;

use headers::{ContentType, HeaderMapExt};
use net::fetch::methods::{DoneChannel, FetchContext};
use net::protocols::ProtocolHandler;
use net_traits::ResourceFetchTiming;
use net_traits::http_status::HttpStatus;
use net_traits::request::Request;
use net_traits::response::{Response, ResponseBody};

#[derive(Default)]
pub struct UrlInfoProtocolHander {}

// A simple protocol handler that displays information about the url itself.
impl ProtocolHandler for UrlInfoProtocolHander {
    fn load(
        &self,
        request: &mut Request,
        _done_chan: &mut DoneChannel,
        _context: &FetchContext,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let url = request.current_url();

        let content = format!(
            r#"Full url: {url}
  scheme: {}
    path: {}
   query: {:?}"#,
            url.scheme(),
            url.path(),
            url.query()
        );
        let mut response = Response::new(url, ResourceFetchTiming::new(request.timing_type()));
        *response.body.lock().unwrap() = ResponseBody::Done(content.as_bytes().to_vec());
        response.headers.typed_insert(ContentType::text());
        response.status = HttpStatus::default();

        Box::pin(std::future::ready(response))
    }

    fn is_fetchable(&self) -> bool {
        true
    }

    fn is_secure(&self) -> bool {
        true
    }
}
