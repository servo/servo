/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use mime_classifier::MIMEClassifier;
use net_traits::ProgressMsg::{Done, Payload};
use net_traits::{LoadConsumer, LoadData, Metadata};
use resource_thread::{CancellationListener, send_error, start_sending_sniffed_opt};
use rustc_serialize::base64::FromBase64;
use std::sync::Arc;
use url::{Url, Position};
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

fn parse(url: &Url, cancel_listener: CancellationListener)
         -> Result<(Mime, Vec<u8>), Option<&'static str>> {
    // Split out content type and data.
    let parts: Vec<&str> = url[Position::BeforePath..Position::AfterQuery].splitn(2, ',').collect();
    if parts.len() != 2 {
        return Err(Some("invalid data uri"))
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

    if cancel_listener.is_cancelled() {
        return Err(None)
    }

    let mut bytes = percent_decode(parts[1].as_bytes()).collect::<Vec<_>>();
    if is_base64 {
        // FIXME(#2909): It’s unclear what to do with non-alphabet characters,
        // but Acid 3 apparently depends on spaces being ignored.
        bytes = bytes.into_iter().filter(|&b| b != ' ' as u8).collect::<Vec<u8>>();
        match bytes.from_base64() {
            Err(..) => return Err(Some("non-base64 data uri")),
            Ok(data) => bytes = data,
        }
    }
    Ok((content_type, bytes))
}

pub fn load(load_data: LoadData,
            start_chan: LoadConsumer,
            classifier: Arc<MIMEClassifier>,
            cancel_listener: CancellationListener) {
    let url = load_data.url;
    assert!(url.scheme() == "data");

    let (content_type, bytes) = match parse(&url, cancel_listener) {
        Ok((content_type, bytes)) => (content_type, bytes),
        Err(Some(s)) => return send_error(url, s.to_owned(), start_chan),
        Err(None) => return
    };

    let mut metadata = Metadata::default(url);
    metadata.set_content_type(Some(&content_type));
    if let Ok(chan) = start_sending_sniffed_opt(start_chan,
                                                metadata,
                                                classifier,
                                                &bytes,
                                                load_data.context) {
        let _ = chan.send(Payload(bytes));
        let _ = chan.send(Done(Ok(())));
    }
}
