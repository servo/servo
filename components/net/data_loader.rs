/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use mime_classifier::MIMEClassifier;
use net_traits::LoadConsumer;
use net_traits::ProgressMsg::{Payload, Done};
use net_traits::{LoadData, Metadata};
use resource_thread::{CancellationListener, send_error, start_sending_sniffed_opt};
use rustc_serialize::base64::FromBase64;
use std::sync::Arc;
use url::SchemeData;
use url::Url;
use url::percent_encoding::percent_decode;

pub fn factory(load_data: LoadData,
               senders: LoadConsumer,
               classifier: Arc<MIMEClassifier>,
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

pub fn decode(url: &Url) -> Result<DecodeData, DecodeError> {
    assert!(&*url.scheme == "data");
    // Split out content type and data.
    let mut scheme_data = match url.scheme_data {
        SchemeData::NonRelative(ref scheme_data) => scheme_data.clone(),
        _ => panic!("Expected a non-relative scheme URL."),
    };
    match url.query {
        Some(ref query) => {
            scheme_data.push_str("?");
            scheme_data.push_str(query);
        },
        None => ()
    }
    let parts: Vec<&str> = scheme_data.splitn(2, ',').collect();
    if parts.len() != 2 {
        return Err(DecodeError::InvalidDataUri);
    }

    // ";base64" must come at the end of the content type, per RFC 2397.
    // rust-http will fail to parse it because there's no =value part.
    let mut is_base64 = false;
    let mut ct_str = parts[0].to_owned();
    if ct_str.ends_with(";base64") {
        is_base64 = true;
        let end_index = ct_str.len() - 7;
        ct_str.truncate(end_index);
    }
    if ct_str.starts_with(";charset=") {
        ct_str = format!("text/plain{}", ct_str);
    }

    // Parse the content type using rust-http.
    // FIXME: this can go into an infinite loop! (rust-http #25)
    let mut content_type: Option<Mime> = ct_str.parse().ok();
    if content_type == None {
        content_type = Some(Mime(TopLevel::Text, SubLevel::Plain,
                                 vec!((Attr::Charset, Value::Ext("US-ASCII".to_owned())))));
    }

    let bytes = percent_decode(parts[1].as_bytes());
    let bytes = if is_base64 {
        // FIXME(#2909): It’s unclear what to do with non-alphabet characters,
        // but Acid 3 apparently depends on spaces being ignored.
        let bytes = bytes.into_iter().filter(|&b| b != ' ' as u8).collect::<Vec<u8>>();
        match bytes.from_base64() {
            Err(..) => return Err(DecodeError::NonBase64DataUri),
            Ok(data) => data,
        }
    } else {
        bytes
    };
    Ok((content_type.unwrap(), bytes))
}

pub fn load(load_data: LoadData,
            start_chan: LoadConsumer,
            classifier: Arc<MIMEClassifier>,
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
        Err(DecodeError::InvalidDataUri) => send_error(url, "invalid data uri".to_owned(), start_chan),
        Err(DecodeError::NonBase64DataUri) => send_error(url, "non-base64 data uri".to_owned(), start_chan),
    }
}
