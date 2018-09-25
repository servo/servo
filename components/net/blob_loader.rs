/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::filemanager_thread::FileManager;
use fetch::methods::DoneChannel;
use headers_core::HeaderMapExt;
use headers_ext::{ContentLength, ContentType};
use http::header::{self, HeaderValue};
use http::HeaderMap;
use ipc_channel::ipc;
use mime::{self, Mime};
use net_traits::blob_url_store::parse_blob_url;
use net_traits::filemanager_thread::ReadFileProgress;
use net_traits::response::{Response, ResponseBody};
use net_traits::{http_percent_encode, NetworkError, ResourceFetchTiming};
use servo_url::ServoUrl;
use servo_channel::channel;

// TODO: Check on GET
// https://w3c.github.io/FileAPI/#requestResponseModel

/// https://fetch.spec.whatwg.org/#concept-basic-fetch (partial)
pub fn load_blob_async(
    url: ServoUrl,
    filemanager: FileManager,
    done_chan: &mut DoneChannel
)-> Response {
    let (id, origin) = match parse_blob_url(&url) {
        Ok((id, origin)) => (id, origin),
        Err(()) => {
            return Response::network_error(NetworkError::Internal("Invalid blob url".into()));
        }
    };

    let mut response = Response::new(url, ResourceFetchTiming::new(request.timing_type()));
    let (sender, receiver) = channel();
    *done_chan = Some((sender.clone(), receiver));
    *response.body.lock().unwrap() = ResponseBody::Receiving(vec![]);
    let check_url_validity = true;
    filemanager.fetch_file(sender, id, check_url_validity, origin, &mut response);

    response
}
