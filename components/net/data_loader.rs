/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use data_url::forgiving_base64;
use mime::Mime;
use percent_encoding::percent_decode;
use servo_url::ServoUrl;
use url::Position;

pub enum DecodeError {
    InvalidDataUri,
    NonBase64DataUri,
}

pub type DecodeData = (Mime, Vec<u8>);

pub fn decode(url: &ServoUrl) -> Result<DecodeData, DecodeError> {
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
