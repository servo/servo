/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::future::Future;
use std::pin::Pin;

use data_url::DataUrl;
use headers::HeaderValue;
use net_traits::http_status::HttpStatus;
use net_traits::request::Request;
use net_traits::response::{Response, ResponseBody};
use net_traits::{NetworkError, ResourceFetchTiming};

use crate::fetch::methods::{DoneChannel, FetchContext};
use crate::protocols::ProtocolHandler;

#[derive(Default)]
pub struct DataProtocolHander {}

impl ProtocolHandler for DataProtocolHander {
    fn load(
        &self,
        request: &mut Request,
        _done_chan: &mut DoneChannel,
        _context: &FetchContext,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let url = request.current_url();

        assert_eq!(url.scheme(), "data");

        let response = match DataUrl::process(url.clone().as_str()) {
            Ok(data_url) => match data_url.decode_to_vec() {
                Ok((bytes, _fragment_id)) => {
                    let mut response =
                        Response::new(url, ResourceFetchTiming::new(request.timing_type()));
                    *response.body.lock().unwrap() = ResponseBody::Done(bytes);
                    let mime = data_url.mime_type();
                    response.headers.insert(
                        http::header::CONTENT_TYPE,
                        HeaderValue::from_str(&mime.to_string()).unwrap(),
                    );
                    response.status = HttpStatus::default();
                    Some(response)
                },
                Err(_) => None,
            },
            Err(_) => None,
        }
        .unwrap_or_else(|| {
            Response::network_error(NetworkError::Internal("Decoding data URL failed".into()))
        });

        Box::pin(std::future::ready(response))
    }

    fn is_fetchable(&self) -> bool {
        true
    }
}
