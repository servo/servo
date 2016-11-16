/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::mime::{Attr, Mime, SubLevel, TopLevel, Value};
use mime_classifier::MimeClassifier;
use net_traits::{LoadData, Metadata, NetworkError};
use net_traits::LoadConsumer;
use net_traits::ProgressMsg::{Done, Payload};
use resource_thread::{CancellationListener, send_error, start_sending_sniffed_opt};
use rustc_serialize::base64::FromBase64;
use servo_url::ServoUrl;
use std::sync::Arc;
use url::Position;
use url::percent_encoding::percent_decode;

pub fn factory(load_data: LoadData,
               senders: LoadConsumer,
               classifier: Arc<MimeClassifier>,
               cancel_listener: CancellationListener) {
    // NB: we don't spawn a new thread.
    // Hypothesis: data URLs are too small for parallel base64 etc. to be worth it.
    // Should be tested at some point.
    // Left in separate function to allow easy moving to a thread, if desired.
    load(load_data, senders, classifier, cancel_listener)
}

pub enum DecodeError {
    InvalidDataUri,
    NonBase64DataUri,
}

pub type DecodeData = (Mime, Vec<u8>);

pub fn decode(url: &ServoUrl) -> Result<DecodeData, DecodeError> {
    assert_eq!(url.scheme(), "data");
    // Split out content type and data.
    let parts: Vec<&str> = url.as_url().unwrap()[Position::BeforePath..Position::AfterQuery].splitn(2, ',').collect();
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

    let content_type = ct_str.parse().unwrap_or_else(|_| {
        Mime(TopLevel::Text, SubLevel::Plain,
             vec![(Attr::Charset, Value::Ext("US-ASCII".to_owned()))])
    });

    let mut bytes = percent_decode(parts[1].as_bytes()).collect::<Vec<_>>();
    if is_base64 {
        // FIXME(#2909): It’s unclear what to do with non-alphabet characters,
        // but Acid 3 apparently depends on spaces being ignored.
        bytes = bytes.into_iter().filter(|&b| b != b' ').collect::<Vec<u8>>();
        match bytes.from_base64() {
            Err(..) => return Err(DecodeError::NonBase64DataUri),
            Ok(data) => bytes = data,
        }
    }
    Ok((content_type, bytes))
}

pub fn load(load_data: LoadData,
            start_chan: LoadConsumer,
            classifier: Arc<MimeClassifier>,
            cancel_listener: CancellationListener) {
    let url = load_data.url;

    if cancel_listener.is_cancelled() {
        return;
    }

    match decode(&url) {
        Ok((content_type, bytes)) => {
            let mut metadata = Metadata::default(url);
            metadata.set_content_type(Some(content_type).as_ref());
            if let Ok(chan) = start_sending_sniffed_opt(start_chan,
                                                metadata,
                                                classifier,
                                                &bytes,
                                                load_data.context) {
                let _ = chan.send(Payload(bytes));
                let _ = chan.send(Done(Ok(())));
            }
        },
        Err(DecodeError::InvalidDataUri) =>
            send_error(url, NetworkError::Internal("invalid data uri".to_owned()), start_chan),
        Err(DecodeError::NonBase64DataUri) =>
            send_error(url, NetworkError::Internal("non-base64 data uri".to_owned()), start_chan),
    }
}
