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
        let url_and_blob_claim = request.current_url_with_blob_claim();
        debug!("Loading blob {}", url_and_blob_claim.as_str());

        // Step 2.
        if request.method != Method::GET {
            return Box::pin(ready(Response::network_error(NetworkError::InvalidMethod)));
        }

        let range_header = request.headers.typed_get::<Range>();
        let is_range_request = range_header.is_some();

        let (file_id, origin) = if let Some(token) = url_and_blob_claim.token() {
            (token.file_id, token.origin.clone())
        } else {
            // FIXME: This should never happen, we should have acquired a token beforehand
            let Ok((id, _)) = parse_blob_url(&url_and_blob_claim.url()) else {
                return Box::pin(ready(Response::network_error(
                    NetworkError::ResourceLoadError("Invalid blob URL".into()),
                )));
            };
            (id, url_and_blob_claim.url().origin())
        };

        let mut response = Response::new(
            url_and_blob_claim.url(),
            ResourceFetchTiming::new(request.timing_type()),
        );
        response.status = HttpStatus::default();

        if is_range_request {
            response.range_requested = true;
            partial_content(&mut response);
        }

        let (mut done_sender, done_receiver) = unbounded_channel();
        *done_chan = Some((done_sender.clone(), done_receiver));
        *response.body.lock() = ResponseBody::Receiving(vec![]);

        if let Err(err) = context.filemanager.fetch_file(
            &mut done_sender,
            context.cancellation_listener.clone(),
            file_id,
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
            return Box::pin(ready(Response::network_error(
                NetworkError::BlobURLStoreError(err),
            )));
        };

        Box::pin(ready(response))
    }
}
