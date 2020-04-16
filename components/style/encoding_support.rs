/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Parsing stylesheets from bytes (not `&str`).

extern crate encoding_rs;

use crate::context::QuirksMode;
use crate::error_reporting::ParseErrorReporter;
use crate::media_queries::MediaList;
use crate::shared_lock::SharedRwLock;
use crate::stylesheets::{AllowImportRules, Origin, Stylesheet, StylesheetLoader, UrlExtraData};
use cssparser::{stylesheet_encoding, EncodingSupport};
use servo_arc::Arc;
use std::borrow::Cow;
use std::str;

struct EncodingRs;

impl EncodingSupport for EncodingRs {
    type Encoding = &'static encoding_rs::Encoding;

    fn utf8() -> Self::Encoding {
        encoding_rs::UTF_8
    }

    fn is_utf16_be_or_le(encoding: &Self::Encoding) -> bool {
        *encoding == encoding_rs::UTF_16LE || *encoding == encoding_rs::UTF_16BE
    }

    fn from_label(ascii_label: &[u8]) -> Option<Self::Encoding> {
        encoding_rs::Encoding::for_label(ascii_label)
    }
}

fn decode_stylesheet_bytes<'a>(
    css: &'a [u8],
    protocol_encoding_label: Option<&str>,
    environment_encoding: Option<&'static encoding_rs::Encoding>,
) -> Cow<'a, str> {
    let fallback_encoding = stylesheet_encoding::<EncodingRs>(
        css,
        protocol_encoding_label.map(str::as_bytes),
        environment_encoding,
    );
    let (result, _used_encoding, _) = fallback_encoding.decode(&css);
    // FIXME record used encoding for environment encoding of @import
    result
}

impl Stylesheet {
    /// Parse a stylesheet from a set of bytes, potentially received over the
    /// network.
    ///
    /// Takes care of decoding the network bytes and forwards the resulting
    /// string to `Stylesheet::from_str`.
    pub fn from_bytes(
        bytes: &[u8],
        url_data: UrlExtraData,
        protocol_encoding_label: Option<&str>,
        environment_encoding: Option<&'static encoding_rs::Encoding>,
        origin: Origin,
        media: MediaList,
        shared_lock: SharedRwLock,
        stylesheet_loader: Option<&dyn StylesheetLoader>,
        error_reporter: Option<&dyn ParseErrorReporter>,
        quirks_mode: QuirksMode,
    ) -> Stylesheet {
        let string = decode_stylesheet_bytes(bytes, protocol_encoding_label, environment_encoding);
        Stylesheet::from_str(
            &string,
            url_data,
            origin,
            Arc::new(shared_lock.wrap(media)),
            shared_lock,
            stylesheet_loader,
            error_reporter,
            quirks_mode,
            0,
            AllowImportRules::Yes,
        )
    }

    /// Updates an empty stylesheet with a set of bytes that reached over the
    /// network.
    pub fn update_from_bytes(
        existing: &Stylesheet,
        bytes: &[u8],
        protocol_encoding_label: Option<&str>,
        environment_encoding: Option<&'static encoding_rs::Encoding>,
        url_data: UrlExtraData,
        stylesheet_loader: Option<&dyn StylesheetLoader>,
        error_reporter: Option<&dyn ParseErrorReporter>,
    ) {
        let string = decode_stylesheet_bytes(bytes, protocol_encoding_label, environment_encoding);
        Self::update_from_str(
            existing,
            &string,
            url_data,
            stylesheet_loader,
            error_reporter,
            0,
            AllowImportRules::Yes,
        )
    }
}
