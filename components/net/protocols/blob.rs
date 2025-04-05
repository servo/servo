/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::future::{Future, ready};
use std::pin::Pin;

use headers::{HeaderMapExt, Range};
use http::Method;
use log::debug;
use net_traits::blob_url_store::{BlobURLStoreError, parse_blob_url};
use net_traits::http_status::HttpStatus;
use net_traits::request::Request;
use net_traits::response::{Response, ResponseBody};
use net_traits::{NetworkError, ResourceFetchTiming};
use tokio::sync::mpsc::unbounded_channel;

use crate::fetch::methods::{Data, DoneChannel, FetchContext};
use crate::protocols::{ProtocolHandler, partial_content, range_not_satisfiable_error};

#[derive(Default)]
pub struct BlobProtocolHander {}

impl ProtocolHandler for BlobProtocolHander {
    fn load(
        &self,
        request: &mut Request,
        done_chan: &mut DoneChannel,
        context: &FetchContext,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let url = request.current_url();
        debug!("Loading blob {}", url.as_str());

        // Step 2.
        if request.method != Method::GET {
            return Box::pin(ready(Response::network_error(NetworkError::Internal(
                "Unexpected method for blob".into(),
            ))));
        }

        let range_header = request.headers.typed_get::<Range>();
        let is_range_request = range_header.is_some();

        let (id, origin) = match parse_blob_url(&url) {
            Ok((id, origin)) => (id, origin),
            Err(error) => {
                return Box::pin(ready(Response::network_error(NetworkError::Internal(
                    format!("Invalid blob URL ({error})"),
                ))));
            },
        };

        let mut response = Response::new(url, ResourceFetchTiming::new(request.timing_type()));
        response.status = HttpStatus::default();

        if is_range_request {
            response.range_requested = true;
            partial_content(&mut response);
        }

        let (mut done_sender, done_receiver) = unbounded_channel();
        *done_chan = Some((done_sender.clone(), done_receiver));
        *response.body.lock().unwrap() = ResponseBody::Receiving(vec![]);

        if let Err(err) = context.filemanager.lock().unwrap().fetch_file(
            &mut done_sender,
            context.cancellation_listener.clone(),
            id,
            &context.file_token,
            origin,
            &mut response,
            range_header,
        ) {
            let _ = done_sender.send(Data::Done);
            let err = match err {
                BlobURLStoreError::InvalidRange => {
                    range_not_satisfiable_error(&mut response);
                    return Box::pin(ready(response));
                },
                _ => format!("{:?}", err),
            };
            return Box::pin(ready(Response::network_error(NetworkError::Internal(err))));
        };

        Box::pin(ready(response))
    }
}
