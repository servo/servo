/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Parsing stylesheets from bytes (not `&str`).

extern crate encoding;

use context::QuirksMode;
use cssparser::{stylesheet_encoding, EncodingSupport};
use error_reporting::ParseErrorReporter;
use media_queries::MediaList;
use self::encoding::{EncodingRef, DecoderTrap};
use servo_arc::Arc;
use shared_lock::SharedRwLock;
use std::str;
use stylesheets::{Stylesheet, StylesheetLoader, Origin, UrlExtraData};

struct RustEncoding;

impl EncodingSupport for RustEncoding {
    type Encoding = EncodingRef;

    fn utf8() -> Self::Encoding {
        encoding::all::UTF_8
    }

    fn is_utf16_be_or_le(encoding: &Self::Encoding) -> bool {
        matches!(encoding.name(), "utf-16be" | "utf-16le")
    }

    fn from_label(ascii_label: &[u8]) -> Option<Self::Encoding> {
        str::from_utf8(ascii_label).ok().and_then(encoding::label::encoding_from_whatwg_label)
    }
}

fn decode_stylesheet_bytes(css: &[u8], protocol_encoding_label: Option<&str>,
                           environment_encoding: Option<EncodingRef>)
                           -> (String, EncodingRef) {
    let fallback_encoding = stylesheet_encoding::<RustEncoding>(
        css, protocol_encoding_label.map(str::as_bytes), environment_encoding);
    let (result, used_encoding) = encoding::decode(css, DecoderTrap::Replace, fallback_encoding);
    (result.unwrap(), used_encoding)
}

impl Stylesheet {
    /// Parse a stylesheet from a set of bytes, potentially received over the
    /// network.
    ///
    /// Takes care of decoding the network bytes and forwards the resulting
    /// string to `Stylesheet::from_str`.
    pub fn from_bytes(bytes: &[u8],
                      url_data: UrlExtraData,
                      protocol_encoding_label: Option<&str>,
                      environment_encoding: Option<EncodingRef>,
                      origin: Origin,
                      media: MediaList,
                      shared_lock: SharedRwLock,
                      stylesheet_loader: Option<&StylesheetLoader>,
                      error_reporter: &ParseErrorReporter,
                      quirks_mode: QuirksMode)
                      -> Stylesheet {
        let (string, _) = decode_stylesheet_bytes(
            bytes, protocol_encoding_label, environment_encoding);
        Stylesheet::from_str(&string,
                             url_data,
                             origin,
                             Arc::new(shared_lock.wrap(media)),
                             shared_lock,
                             stylesheet_loader,
                             error_reporter,
                             quirks_mode,
                             0u64)
    }

    /// Updates an empty stylesheet with a set of bytes that reached over the
    /// network.
    pub fn update_from_bytes(existing: &Stylesheet,
                             bytes: &[u8],
                             protocol_encoding_label: Option<&str>,
                             environment_encoding: Option<EncodingRef>,
                             url_data: UrlExtraData,
                             stylesheet_loader: Option<&StylesheetLoader>,
                             error_reporter: &ParseErrorReporter) {
        let (string, _) = decode_stylesheet_bytes(
            bytes, protocol_encoding_label, environment_encoding);
        Self::update_from_str(existing,
                              &string,
                              url_data,
                              stylesheet_loader,
                              error_reporter,
                              0)
    }
}
