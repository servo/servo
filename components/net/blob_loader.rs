/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use filemanager_thread::FileManager;
use headers_core::HeaderMapExt;
use headers_ext::{ContentLength, ContentType};
use http::HeaderMap;
use http::header::{self, HeaderValue};
use ipc_channel::ipc;
use mime::{self, Mime};
use net_traits::{http_percent_encode, NetworkError};
use net_traits::blob_url_store::parse_blob_url;
use net_traits::filemanager_thread::ReadFileProgress;
use servo_url::ServoUrl;

// TODO: Check on GET
// https://w3c.github.io/FileAPI/#requestResponseModel

/// https://fetch.spec.whatwg.org/#concept-basic-fetch (partial)
// TODO: make async.
pub fn load_blob_sync
            (url: ServoUrl,
             filemanager: FileManager)
             -> Result<(HeaderMap, Vec<u8>), NetworkError> {
    let (id, origin) = match parse_blob_url(&url) {
        Ok((id, origin)) => (id, origin),
        Err(()) => {
            let e = format!("Invalid blob URL format {:?}", url);
            return Err(NetworkError::Internal(e));
        }
    };

    let (sender, receiver) = ipc::channel().unwrap();
    let check_url_validity = true;
    filemanager.read_file(sender, id, check_url_validity, origin);

    let blob_buf = match receiver.recv().unwrap() {
        Ok(ReadFileProgress::Meta(blob_buf)) => blob_buf,
        Ok(_) => {
            return Err(NetworkError::Internal("Invalid filemanager reply".to_string()));
        }
        Err(e) => {
            return Err(NetworkError::Internal(format!("{:?}", e)));
        }
    };

    let content_type: Mime = blob_buf.type_string.parse().unwrap_or(mime::TEXT_PLAIN);
    let charset = content_type.get_param(mime::CHARSET);

    let mut headers = HeaderMap::new();

    if let Some(name) = blob_buf.filename {
        let charset = charset.map(|c| c.as_ref().into()).unwrap_or("us-ascii".to_owned());
        // TODO(eijebong): Replace this once the typed header is there
        headers.insert(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_bytes(
                format!("inline; {}",
                    if charset.to_lowercase() == "utf-8" {
                        format!("filename=\"{}\"", String::from_utf8(name.as_bytes().into()).unwrap())
                    } else {
                        format!("filename*=\"{}\"''{}", charset, http_percent_encode(name.as_bytes()))
                    }
                ).as_bytes()
            ).unwrap()
        );
    }

    // Basic fetch, Step 4.
    headers.typed_insert(ContentLength(blob_buf.size as u64));
    // Basic fetch, Step 5.
    headers.typed_insert(ContentType::from(content_type.clone()));

    let mut bytes = blob_buf.bytes;
    loop {
        match receiver.recv().unwrap() {
            Ok(ReadFileProgress::Partial(ref mut new_bytes)) => {
                bytes.append(new_bytes);
            }
            Ok(ReadFileProgress::EOF) => {
                return Ok((headers, bytes));
            }
            Ok(_) => {
                return Err(NetworkError::Internal("Invalid filemanager reply".to_string()));
            }
            Err(e) => {
                return Err(NetworkError::Internal(format!("{:?}", e)));
            }
        }
    }
}
