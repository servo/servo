/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::future::Future;
use std::pin::Pin;

use data_url::forgiving_base64;
use headers::{ContentType, HeaderMapExt};
use http::StatusCode;
use mime::Mime;
use net_traits::request::Request;
use net_traits::response::{Response, ResponseBody};
use net_traits::{NetworkError, ResourceFetchTiming};
use percent_encoding::percent_decode;
use servo_url::ServoUrl;
use url::Position;

use crate::fetch::methods::{DoneChannel, FetchContext};
use crate::protocols::ProtocolHandler;

#[derive(Default)]
pub struct DataProtocolHander {}

enum DecodeError {
    InvalidDataUri,
    NonBase64DataUri,
}

type DecodeData = (Mime, Vec<u8>);

fn decode(url: &ServoUrl) -> Result<DecodeData, DecodeError> {
    // data_url could do all of this work for us,
    // except that it currently (Nov 2019) parses mime types into a
    // different Mime class than other code expects

    assert_eq!(url.scheme(), "data");
    // Split out content type and data.
    let parts: Vec<&str> = url[Position::BeforePath..Position::AfterQuery]
        .splitn(2, ',')
        .collect();
    if parts.len() != 2 {
        return Err(DecodeError::InvalidDataUri);
    }

    // ";base64" must come at the end of the content type, per RFC 2397.
    // rust-http will fail to parse it because there's no =value part.
    let mut ct_str = parts[0];
    let is_base64 = ct_str.ends_with(";base64");
    if is_base64 {
        ct_str = &ct_str[..ct_str.len() - ";base64".len()];
    }
    let ct_str = if ct_str.starts_with(";charset=") {
        format!("text/plain{}", ct_str)
    } else {
        ct_str.to_owned()
    };

    let content_type = ct_str
        .parse()
        .unwrap_or_else(|_| "text/plain; charset=US-ASCII".parse().unwrap());

    let mut bytes = percent_decode(parts[1].as_bytes()).collect::<Vec<_>>();
    if is_base64 {
        match forgiving_base64::decode_to_vec(&bytes) {
            Err(..) => return Err(DecodeError::NonBase64DataUri),
            Ok(data) => bytes = data,
        }
    }
    Ok((content_type, bytes))
}

impl ProtocolHandler for DataProtocolHander {
    fn load(
        &self,
        request: &mut Request,
        _done_chan: &mut DoneChannel,
        _context: &FetchContext,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let url = request.current_url();
        let response = match decode(&url) {
            Ok((mime, bytes)) => {
                let mut response =
                    Response::new(url, ResourceFetchTiming::new(request.timing_type()));
                *response.body.lock().unwrap() = ResponseBody::Done(bytes);
                response.headers.typed_insert(ContentType::from(mime));
                response.status = Some((StatusCode::OK, "OK".to_string()));
                response.raw_status = Some((StatusCode::OK.as_u16(), b"OK".to_vec()));
                response
            },
            Err(_) => {
                Response::network_error(NetworkError::Internal("Decoding data URL failed".into()))
            },
        };
        Box::pin(std::future::ready(response))
    }
}
